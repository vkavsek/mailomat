use derive_more::From;

use crate::{config, email_client, web};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Config(config::ConfigError),
    #[from]
    Web(web::Error),
    #[from]
    EmailClient(email_client::Error),

    #[from]
    TokioJoin(tokio::task::JoinError),
    #[from]
    Io(std::io::Error),
    #[from]
    SqlxMigrate(sqlx::migrate::MigrateError),

    #[from]
    ModelSqlxTestInit(sqlx::Error),
    ModelFailToCreatePool(String),
}

// Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
