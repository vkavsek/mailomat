//! Tries to create a `Config` from a config file:

use serde::Deserialize;

use crate::Result;

#[derive(Deserialize)]
pub struct AppConfig {
    pub db_config: DatabaseConfig,
    pub app_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

pub fn get_config() -> Result<AppConfig> {
    // Init config reader
    let app_conf = config::Config::builder()
        .add_source(config::File::new(
            "app_config.toml",
            config::FileFormat::Toml,
        ))
        .build()?
        .try_deserialize::<AppConfig>()?;

    Ok(app_conf)
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }
}
