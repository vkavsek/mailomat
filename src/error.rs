use crate::{app, config, email_client, model, web};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("web error: {0}")]
    Web(#[from] web::Error),
    #[error("email client error: {0}")]
    EmailClient(#[from] email_client::Error),
    #[error("model manager error: {0}")]
    Model(#[from] model::Error),
    #[error("serving error: {0}")]
    Serve(#[from] app::serve::ServeError),

    #[error("tokio joining error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
