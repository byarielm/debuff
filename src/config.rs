use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub labeler: LabelerConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_public_url")]
    pub public_url: String,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
    #[serde(default = "default_session_secret")]
    pub session_secret: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_backend")]
    pub backend: DatabaseBackend,
    #[serde(default = "default_database_url")]
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    Sqlite,
    Postgres,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LabelerConfig {
    #[serde(default = "default_labeler_did")]
    pub did: String,
    pub signing_key_path: Option<String>,
    #[serde(default = "default_plc_url")]
    pub plc_url: String,
}

// Default functions for serde
fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 3000 }
fn default_public_url() -> String { "http://127.0.0.1:3001".into() }
fn default_static_dir() -> String { "./web/out".into() }
fn default_session_secret() -> String { "change-me-in-production".into() }
fn default_backend() -> DatabaseBackend { DatabaseBackend::Sqlite }
fn default_database_url() -> String { "sqlite://data/debuff.db?mode=rwc".into() }
fn default_labeler_did() -> String { "did:plc:placeholder".into() }
fn default_plc_url() -> String { "https://plc.directory".into() }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            public_url: default_public_url(),
            static_dir: default_static_dir(),
            session_secret: default_session_secret(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            url: default_database_url(),
        }
    }
}

impl Default for LabelerConfig {
    fn default() -> Self {
        Self {
            did: default_labeler_did(),
            signing_key_path: None,
            plc_url: default_plc_url(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            labeler: LabelerConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = std::env::var("DEBUFF_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./config.toml"));

        if path.exists() {
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read config file {}: {}", path.display(), e));
            toml::from_str(&contents)
                .unwrap_or_else(|e| panic!("Failed to parse config file {}: {}", path.display(), e))
        } else {
            tracing::info!(
                "No config file found at {} — using defaults",
                path.display()
            );
            Config::default()
        }
    }
}
