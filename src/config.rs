//! Tries to create a `Config` from a config file:

use std::sync::OnceLock;

use lazy_regex::regex_captures;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use strum_macros::AsRefStr;
use tracing::debug;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub net_config: NetConfig,
    pub db_config: DbConfig,
}

#[derive(Deserialize, Clone)]
pub struct NetConfig {
    pub host: [u8; 4],
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub app_port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DbConfig {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub db_name: String,
}
impl DbConfig {
    pub fn connection_options(&self) -> PgConnectOptions {
        self.connection_options_without_db().database(&self.db_name)
    }
    pub fn connection_options_without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
    }
}

impl TryFrom<String> for DbConfig {
    type Error = crate::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // postgres://{username}:{password}@{hostname}:{port}/{database}
        let (_whole, username, password, host, port, db_name) = regex_captures!(
            r#"^postgres:\/\/(^:)+:(^@)+@(^:\/)+:(\d+)\/(^\s\/)+$"#,
            &value
        )
        .ok_or(crate::Error::StringToDbConfigFail)?;

        let (username, db_name, host) =
            (username.to_string(), db_name.to_string(), host.to_string());
        let password = SecretString::new(password.to_string());
        let port = port
            .parse()
            .map_err(|_| crate::Error::StringToDbConfigFail)?;

        Ok(DbConfig {
            username,
            password,
            port,
            host,
            db_name,
        })
    }
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
            _ => Err(Self::Error::StringToEnvironmentFail),
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

        let mut config = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base.toml")))
            .add_source(config::File::from(config_dir.join(environment_filename)))
            // TODO: Delete this ?
            // Injects in settings from environment.
            // Only captures variables that start with prefix `APP_`,
            // the values are separated with a `-`.
            // APP_NETCONFIG__PORT, would set NetConfig.port
            // .add_source(
            //     config::Environment::with_prefix("APP")
            //         .prefix_separator("_")
            //         .separator("__"),
            // )
            .build()
            .unwrap_or_else(|er| panic!("Fatal Error: While trying to build AppConfig: {er:?}"))
            .try_deserialize::<AppConfig>()
            .unwrap_or_else(|er| {
                panic!("Fatal Error: While deserializing Config to AppConfig: {er:?}")
            });

        // Setup DbConfig for production
        if matches!(environment, Environment::Production) {
            // Panic early if there are any problems.
            let production_db = std::env::var("DATABASE_URL").unwrap_or_else(|er| {
                panic!("Fatal Error: While looking for DATABASE_URL env variable: {er:?}")
            });
            let prod_db_config = DbConfig::try_from(production_db).unwrap_or_else(|er| {
                panic!("Fatal Error: While parsing DbConfig from String: {er:?}")
            });
            config.db_config = prod_db_config;
        }

        config
    })
}
