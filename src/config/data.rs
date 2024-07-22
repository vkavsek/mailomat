use std::{
    collections::{hash_map::Entry, HashMap},
    io::Read,
};

use lazy_regex::regex_captures;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use strum_macros::AsRefStr;
use toml::Value;

// ###################################
// ->   STRUCTS
// ###################################
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

// ###################################
// ->   IMPLs
// ###################################
impl AppConfig {
    pub fn init() -> AppConfigBuilder {
        AppConfigBuilder::default()
    }
}

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

        let file_len = file
            .metadata()
            .map(|data| data.len())
            .unwrap_or_else(|e| panic!("Fatal Error: Building config: {e}"));

        let read_len = file
            .read_to_string(&mut file_content)
            .unwrap_or_else(|e| panic!("Fatal Error: Building config: {e}"));

        assert_eq!(file_len, read_len as u64);

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
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(self.require_ssl.into())
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
// ->   RESULT & ERROR
// ###################################
use derive_more::From;

use crate::web::data::ValidEmail;

pub type ConfigResult<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, From)]
pub enum ConfigError {
    StringToEnvironmentFail,
    StringToDbConfigFail,
    InvalidEmail(String),

    Io(std::io::Error),
    TomlDeser(toml::de::Error),
    TomlSer(toml::ser::Error),
}
// Error Boilerplate
impl core::fmt::Display for ConfigError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
impl std::error::Error for ConfigError {}

// ###################################
// ->   TESTS
// ###################################

#[cfg(test)]
mod tests {
    use std::{fs::File, str::FromStr};

    use super::*;

    #[test]
    fn test_app_config_add_source_and_build_ok() -> ConfigResult<()> {
        let base_path = std::env::current_dir().expect("Failed to determine the current DIR.");
        let config_dir = base_path.join("config");
        let base_file = File::open(config_dir.join("base.toml"))?;
        let local_file = File::open(config_dir.join("local.toml"))?;
        let email_config = EmailConfig {
            sender_addr: "admin@majkavsek.com".to_string(),
            url: "https://api.postmarkapp.com".to_string(),
            auth_token: "dev_token".to_string().into(),
            timeout_millis: 10000,
        };

        let test_app_config = AppConfig {
            net_config: NetConfig {
                host: [127, 0, 0, 1],
                app_port: 8080,
            },
            db_config: DbConfig {
                username: "postgres".to_string(),
                password: SecretString::from_str("password").unwrap(),
                port: 5432,
                host: "127.0.0.1".to_string(),
                db_name: "newsletter".to_string(),
                require_ssl: SslRequire::Disable,
            },
            email_config,
        };

        let app_config = AppConfig::init()
            .add_source_file(base_file)
            .add_source_file(local_file)
            .build()?;

        assert_eq!(test_app_config.net_config, app_config.net_config);
        assert_eq!(
            test_app_config.db_config.username,
            app_config.db_config.username
        );
        assert_eq!(
            test_app_config.db_config.password.expose_secret(),
            app_config.db_config.password.expose_secret()
        );
        assert_eq!(test_app_config.db_config.port, app_config.db_config.port);
        assert_eq!(test_app_config.db_config.host, app_config.db_config.host);
        assert_eq!(
            test_app_config.db_config.db_name,
            app_config.db_config.db_name
        );
        assert_eq!(
            test_app_config.email_config.sender_addr,
            app_config.email_config.sender_addr
        );
        assert_eq!(
            test_app_config.email_config.url,
            app_config.email_config.url
        );
        assert_eq!(
            test_app_config.email_config.auth_token.expose_secret(),
            app_config.email_config.auth_token.expose_secret()
        );
        assert_eq!(
            test_app_config.email_config.timeout_millis,
            app_config.email_config.timeout_millis
        );

        Ok(())
    }

    #[test]
    fn test_db_config_from_str_ok() -> ConfigResult<()> {
        {
            let db_url = "postgres://my_uname:pwd@localhost:6666/my_db?sslmode=disable";
            let db_config = DbConfig::try_from(db_url)?;

            assert_eq!("my_uname", db_config.username);
            assert_eq!("pwd", db_config.password.expose_secret());
            assert_eq!("localhost", db_config.host);
            assert_eq!(6666, db_config.port);
            assert_eq!("my_db", db_config.db_name);
            assert_eq!(SslRequire::Disable, db_config.require_ssl);
        }

        {
            let db_url = "postgres://my_uname:pwd@localhost:6666/my_db?sslmode=require";
            let db_config = DbConfig::try_from(db_url)?;

            assert_eq!("my_uname", db_config.username);
            assert_eq!("pwd", db_config.password.expose_secret());
            assert_eq!("localhost", db_config.host);
            assert_eq!(6666, db_config.port);
            assert_eq!("my_db", db_config.db_name);
            assert_eq!(SslRequire::Require, db_config.require_ssl);
        }

        {
            let db_url = "postgres://my_uname:pwd@localhost:6666/my_db";
            let db_config = DbConfig::try_from(db_url)?;

            assert_eq!("my_uname", db_config.username);
            assert_eq!("pwd", db_config.password.expose_secret());
            assert_eq!("localhost", db_config.host);
            assert_eq!(6666, db_config.port);
            assert_eq!("my_db", db_config.db_name);
            assert_eq!(SslRequire::Prefer, db_config.require_ssl);
        }

        Ok(())
    }

    #[test]
    fn test_db_config_from_str_fail() {
        {
            let db_url = "postgres://my_uname:pwd@localh";
            let db_config = DbConfig::try_from(db_url);
            assert!(db_config.is_err())
        }

        {
            let db_url = "postgres://my_uname:pwd@localhost:asd/my_db";
            let db_config = DbConfig::try_from(db_url);
            assert!(db_config.is_err())
        }

        {
            let db_url = "postgres://my_uname:pwd@localhost:asd/my_db/fail";
            let db_config = DbConfig::try_from(db_url);
            assert!(db_config.is_err())
        }
    }
}
