use figment::providers::{Env, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};

use std::sync::OnceLock;

const DEFAULT_APP_PORT: u16 = 8000;
const DEFAULT_APP_HOST: &str = "0.0.0.0";
const DEFAULT_DB_MAX_CONN: u32 = 10;
const DEFAULT_DB_MIN_CONN: u32 = 1;
const DEFAULT_DB_CONN_TIMEOUT_S: u64 = 10;
const DEFAULT_DB_IDLE_TIMEOUT_S: u64 = 0;
const DEFAULT_ACCESS_TOKEN_EXPIRATION_S: i64 = 60 * 15; // 15 min
const DEFAULT_REFRESH_TOKEN_EXPIRATION_S: i64 = 60 * 60 * 24 * 15; // 15 days
const DEFULAT_WEB_FOLDER: &str = "web-folder";

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Figment::new()
            // --- LAYER 1: Non-sensitive Defaults ---
            .merge(Serialized::default("app_port", DEFAULT_APP_PORT))
            .merge(Serialized::default("app_host", DEFAULT_APP_HOST))
            .merge(Serialized::default("db_max_conn", DEFAULT_DB_MAX_CONN))
            .merge(Serialized::default("db_min_conn", DEFAULT_DB_MIN_CONN))
            .merge(Serialized::default("db_conn_timeout_seconds", DEFAULT_DB_CONN_TIMEOUT_S))
            .merge(Serialized::default("db_idle_timeout_seconds", DEFAULT_DB_IDLE_TIMEOUT_S))
            .merge(Serialized::default("access_token_expiration_seconds", DEFAULT_ACCESS_TOKEN_EXPIRATION_S))
            .merge(Serialized::default("refresh_token_expiration_seconds", DEFAULT_REFRESH_TOKEN_EXPIRATION_S))
            .merge(Serialized::default("web_folder", DEFULAT_WEB_FOLDER))

            // --- LAYER 2: Environment Variables ---
            .merge(Env::prefixed("SOCKY_"))

            // --- FINAL STEP: Validation ---
            .extract::<Config>().unwrap_or_else(|error| {
                eprintln!("====================================================");
                eprintln!("❌ CONFIGURATION ERROR");
                eprintln!("====================================================");
                
                for e in error {
                    eprintln!("Issue: {}", e);
                }

                eprintln!("----------------------------------------------------");
                eprintln!("💡 TROUBLESHOOTING:");
                eprintln!("1. If developing, make sure you created you own .env file using .env.example as template");
                eprintln!("2. Ensure required Environment Variables are set:");
                eprintln!("   - SOCKY_DB_URL");
                eprintln!("   - SOCKY_ACCESS_TOKEN_SECRET");
                eprintln!("   - SOCKY_REFRESH_TOKEN_SECRET");
                eprintln!("   - SOCKY_REFRESH_TOKEN_PEPPER");
                eprintln!("   - SOCKY_PASSWORD_PEPPER");
                eprintln!("====================================================");
                
                std::process::exit(1);
            })
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Application port
    pub app_port: u16,
    /// Application host
    pub app_host: String,

    /// Database URL
    pub db_url: String,
    /// Database maximum connection count
    pub db_max_conn: u32,
    /// Database minimum connection count
    pub db_min_conn: u32,
    /// Database connection timeout
    pub db_conn_timeout_seconds: u64,
    /// Database idle connection timeout
    pub db_idle_timeout_seconds: u64,

    /// Pepper for password hashing
    pub password_pepper: String,

    /// Secret for access JWT signing
    pub access_token_secret: String,
    /// Expiration for access JWT
    pub access_token_expiration_seconds: i64,
    /// Secret for refresh JWT signing
    pub refresh_token_secret: String,
    /// Expiration for refresh JWT
    pub refresh_token_expiration_seconds: i64,
    /// Pepper for refresh JWT hashing
    pub refresh_token_pepper: String,

    /// Web folder path
    pub web_folder: String,
}
