use figment::providers::{Env, Serialized};
use figment::Figment;
use secrecy::SecretString;
use serde::Deserialize;

use std::sync::Arc;

const DEFAULT_APP_PORT: u16 = 8000;
const DEFAULT_APP_HOST: &str = "0.0.0.0";
const DEFAULT_DB_MAX_CONN: u32 = 10;
const DEFAULT_DB_MIN_CONN: u32 = 1;
const DEFAULT_DB_CONN_TIMEOUT_S: u64 = 10;
const DEFAULT_DB_IDLE_TIMEOUT_S: u64 = 0;
const DEFAULT_ACCESS_TOKEN_EXPIRATION_S: i64 = 60 * 15; // 15 min
const DEFAULT_REFRESH_TOKEN_EXPIRATION_S: i64 = 60 * 60 * 24 * 15; // 15 days
const DEFULAT_WEB_FOLDER: &str = "web-folder";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub db: DbConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
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

pub fn load_config() -> Config {
    Figment::new()
        // --- App Defaults ---
        .merge(Serialized::default("app.port", DEFAULT_APP_PORT))
        .merge(Serialized::default("app.host", DEFAULT_APP_HOST))
        .merge(Serialized::default("app.web_folder", DEFULAT_WEB_FOLDER))
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
        .extract::<Config>()
        .unwrap_or_else(|error| {
            print_config_error(error);
            std::process::exit(1);
        })
}

fn print_config_error(error: figment::Error) {
    eprintln!("====================================================");
    eprintln!("❌ CONFIGURATION ERROR");
    eprintln!("====================================================");

    for (i, e) in error.into_iter().enumerate() {
        eprintln!("{}. Issue: {}", i + 1, e);
    }

    eprintln!("----------------------------------------------------");
    eprintln!("💡 TROUBLESHOOTING:");
    eprintln!("Ensure these Environment Variables are set (note the double __ for nesting):");
    eprintln!("   - SOCKY_DB__URL");
    eprintln!("   - SOCKY_AUTH__PASSWORD_PEPPER");
    eprintln!("   - SOCKY_AUTH__ACCESS_TOKEN_SECRET");
    eprintln!("   - SOCKY_AUTH__REFRESH_TOKEN_SECRET");
    eprintln!("   - SOCKY_AUTH__REFRESH_TOKEN_PEPPER");
    eprintln!("====================================================");
}
