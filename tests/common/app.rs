use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use cookie::{CookieJar, Key};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::AnyPool;
use tempfile::TempDir;
use tokio::sync::broadcast;
use tower::ServiceExt;
use wiremock::MockServer;

use debuff::AppState;
use debuff::auth::COOKIE_NAME;
use debuff::auth::oauth_store::{DbSessionStore, DbStateStore};
use debuff::config::{Config, DatabaseBackend, DatabaseConfig, LabelerConfig, ServerConfig};
use debuff::db::adapt_sql;
use debuff::signing::LabelSigner;

use super::db::{self, Backend};

use debuff::dns::NativeDnsResolver;

use atrium_identity::did::{CommonDidResolver, CommonDidResolverConfig};
use atrium_identity::handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig};
use atrium_oauth::{
    AtprotoLocalhostClientMetadata, DefaultHttpClient, KnownScope, OAuthClientConfig,
    OAuthResolverConfig, Scope,
};

pub const ADMIN_DID: &str = "did:plc:testadmin";

pub struct TestApp {
    pub router: Router,
    pub state: AppState,
    pub pool: AnyPool,
    pub mock_server: MockServer,
    pub admin_cookie: String,
    _temp_dir: TempDir,
}

impl TestApp {
    pub async fn new(backend: Backend) -> Self {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let mock_server = MockServer::start().await;

        let pool = db::test_pool(backend, temp_dir.path()).await;
        db::truncate_all(&pool).await;

        let db_backend = match backend {
            Backend::Sqlite => DatabaseBackend::Sqlite,
            Backend::Postgres => DatabaseBackend::Postgres,
        };

        // Seed admin moderator
        sqlx::query(&adapt_sql(
            "INSERT INTO moderators (did, role) VALUES ($1, 'admin') \
             ON CONFLICT (did) DO NOTHING",
            db_backend.clone(),
        ))
        .bind(ADMIN_DID)
        .execute(&pool)
        .await
        .expect("failed to seed admin moderator");

        let signer = LabelSigner::generate();
        let (label_tx, _) = broadcast::channel::<i64>(1024);
        let http = reqwest::Client::new();

        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 0,
                public_url: "http://127.0.0.1:3001".into(),
                static_dir: temp_dir.path().join("static").to_string_lossy().into(),
                session_secret: "test-secret-at-least-64-bytes-long-for-key-derivation-to-work-ok"
                    .into(),
            },
            database: DatabaseConfig {
                backend: db_backend.clone(),
                ..DatabaseConfig::default()
            },
            labeler: LabelerConfig {
                did: "did:plc:testlabeler".into(),
                signing_key_path: None,
                plc_url: mock_server.uri(),
            },
        };

        let cookie_key =
            axum_extra::extract::cookie::Key::derive_from(config.server.session_secret.as_bytes());

        // Build OAuth client (loopback mode)
        let atrium_http = Arc::new(DefaultHttpClient::default());
        let did_resolver = CommonDidResolver::new(CommonDidResolverConfig {
            plc_directory_url: config.labeler.plc_url.clone(),
            http_client: Arc::clone(&atrium_http),
        });
        let dns = NativeDnsResolver::new();
        let handle_resolver = AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
            dns_txt_resolver: dns.clone(),
            http_client: Arc::clone(&atrium_http),
        });
        let oauth_client = atrium_oauth::OAuthClient::new(OAuthClientConfig {
            client_metadata: AtprotoLocalhostClientMetadata {
                redirect_uris: Some(vec!["http://127.0.0.1:3001/auth/callback".into()]),
                scopes: Some(vec![Scope::Known(KnownScope::Atproto)]),
            },
            keys: None,
            state_store: DbStateStore::new(pool.clone(), db_backend.clone()),
            session_store: DbSessionStore::new(pool.clone(), db_backend.clone()),
            resolver: OAuthResolverConfig {
                did_resolver,
                handle_resolver,
                authorization_server_metadata: Default::default(),
                protected_resource_metadata: Default::default(),
            },
        })
        .expect("failed to create test OAuth client");

        let state = AppState {
            config,
            db: pool.clone(),
            http,
            dns,
            label_broadcast: label_tx,
            signer: Arc::new(signer),
            oauth: Arc::new(oauth_client),
            cookie_key,
            setup_labeler_did: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let router = debuff::server::router(state.clone());
        let admin_cookie = build_signed_cookie(&state.cookie_key, ADMIN_DID);

        TestApp {
            router,
            state,
            pool,
            mock_server,
            admin_cookie,
            _temp_dir: temp_dir,
        }
    }

    // -----------------------------------------------------------------------
    // HTTP helpers
    // -----------------------------------------------------------------------

    pub async fn get(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        send_request(&self.router, req).await
    }

    pub async fn get_authed(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("cookie", &self.admin_cookie)
            .body(Body::empty())
            .unwrap();
        send_request(&self.router, req).await
    }

    pub async fn post_authed(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("cookie", &self.admin_cookie)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();
        send_request(&self.router, req).await
    }

    pub async fn patch_authed(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("PATCH")
            .uri(uri)
            .header("cookie", &self.admin_cookie)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();
        send_request(&self.router, req).await
    }

    pub async fn delete_authed(&self, uri: &str) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("cookie", &self.admin_cookie)
            .body(Body::empty())
            .unwrap();
        send_request(&self.router, req).await
    }

    pub async fn delete_authed_with_body(&self, uri: &str, body: &Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("cookie", &self.admin_cookie)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap();
        send_request(&self.router, req).await
    }

    // -----------------------------------------------------------------------
    // Cookie helpers
    // -----------------------------------------------------------------------

    /// Build a signed session cookie for an arbitrary DID.
    pub fn cookie_for(&self, did: &str) -> String {
        build_signed_cookie(&self.state.cookie_key, did)
    }

    // -----------------------------------------------------------------------
    // Seed helpers
    // -----------------------------------------------------------------------

    /// Seed a moderator with the given DID and role.
    pub async fn seed_moderator(&self, did: &str, role: &str) {
        let backend = self.state.config.database.backend.clone();
        sqlx::query(&adapt_sql(
            "INSERT INTO moderators (did, role) VALUES ($1, $2) \
             ON CONFLICT (did) DO UPDATE SET role = EXCLUDED.role",
            backend,
        ))
        .bind(did)
        .bind(role)
        .execute(&self.pool)
        .await
        .expect("failed to seed moderator");
    }

    /// Seed a custom label definition and return its database id.
    pub async fn seed_definition(&self, identifier: &str) -> i64 {
        let backend = self.state.config.database.backend.clone();
        sqlx::query(&adapt_sql(
            "INSERT INTO label_definitions (identifier, severity, blurs, default_setting, adult_only) \
             VALUES ($1, 'inform', 'none', 'warn', 0) \
             ON CONFLICT (identifier) DO NOTHING",
            backend.clone(),
        ))
        .bind(identifier)
        .execute(&self.pool)
        .await
        .expect("failed to seed definition");

        let row: (i64,) = sqlx::query_as(&adapt_sql(
            "SELECT id FROM label_definitions WHERE identifier = $1",
            backend,
        ))
        .bind(identifier)
        .fetch_one(&self.pool)
        .await
        .expect("failed to fetch seeded definition id");
        row.0
    }

    /// Seed a report and return its database id.
    pub async fn seed_report(&self, uri: &str, did: &str, status: &str) -> i64 {
        let backend = self.state.config.database.backend.clone();
        sqlx::query(&adapt_sql(
            "INSERT INTO reports (subject_uri, subject_did, reported_by, reason_type, reason, status) \
             VALUES ($1, $2, 'did:plc:reporter', 'com.atproto.moderation.defs#reasonSpam', 'test', $3)",
            backend.clone(),
        ))
        .bind(uri)
        .bind(did)
        .bind(status)
        .execute(&self.pool)
        .await
        .expect("failed to seed report");

        let row: (i64,) = sqlx::query_as(&adapt_sql(
            "SELECT id FROM reports WHERE subject_uri = $1 AND subject_did = $2 ORDER BY id DESC LIMIT 1",
            backend,
        ))
        .bind(uri)
        .bind(did)
        .fetch_one(&self.pool)
        .await
        .expect("failed to fetch seeded report id");
        row.0
    }
}

