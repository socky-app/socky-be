//! Configuration management module.
//!
//! Loads configurations for database connectivity, network bindings, and authentication secrets
//! using the Figment library. Re-maps configurations to specialized subsets for easier injection.

use figment::providers::{Env, Serialized};
use figment::Figment;
use secrecy::SecretString;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_NETWORK_PORT: u16 = 8000;
const DEFAULT_NETWORK_HOST: &str = "0.0.0.0";
const DEFAULT_DB_MAX_CONN: u32 = 10;
const DEFAULT_DB_MIN_CONN: u32 = 1;
const DEFAULT_DB_CONN_TIMEOUT_S: u64 = 10;
const DEFAULT_DB_IDLE_TIMEOUT_S: u64 = 0;
const DEFAULT_ACCESS_TOKEN_EXPIRATION_S: i64 = 60 * 15; // 15 min
const DEFAULT_REFRESH_TOKEN_EXPIRATION_S: i64 = 60 * 60 * 24 * 15; // 15 days
const DEFULAT_WEB_FOLDER: &str = "web-folder";

/// Configuration parsing error.
#[derive(Debug, Error)]
#[error("invalid configuration: {}", details.join("; "))]
pub struct ConfigError {
    /// List of configuration error details.
    pub details: Vec<String>,
}

impl From<figment::Error> for ConfigError {
    fn from(value: figment::Error) -> Self {
        let details = value.into_iter().map(|e| e.to_string()).collect::<Vec<_>>();

        ConfigError { details }
    }
}

/// The top-level combined application configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Network listener details (host/port).
    pub network: NetworkConfig,
    /// Static assets router details.
    pub router: RouterConfig,
    /// Database pool configurations.
    pub db: DbConfig,
    /// Cryptographic parameters for authentication.
    pub auth: AuthConfig,
}

impl From<Config> for (NetworkConfig, AppConfig, DbConfig) {
    fn from(value: Config) -> Self {
        (
            value.network,
            AppConfig {
                router: value.router,
                auth: value.auth,
            },
            value.db,
        )
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub router: RouterConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub port: u16,
    pub host: String,
}

impl NetworkConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouterConfig {
    pub web_folder: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub name: String,
    pub max_conn: u32,
    pub min_conn: u32,
    pub conn_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl DbConfig {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}/{}",
            self.user, self.password, self.host, self.name,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub password_pepper: SecretString,
    pub access_token_secret: SecretString,
    pub access_token_expiration_seconds: i64,
    pub refresh_token_secret: SecretString,
    pub refresh_token_expiration_seconds: i64,
    pub refresh_token_pepper: SecretString,
}

pub fn load_config() -> Result<Config, ConfigError> {
    Ok(Figment::new()
        // --- App Defaults ---
        .merge(Serialized::default("network.port", DEFAULT_NETWORK_PORT))
        .merge(Serialized::default("network.host", DEFAULT_NETWORK_HOST))
        .merge(Serialized::default("router.web_folder", DEFULAT_WEB_FOLDER))
        // --- Database Defaults ---
        .merge(Serialized::default("db.max_conn", DEFAULT_DB_MAX_CONN))
        .merge(Serialized::default("db.min_conn", DEFAULT_DB_MIN_CONN))
        .merge(Serialized::default(
            "db.conn_timeout_seconds",
            DEFAULT_DB_CONN_TIMEOUT_S,
        ))
        .merge(Serialized::default(
            "db.idle_timeout_seconds",
            DEFAULT_DB_IDLE_TIMEOUT_S,
        ))
        // --- Auth Defaults ---
        .merge(Serialized::default(
            "auth.access_token_expiration_seconds",
            DEFAULT_ACCESS_TOKEN_EXPIRATION_S,
        ))
        .merge(Serialized::default(
            "auth.refresh_token_expiration_seconds",
            DEFAULT_REFRESH_TOKEN_EXPIRATION_S,
        ))
        // --- Environment Variables (With Nesting Support) ---
        // split("__") tells Figment that SOCKY_DB__URL means db.url
        .merge(Env::prefixed("SOCKY_").split("__"))
        .extract::<Config>()?)
}
