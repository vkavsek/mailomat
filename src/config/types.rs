//! The configuration structs used to build the AppConfig, and their impls.
use lazy_regex::regex_captures;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
use strum_macros::AsRefStr;

use crate::config::{ConfigError, ConfigResult};
use crate::web::types::ValidEmail;

// ###################################
// ->   STRUCTS
// ###################################
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
    pub session_config: SessionConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SessionConfig {
    pub secure: bool,
    pub expiry_secs: i64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NetConfig {
    pub host: [u8; 4],
    pub app_port: u16,
    pub redis_uri: SecretString,
    pub base_url: String,
    pub cookie_secret_b64enc: SecretString,
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
        let password = SecretString::from(password.to_string());
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
    use super::*;

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
