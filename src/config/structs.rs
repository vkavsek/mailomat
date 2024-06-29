use std::{
    collections::{hash_map::Entry, HashMap},
    io::Read,
};

use derive_more::From;
use lazy_regex::regex_captures;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgConnectOptions;
use strum_macros::AsRefStr;
use toml::Value;

// ###################################
// ->   RESULT & ERROR
// ###################################

pub type ConfigResult<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, From)]
pub enum ConfigError {
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
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfigBuilder(HashMap<String, HashMap<String, Value>>);

// ###################################
// ->   IMPLs
// ###################################
impl AppConfig {
    pub fn init() -> AppConfigBuilder {
        AppConfigBuilder::default()
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
    }
}

impl AppConfigBuilder {
    pub fn add_source(mut self, mut file: std::fs::File) -> ConfigResult<Self> {
        let mut file_content = String::new();

        let file_len = file.metadata().map(|data| data.len())?;
        let read_len = file.read_to_string(&mut file_content)?;
        assert_eq!(file_len, read_len as u64);

        let app_conf_builder: AppConfigBuilder = toml::from_str(&file_content)?;

        for (entry, entry_hm) in app_conf_builder.0 {
            if let Entry::Vacant(e) = self.0.entry(entry.clone()) {
                e.insert(entry_hm);
            } else {
                let target_hm = self.0.get_mut(&entry).expect("Checked above!");
                for (inner_entry, inner_value) in entry_hm {
                    target_hm.insert(inner_entry, inner_value);
                }
            }
        }

        Ok(self)
    }

    pub fn build(self) -> ConfigResult<AppConfig> {
        let serialized = toml::to_string(&self)?;
        let app_config: AppConfig = toml::from_str(&serialized)?;
        Ok(app_config)
    }
}

// ###################################
// ->   TRY FROMs
// ###################################

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

impl TryFrom<&str> for DbConfig {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // postgres://{username}:{password}@{hostname}:{port}/{database}
        let (_whole, username, password, host, port, db_name, _options) = regex_captures!(
            r#"^postgres:\/\/([^:]+):([^@]+)@([^:\/]+):(\d+)\/([^\s\/?]+)(\?[^\s]*)?$"#,
            value
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

// FIXME: DELETE
// impl TryFrom<AppConfigBuilder> for NetConfig {
//     type Error = ConfigError;
//     fn try_from(value: AppConfigBuilder) -> Result<Self, Self::Error> {
//         let net_config_hm = value
//             .0
//             .get("net_config")
//             .ok_or(ConfigError::NetConfigBuildFail)?;
//         let host = net_config_hm
//             .get("host")
//             .and_then(|val| val.as_array())
//             .and_then(|arr| {
//                 if arr.len() == 4 {
//                     let mut host_arr = [0u8; 4];
//                     for (i, val) in arr.iter().enumerate() {
//                         if let Some(val) = val.as_integer().and_then(|val| val.try_into().ok()) {
//                             host_arr[i] = val;
//                         } else {
//                             return None;
//                         }
//                     }
//                     Some(host_arr)
//                 } else {
//                     None
//                 }
//             })
//             .ok_or(ConfigError::NetConfigBuildFail)?;
//         let app_port = net_config_hm
//             .get("app_port")
//             .and_then(|val| val.as_integer())
//             .and_then(|val| val.try_into().ok())
//             .ok_or(ConfigError::NetConfigBuildFail)?;
//         Ok(NetConfig { host, app_port })
//     }
// }

// ###################################
// ->   TESTS
// ###################################

#[cfg(test)]
mod tests {
    use std::{fs::File, str::FromStr};

    use super::*;
    use crate::Result;

    // FIXME: DELETE
    // #[test]
    // fn test_try_from_app_config_builder_for_net_config_success(
    // ) -> core::result::Result<(), ConfigError> {
    //     let test_net_config = NetConfig {
    //         host: [127, 0, 0, 1],
    //         app_port: 8080,
    //     };
    //     let mut app_config_builder_hm = HashMap::new();
    //     app_config_builder_hm.insert(
    //         "net_config".to_string(),
    //         HashMap::from([
    //             (
    //                 "host".to_string(),
    //                 Value::Array(vec![
    //                     Value::Integer(127),
    //                     Value::Integer(0),
    //                     Value::Integer(0),
    //                     Value::Integer(1),
    //                 ]),
    //             ),
    //             ("app_port".to_string(), Value::Integer(8080)),
    //         ]),
    //     );
    //     let app_config_builder = AppConfigBuilder(app_config_builder_hm);
    //     let net_config = NetConfig::try_from(app_config_builder)?;
    //     assert_eq!(test_net_config, net_config);
    //     Ok(())
    // }

    #[test]
    fn test_app_config_add_source_and_succesful_build() -> ConfigResult<()> {
        let base_path = std::env::current_dir().expect("Failed to determine the current DIR.");
        let config_dir = base_path.join("config");
        let base_file = File::open(config_dir.join("base.toml"))?;
        let local_file = File::open(config_dir.join("local.toml"))?;

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
            },
        };

        let app_config = AppConfig::init()
            .add_source(base_file)?
            .add_source(local_file)?
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

        Ok(())
    }

    #[test]
    fn test_db_config_from_str_success() -> Result<()> {
        {
            let db_url = "postgres://my_uname:pwd@localhost:6666/my_db";
            let db_config = DbConfig::try_from(db_url)?;

            assert_eq!("my_uname", db_config.username);
            assert_eq!("pwd", db_config.password.expose_secret());
            assert_eq!("localhost", db_config.host);
            assert_eq!(6666, db_config.port);
            assert_eq!("my_db", db_config.db_name);
        }

        {
            let db_url = "postgres://my_uname:pwd@localhost:6666/my_db?ssl=disable";
            let db_config = DbConfig::try_from(db_url)?;

            assert_eq!("my_uname", db_config.username);
            assert_eq!("pwd", db_config.password.expose_secret());
            assert_eq!("localhost", db_config.host);
            assert_eq!(6666, db_config.port);
            assert_eq!("my_db", db_config.db_name);
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
