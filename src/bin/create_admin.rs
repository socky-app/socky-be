use std::io::Write;

use anyhow::{Context, Result};
use rpassword::read_password;
use secrecy::ExposeSecret;
use socky_be::{
    config::{load_config, Config},
    model::user::{dto::CreateUserDto, UserRole, UserStatus},
    repository::{user_repo::UserRepository, Create, RepositoryManager},
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

    // Prompt for email
    print!("Enter admin email: ");
    std::io::stdout()
        .flush()
        .context("Failed to flush stdout")?;
    let mut email = String::new();
    std::io::stdin()
        .read_line(&mut email)
        .context("Failed to read email from stdin")?;
    let email = email.trim();

    // Prompt for password (Secure)
    print!("Enter admin password: ");
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
        db: db_config,
    } = load_config().context("Failed to configuration from env variables")?;

    // Initialize RepositoryManager
    info!("Connecting to the database");
    let rm = RepositoryManager::new(&db_config)
        .await
        .context("Failed to establish a connection to the database")?;

    // Hash password
    info!("Hashing password");
    let password_hash =
        PasswordUtils::hash_password(password, auth_config.password_pepper.expose_secret())
            .context("Failed to hash the password using the configured algorithm and pepper")?;

    // Insert user into respository
    info!("Inserting admin user into the database");
    let dto = CreateUserDto {
        email: email.to_string(),
        password: password_hash,
        role: UserRole::Admin,
        status: UserStatus::Active,
    };
    let admin_id = UserRepository::create(&rm, &dto)
        .await
        .context("Failed to insert the new admin user")?;

    info!(
        "✅ Successfully created admin user '{}' with ID: {}",
        email,
        admin_id
    );

    Ok(())
}
