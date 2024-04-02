//! Tries to create a `Config` from a config file:

use std::sync::OnceLock;

use secrecy::{ExposeSecret, Secret, SecretString};
use serde::Deserialize;
use strum_macros::AsRefStr;
use tracing::debug;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub net_config: NetConfig,
    pub db_config: DatabaseConfig,
}

#[derive(Deserialize, Clone)]
pub struct NetConfig {
    pub host: [u8; 4],
    pub app_port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

#[derive(AsRefStr)]
enum Environment {
    Local,
    Production,
}

impl TryFrom<String> for Environment {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(Self::Error::StrToEnvironmentFail),
        }
    }
}

/// Allocates a static `OnceLock` containing `AppConfig`.
/// This ensures configuration only gets initialized the first time we call this function.
/// Every other caller gets a &'static ref to AppConfig.
pub fn get_or_init_config() -> &'static AppConfig {
    static CONFIG_INIT: OnceLock<AppConfig> = OnceLock::new();
    CONFIG_INIT.get_or_init(|| {
        debug!(
            "{:<12} - Initializing the configuration",
            "get_or_init_config"
        );
        let base_path = std::env::current_dir().expect("Failed to determine the current DIR.");
        let config_dir = base_path.join("config");

        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");
        let environment_filename = format!("{}.toml", environment.as_ref().to_lowercase());

        config::Config::builder()
            .add_source(config::File::from(config_dir.join("base.toml")))
            .add_source(config::File::from(config_dir.join(environment_filename)))
            .build()
            .unwrap_or_else(|er| panic!("Fatal Error: While trying to build AppConfig: {er:?}"))
            .try_deserialize::<AppConfig>()
            .unwrap_or_else(|er| {
                panic!("Fatal Error: While deserializing Config to AppConfig: {er:?}")
            })
    })
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> SecretString {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.db_name
        ))
    }
    pub fn connection_string_without_db(&self) -> SecretString {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
