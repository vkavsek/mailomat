//! Creates a `&'static AppConfig` from config files or panics early!
//! Gets initialized with `OnceLock` so it only needs to get initialized once.
//! Currently uses `figment` to build up configuration from multiple files and env variables.

mod error;
mod types;

use core::panic;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::sync::OnceLock;
use tracing::info;

use types::Environment;

// Re-export config structs
pub use error::{ConfigError, ConfigResult};
pub use types::{AppConfig, DbConfig, EmailConfig, NetConfig};

/// Allocates a static `OnceLock` containing `AppConfig`.
/// This ensures configuration only gets initialized the first time we call this function.
/// Every other caller gets a &'static ref to AppConfig.
/// Panics if anything goes wrong.
pub fn get_or_init_config() -> &'static AppConfig {
    static CONFIG_INIT: OnceLock<AppConfig> = OnceLock::new();
    CONFIG_INIT.get_or_init(|| {
        info!(
            "{:<20} - Initializing the configuration",
            "get_or_init_config"
        );
        let base_path = std::env::current_dir().expect("Failed to determine the current DIR.");
        let config_dir = base_path.join("config");

        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");
        let environment_filename = format!("{}.toml", environment.as_ref().to_lowercase());

        let mut config: AppConfig = Figment::new()
            .merge(Toml::file(config_dir.join("base.toml")))
            .merge(Toml::file(config_dir.join(environment_filename)))
            .merge(Env::prefixed("CONFIG__").split("__"))
            .extract()
            .unwrap_or_else(|e| panic!("Unable to build AppConfig: {}", e.to_string()));

        if matches!(environment, Environment::Production) {
            // Setup DbConfig for production
            //
            // Panic early if there are any problems.
            // DATABASE_URL is a secret provided by Fly.io
            let production_db = std::env::var("DATABASE_URL").unwrap_or_else(|er| {
                panic!("Fatal Error: While looking for DATABASE_URL env variable: {er:?}")
            });
            let prod_db_config = DbConfig::try_from(production_db.as_str()).unwrap_or_else(|er| {
                panic!("Fatal Error: While parsing DbConfig from String: {er:?}")
            });
            config.db_config = prod_db_config;

            // TODO: redis on fly.io
        }

        config
    })
}
