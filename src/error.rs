use crate::{config, email_client, model, web};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Config Error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Web Error: {0}")]
    Web(#[from] web::Error),
    #[error("Email Client Error: {0}")]
    EmailClient(#[from] email_client::Error),
    #[error("Model Manager Error: {0}")]
    Model(#[from] model::Error),

    #[error("Tokio Joining Error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
