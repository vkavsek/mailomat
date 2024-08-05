pub type ConfigResult<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to build the Enviroment from the provided string.")]
    StringToEnvironmentFail,
    #[error("Failed to parse DbConfig from the provided string.")]
    StringToDbConfigFail,
    #[error("Invalid email: {0}")]
    InvalidEmail(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDeser(#[from] toml::de::Error),
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}
