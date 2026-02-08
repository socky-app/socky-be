use figment::providers::{Env, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};

use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        let config: Config = Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Env::prefixed("SOCKY_"))
            .extract()
            .expect("Failed to load configuration");

        tracing::info!("CONFIG: {:?}", config);
        config
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// application port
    pub app_port: u16,
    /// application host
    pub app_host: String,

    /// database URL
    pub db_url: String,
    /// database maximum connection count
    pub db_max_conn: u32,
    /// database minimum connection count
    pub db_min_conn: u32,
    /// database connection timeout
    pub db_conn_timeout: u64,
    /// database idle connection timeout
    pub db_idle_timeout: u64,
    /// JWT secret key

    /// web folder path
    pub web_folder: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_port: 8000,
            app_host: "0.0.0.0".into(),
            db_url: "sqlite://socky.db".into(),
            db_max_conn: 10,
            db_min_conn: 1,
            db_conn_timeout: 10,
            db_idle_timeout: 0,
            web_folder: "web-folder".into(),
        }
    }
}
