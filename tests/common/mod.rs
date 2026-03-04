use reqwest::Client;

use secrecy::ExposeSecret;
use socky_be::{
    config::{AppConfig, AuthConfig},
    create_app,
    model::user::{dto::CreateUserDto, UserRole, UserStatus},
    repository::{user_repo::UserRepository, Create, RepositoryManager},
    utils::password::PasswordUtils,
};

pub struct TestApp {
    pub base_url: String,
    pub rm: RepositoryManager,
    pub app_config: AppConfig,
    pub client: Client,
}

impl TestApp {
    pub async fn new(pool: sqlx::PgPool) -> Self {
        let app_config = AppConfig {
            router: socky_be::config::RouterConfig {
                web_folder: "web-folder".to_string(),
            },
            auth: AuthConfig {
                password_pepper: secrecy::SecretString::new("test_pepper".to_string().into()),
                access_token_secret: secrecy::SecretString::new(
                    "test_access_secret".to_string().into(),
                ),
                access_token_expiration_seconds: 900, // 15 min
                refresh_token_secret: secrecy::SecretString::new(
                    "test_refresh_secret".to_string().into(),
                ),
                refresh_token_expiration_seconds: 86400 * 15, // 15 days
                refresh_token_pepper: secrecy::SecretString::new(
                    "test_refresh_pepper".to_string().into(),
                ),
            },
        };

        Self::new_with_config(pool, app_config).await
    }

    pub async fn new_with_config(pool: sqlx::PgPool, app_config: AppConfig) -> Self {
        // Initialize DB from pool
        let rm = RepositoryManager::from_pool(pool);

        // Start app on random port to avoid conflicts
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);

        // Spawn server
        let app = create_app(rm.clone(), app_config.clone());

        // We use tokio::spawn to run it in the background
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            base_url,
            rm,
            app_config,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        }
    }

    /// Helper to create a user in the database directly.
    /// Returns the ID of the created user.
    pub async fn create_user(&self, email: &str, password: &str) -> i64 {
        let pepper = self.app_config.auth.password_pepper.clone();
        let pwd = password.to_string();

        let password_hash = tokio::task::spawn_blocking(move || {
            PasswordUtils::hash_password(&pwd, pepper.expose_secret())
        })
        .await
        .unwrap()
        .expect("Failed to hash password");

        let dto = CreateUserDto {
            email: email.to_string(),
            password: password_hash,
            role: UserRole::Standard,
            status: UserStatus::Active,
        };

        UserRepository::create(&self.rm, &dto)
            .await
            .expect("Failed to insert user")
    }
}
