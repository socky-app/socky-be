use std::fmt;

use figment::providers::{Env, Serialized};
use figment::Figment;
use secrecy::SecretString;
use serde::Deserialize;
use thiserror::Error;
use tracing::error;

const DEFAULT_NETWORK_PORT: u16 = 8000;
const DEFAULT_NETWORK_HOST: &str = "0.0.0.0";
const DEFAULT_DB_MAX_CONN: u32 = 10;
const DEFAULT_DB_MIN_CONN: u32 = 1;
const DEFAULT_DB_CONN_TIMEOUT_S: u64 = 10;
const DEFAULT_DB_IDLE_TIMEOUT_S: u64 = 0;
const DEFAULT_ACCESS_TOKEN_EXPIRATION_S: i64 = 60 * 15; // 15 min
const DEFAULT_REFRESH_TOKEN_EXPIRATION_S: i64 = 60 * 60 * 24 * 15; // 15 days
const DEFULAT_WEB_FOLDER: &str = "web-folder";

#[derive(Debug, Error)]
pub struct ConfigError {
    pub details: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub router: RouterConfig,
    pub db: DbConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub port: u16,
    pub host: String
}

#[derive(Debug, Deserialize)]
pub struct RouterConfig {
    pub web_folder: String,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub url: String,
    pub max_conn: u32,
    pub min_conn: u32,
    pub conn_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
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
        .extract::<Config>()?
    )
}

impl From<figment::Error> for ConfigError {
    fn from(value: figment::Error) -> Self {
        let details = value
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>();

            ConfigError { details }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n====================================================")?;
        writeln!(f, "❌ CONFIGURATION ERROR")?;
        writeln!(f, "====================================================")?;

        for (i, msg) in self.details.iter().enumerate() {
            writeln!(f, "{}. Issue: {}", i + 1, msg)?;
        }

        writeln!(f, "----------------------------------------------------")?;
        writeln!(f, "💡 TROUBLESHOOTING:")?;
        writeln!(f, "Ensure these Environment Variables are set (double __ for nesting):")?;
        writeln!(f, "   - SOCKY_DB__URL")?;
        writeln!(f, "   - SOCKY_AUTH__PASSWORD_PEPPER")?;
        writeln!(f, "   - SOCKY_AUTH__ACCESS_TOKEN_SECRET")?;
        writeln!(f, "   - SOCKY_AUTH__REFRESH_TOKEN_SECRET")?;
        writeln!(f, "   - SOCKY_AUTH__REFRESH_TOKEN_PEPPER")?;
        writeln!(f, "====================================================")
    }
}
