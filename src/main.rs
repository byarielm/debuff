use std::sync::Arc;

use tokio::sync::broadcast;
use tracing_subscriber::{fmt, EnvFilter};

use debuff::auth::oauth_store::{DbSessionStore, DbStateStore};
use debuff::config::Config;
use debuff::server::router;
use debuff::signing::LabelSigner;
use debuff::AppState;

use atrium_identity::did::{CommonDidResolver, CommonDidResolverConfig};
use atrium_identity::handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig};
use atrium_identity::handle::{DohDnsTxtResolver, DohDnsTxtResolverConfig};
use atrium_oauth::{
    AtprotoClientMetadata, AtprotoLocalhostClientMetadata, AuthMethod, GrantType, KnownScope, Scope,
    DefaultHttpClient, OAuthClientConfig, OAuthResolverConfig,
};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let config = Config::load();
    let db = debuff::db::connect(&config.database).await;

    let signer = match &config.labeler.signing_key_path {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(pem) => LabelSigner::from_pem(&pem).expect("Failed to parse signing key"),
            Err(e) => {
                tracing::warn!(
                    "Could not read signing key from {path}: {e} — generating an ephemeral key."
                );
                LabelSigner::generate()
            }
        },
        None => {
            tracing::warn!(
                "No SIGNING_KEY_PATH set — generating an ephemeral signing key. \
                 Labels will not be verifiable until a persistent key is registered in the DID document."
            );
            LabelSigner::generate()
        }
    };

    let (label_tx, _) = broadcast::channel::<i64>(1024);
    let http = reqwest::Client::new();

    // Build atrium-oauth client
    let callback_url = format!("{}/auth/callback", config.server.public_url.trim_end_matches('/'));
    let atrium_http = Arc::new(DefaultHttpClient::default());

    let did_resolver = CommonDidResolver::new(CommonDidResolverConfig {
        plc_directory_url: config.labeler.plc_url.clone(),
        http_client: Arc::clone(&atrium_http),
    });

    let handle_resolver = AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
        dns_txt_resolver: DohDnsTxtResolver::new(DohDnsTxtResolverConfig {
            service_url: "https://dns.google/dns-query".into(),
            http_client: Arc::clone(&atrium_http),
        }),
        http_client: Arc::clone(&atrium_http),
    });

    let is_loopback = config.server.public_url.contains("127.0.0.1")
        || config.server.public_url.contains("[::1]");

    let resolver_config = OAuthResolverConfig {
        did_resolver,
        handle_resolver,
        authorization_server_metadata: Default::default(),
        protected_resource_metadata: Default::default(),
    };

    let oauth_client = if is_loopback {
        tracing::info!("Using loopback OAuth client metadata (local development)");
        atrium_oauth::OAuthClient::new(OAuthClientConfig {
            client_metadata: AtprotoLocalhostClientMetadata {
                redirect_uris: Some(vec![callback_url]),
                scopes: Some(vec![Scope::Known(KnownScope::Atproto), Scope::Known(KnownScope::TransitionGeneric), Scope::Unknown("identity:*".into())]),
            },
            keys: None,
            state_store: DbStateStore::new(db.clone()),
            session_store: DbSessionStore::new(db.clone()),
            resolver: resolver_config,
        })
        .expect("Failed to create OAuth client")
    } else {
        atrium_oauth::OAuthClient::new(OAuthClientConfig {
            client_metadata: AtprotoClientMetadata {
                client_id: format!("{}/oauth/client-metadata.json", config.server.public_url.trim_end_matches('/')),
                client_uri: Some(config.server.public_url.clone()),
                redirect_uris: vec![callback_url],
                token_endpoint_auth_method: AuthMethod::None,
                grant_types: vec![GrantType::AuthorizationCode, GrantType::RefreshToken],
                scopes: vec![Scope::Known(KnownScope::Atproto), Scope::Known(KnownScope::TransitionGeneric), Scope::Unknown("identity:*".into())],
                jwks_uri: None,
                token_endpoint_auth_signing_alg: None,
            },
            keys: None,
            state_store: DbStateStore::new(db.clone()),
            session_store: DbSessionStore::new(db.clone()),
            resolver: resolver_config,
        })
        .expect("Failed to create OAuth client")
    };

    if config.server.session_secret == "change-me-in-production" {
        tracing::warn!(
            "No SESSION_SECRET set — using an insecure default. \
             Set SESSION_SECRET to a random string in production."
        );
    }

    let cookie_key = axum_extra::extract::cookie::Key::derive_from(
        config.server.session_secret.as_bytes(),
    );

    let state = AppState {
        config: config.clone(),
        db,
        http,
        label_broadcast: label_tx,
        signer: Arc::new(signer),
        oauth: Arc::new(oauth_client),
        cookie_key,
        setup_labeler_did: Arc::new(tokio::sync::Mutex::new(None)),
    };

    let app = router(state);
    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
