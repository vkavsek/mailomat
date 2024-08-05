//! The configuration structs used to build the AppConfig, and their impls.
use std::{
    collections::{hash_map::Entry, HashMap},
    io::Read,
};

use lazy_regex::regex_captures;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
use strum_macros::AsRefStr;
use toml::Value;

use crate::config::{ConfigError, ConfigResult};
use crate::web::data::ValidEmail;

// ###################################
// ->   STRUCTS
// ###################################
/// Not currently used.
/// TODO: Remove
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfigBuilder(HashMap<String, HashMap<String, Value>>);

#[derive(AsRefStr)]
pub enum Environment {
    Local,
    Production,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub net_config: NetConfig,
    pub db_config: DbConfig,
    pub email_config: EmailConfig,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NetConfig {
    pub host: [u8; 4],
    pub app_port: u16,
    pub base_url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DbConfig {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub db_name: String,
    pub require_ssl: SslRequire,
}

#[derive(Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SslRequire {
    #[default]
    Prefer,
    Require,
    Disable,
}

#[derive(Deserialize, Clone, Debug)]
pub struct EmailConfig {
    pub sender_addr: String,
    pub url: String,
    pub auth_token: SecretString,
    pub timeout_millis: u64,
}
// ###################################
// ->   IMPLs
// ###################################
impl EmailConfig {
    pub fn valid_sender(&self) -> ConfigResult<ValidEmail> {
        let addr = ValidEmail::parse(self.sender_addr.clone())
            .map_err(|er| ConfigError::InvalidEmail(er.to_string()))?;
        Ok(addr)
    }
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_millis)
    }
}

impl AppConfig {
    pub fn init() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }
}

/// Not currently used.
/// TODO: Remove
impl AppConfigBuilder {
    /// Extends this `AppConfigBuilder` with the contents of `other` builder.
    fn extend_builder(&mut self, other: Self) {
        for (entry, entry_hm) in other.0 {
            if let Entry::Vacant(e) = self.0.entry(entry.clone()) {
                e.insert(entry_hm);
            } else {
                let target_hm = self.0.get_mut(&entry).expect("Checked above!");
                for (inner_entry, inner_value) in entry_hm {
                    target_hm.insert(inner_entry, inner_value);
                }
            }
        }
    }

    /// Panics if file reading or deserialization goes wrong.
    pub fn add_source_file(mut self, mut file: std::fs::File) -> Self {
        let mut file_content = String::new();

        if let Err(e) = file.read_to_string(&mut file_content) {
            panic!("Fatal Error: Building config: {e}");
        }

        let app_conf_builder: AppConfigBuilder = toml::from_str(&file_content)
            .unwrap_or_else(|e| panic!("Fatal Error: Building config: {e}"));

        self.extend_builder(app_conf_builder);

        self
    }

    pub fn build(self) -> ConfigResult<AppConfig> {
        let serialized = toml::to_string(&self)?;
        let app_config = toml::from_str(&serialized)?;
        Ok(app_config)
    }
}

impl DbConfig {
    pub fn connection_options(&self) -> PgConnectOptions {
        self.connection_options_without_db().database(&self.db_name)
    }
    pub fn connection_options_without_db(&self) -> PgConnectOptions {
        // Create new PgConnectOptions struct but don't try to use the '$HOME/.pgpass' file.
        PgConnectOptions::new_without_pgpass()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(self.require_ssl.into())
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}

impl From<SslRequire> for PgSslMode {
    fn from(value: SslRequire) -> Self {
        match value {
            SslRequire::Require => PgSslMode::Require,
            SslRequire::Disable => PgSslMode::Disable,
            SslRequire::Prefer => PgSslMode::Prefer,
        }
    }
}

// ###################################
// ->   TRY FROMs
// ###################################

impl TryFrom<String> for Environment {
    type Error = ConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            _ => Err(Self::Error::StringToEnvironmentFail),
        }
    }
}

impl TryFrom<&str> for DbConfig {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // postgres://{username}:{password}@{hostname}:{port}/{database}?{options}
        let (_whole, username, password, host, port, db_name, options) = regex_captures!(
            r#"^postgres:\/\/([^:]+):([^@]+)@([^:\/]+):(\d+)\/([^\s\/?]+)(\?[^\s]*)?$"#,
            value
        )
        .ok_or(Self::Error::StringToDbConfigFail)?;

        let (username, db_name, host) =
            (username.to_string(), db_name.to_string(), host.to_string());
        let password = SecretString::new(password.to_string());
        let port = port
            .parse()
            .map_err(|_| Self::Error::StringToDbConfigFail)?;

        let mut require_ssl = SslRequire::default();
        if let Some(options) = options.strip_prefix('?') {
            for option in options.split(',') {
                if let Some((id, val)) = option.split_once('=') {
                    if id == "sslmode" {
                        match val {
                            "disable" => require_ssl = SslRequire::Disable,
                            "require" => require_ssl = SslRequire::Require,
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(DbConfig {
            username,
            password,
            port,
            host,
            db_name,
            require_ssl,
        })
    }
}

// ###################################
// ->   TESTS
// ###################################

#[cfg(test)]
mod tests {
    use std::fs::File;

    use claims::assert_ok;

    use super::*;

    /// Not currently used.
    /// TODO: Remove
    #[test]
    fn app_config_add_source_and_build_ok() -> ConfigResult<()> {
        let base_path = std::env::current_dir().expect("Failed to determine the current DIR.");
        let config_dir = base_path.join("config");
        let base_file = File::open(config_dir.join("base.toml"))?;
        let local_file = File::open(config_dir.join("local.toml"))?;

        let test_app_config = AppConfig::init()
            .add_source_file(base_file)
            .add_source_file(local_file)
            .build();

        assert_ok!(test_app_config);

        Ok(())
    }

    #[test]
    fn db_config_from_str_ok() -> ConfigResult<()> {
        let cases = [
            (
                "postgres://my_uname:pwd@localhost:6666/my_db?sslmode=disable",
                "my_uname",
                "pwd",
                "localhost",
                6666,
                "my_db",
                SslRequire::Disable,
            ),
            (
                "postgres://my_uname:pwd@localhost:6666/my_db?sslmode=require",
                "my_uname",
                "pwd",
                "localhost",
                6666,
                "my_db",
                SslRequire::Require,
            ),
            (
                "postgres://my_uname:pwd@localhost:6666/my_db",
                "my_uname",
                "pwd",
                "localhost",
                6666,
                "my_db",
                SslRequire::Prefer,
            ),
        ];

        for (
            db_url,
            expected_username,
            expected_password,
            expected_host,
            expected_port,
            expected_db_name,
            expected_ssl,
        ) in cases
        {
            let db_config = DbConfig::try_from(db_url)?;
            assert_eq!(expected_username, db_config.username);
            assert_eq!(expected_password, db_config.password.expose_secret());
            assert_eq!(expected_host, db_config.host);
            assert_eq!(expected_port, db_config.port);
            assert_eq!(expected_db_name, db_config.db_name);
            assert_eq!(expected_ssl, db_config.require_ssl);
        }

        Ok(())
    }

    #[test]
    fn db_config_from_str_fail() {
        let invalid_urls = [
            "postgres://my_uname:pwd@localh",
            "postgres://my_uname:pwd@localhost:asd/my_db",
            "postgres://my_uname:pwd@localhost:asd/my_db/fail",
        ];

        for db_url in invalid_urls {
            let db_config = DbConfig::try_from(db_url);
            assert!(db_config.is_err());
        }
    }
}
