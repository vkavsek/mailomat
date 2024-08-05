pub type ConfigResult<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to build the 'Enviroment' from the provided string.")]
    StringToEnvironmentFail,
    #[error("failed to parse 'DbConfig' from the provided string.")]
    StringToDbConfigFail,
    #[error("invalid email: {0}")]
    InvalidEmail(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml deserialization error: {0}")]
    TomlDeser(#[from] toml::de::Error),
    #[error("toml serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}
