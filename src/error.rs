use tower_sessions_redis_store::fred;

use crate::{app, config, database, email_client, web};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("web error: {0}")]
    Web(#[from] web::Error),
    #[error("email client error: {0}")]
    EmailClient(#[from] email_client::Error),
    #[error("redis manager error: {0}")]
    RedisManager(#[from] fred::error::Error),
    #[error("database manager error: {0}")]
    Database(#[from] database::Error),
    #[error("serving error: {0}")]
    Serve(#[from] app::serve::ServeError),

    #[error("tokio joining error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}
