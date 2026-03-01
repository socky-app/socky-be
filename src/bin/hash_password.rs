use std::io::Write;

use anyhow::{Context, Result};
use rpassword::read_password;
use secrecy::ExposeSecret;
use socky_be::{
    config::{load_config, Config},
    utils::password::PasswordUtils,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Load env
    dotenvy::dotenv().ok();

    // Initialize logger
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Prompt for password (Secure)
    print!("Enter password: ");
    std::io::stdout()
        .flush()
        .context("Failed to flush stdout")?;
    let password = read_password().context("Failed to read password securely")?;
    let password = password.trim();

    // Load config
    let Config {
        network: _,
        router: _,
        auth: auth_config,
        db: _,
    } = load_config().context("Failed to configuration from env variables")?;

    // Hash password
    info!("Hashing password");
    let password_hash =
        PasswordUtils::hash_password(password, auth_config.password_pepper.expose_secret())
            .context("Failed to hash the password using the configured algorithm and pepper")?;

    println!("Password hash: {}", password_hash);

    Ok(())
}