// ---------------------------------------------------------------------------
// Signed cookie builder
// ---------------------------------------------------------------------------

fn build_signed_cookie(_axum_key: &axum_extra::extract::cookie::Key, did: &str) -> String {
    // axum_extra::extract::cookie::Key wraps cookie::Key internally.
    // We derive an identical cookie::Key from the same secret to produce a
    // compatible HMAC signature.
    //
    // The axum_extra Key::derive_from uses the cookie crate's Key::derive_from
    // under the hood, so we replicate by using the same session_secret bytes.
    // However, axum_extra::Key doesn't expose the inner bytes, so we use a
    // different approach: build the cookie via the cookie crate directly using
    // the master key bytes that were used at construction time.
    //
    // We work around this by reconstructing from the same secret.
    // Since TestApp always uses a known secret, we hard-code it here.
    let secret = b"test-secret-at-least-64-bytes-long-for-key-derivation-to-work-ok";
    let key = Key::derive_from(secret);
    let mut jar = CookieJar::new();
    {
        let mut signed = jar.signed_mut(&key);
        let mut c = cookie::Cookie::new(COOKIE_NAME, did.to_string());
        c.set_path("/");
        c.set_http_only(true);
        signed.add(c);
    }
    // Extract the cookie header value
    jar.get(COOKIE_NAME)
        .map(|c| c.to_string())
        .expect("signed cookie was not added to jar")
}

// ---------------------------------------------------------------------------
// Request sender (pub for raw request access in tests)
// ---------------------------------------------------------------------------

pub async fn send_request(router: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = router.clone().oneshot(req).await.expect("request failed");

    let status = response.status();
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .expect("failed to read response body")
        .to_bytes();

    let value = if body_bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body_bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body_bytes).into_owned()))
    };

    (status, value)
}
