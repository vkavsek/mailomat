//! Tries to create an `AppConfig` from config files.
//! Currently uses `AppConfigBuilder` to build up configuration from multiple files.
//! Gets initialized with `OnceLock` so it only needs to get initialized once.

mod data;

use std::sync::OnceLock;
use tracing::info;

use data::Environment;

// Re-export config structs
pub use data::{AppConfig, ConfigError, DbConfig, EmailConfig, NetConfig};

/// Allocates a static `OnceLock` containing `AppConfig`.
/// This ensures configuration only gets initialized the first time we call this function.
/// Every other caller gets a &'static ref to AppConfig.
/// Panics if anything goes wrong.
pub fn get_or_init_config() -> &'static AppConfig {
    static CONFIG_INIT: OnceLock<AppConfig> = OnceLock::new();
    CONFIG_INIT.get_or_init(|| {
        info!(
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

        let base_file = std::fs::File::open(config_dir.join("base.toml"))
            .unwrap_or_else(|er| panic!("Fatal Error: Building config: {er}"));
        let env_file = std::fs::File::open(config_dir.join(environment_filename))
            .unwrap_or_else(|er| panic!("Fatal Error: Building config: {er}"));

        let mut config = AppConfig::init()
            .add_source_file(base_file)
            .add_source_file(env_file)
            .build()
            .unwrap_or_else(|er| panic!("Fatal Error: Building config: {er}"));

        // Setup DbConfig for production
        if matches!(environment, Environment::Production) {
            // Panic early if there are any problems.
            let production_db = std::env::var("DATABASE_URL").unwrap_or_else(|er| {
                panic!("Fatal Error: While looking for DATABASE_URL env variable: {er:?}")
            });
            let prod_db_config = DbConfig::try_from(production_db.as_str()).unwrap_or_else(|er| {
                panic!("Fatal Error: While parsing DbConfig from String: {er:?}")
            });
            config.db_config = prod_db_config;
        }

        config
    })
}
