use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
}
