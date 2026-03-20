pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod dns;
pub mod error;
pub mod server;
pub mod signing;
pub mod xrpc;

use auth::oauth_store::{DbSessionStore, DbStateStore};
use config::Config;
use dns::NativeDnsResolver;
use reqwest::Client;
use signing::LabelSigner;
use sqlx::AnyPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

use atrium_identity::did::CommonDidResolver;
use atrium_identity::handle::AtprotoHandleResolver;
use atrium_oauth::DefaultHttpClient;

pub type DebuffOAuthClient = atrium_oauth::OAuthClient<
    DbStateStore,
    DbSessionStore,
    CommonDidResolver<DefaultHttpClient>,
    AtprotoHandleResolver<NativeDnsResolver, DefaultHttpClient>,
>;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: AnyPool,
    pub http: Client,
    pub dns: NativeDnsResolver,
    pub label_broadcast: broadcast::Sender<i64>,
    pub signer: Arc<RwLock<LabelSigner>>,
    pub oauth: Arc<DebuffOAuthClient>,
    pub cookie_key: axum_extra::extract::cookie::Key,
    pub setup_labeler_did: Arc<Mutex<Option<String>>>,
}

impl AppState {
    /// Resolve the labeler DID from in-memory mutex, config, or database.
    ///
    /// Returns `None` if no valid DID has been configured yet.
    pub async fn labeler_did(&self) -> Option<String> {
        // 1. In-memory mutex (set during current process's setup flow)
        let mutex_did = self.setup_labeler_did.lock().await.clone();
        let did = mutex_did
            .as_deref()
            .unwrap_or(self.config.labeler.did.as_str())
            .to_string();

        // 2. If still placeholder, check the database (covers Railway/Docker after restart)
        let did = if did.is_empty() || did == "did:plc:placeholder" {
            crate::db::settings::get(
                &self.db,
                self.config.database.backend.clone(),
                "labeler.did",
            )
            .await
            .ok()
            .flatten()
            .unwrap_or(did)
        } else {
            did
        };

        if did.is_empty() || did == "did:plc:placeholder" {
            None
        } else {
            Some(did)
        }
    }
}

impl axum::extract::FromRef<AppState> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
