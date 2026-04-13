use thiserror::Error;

#[derive(Debug, Error)]
pub enum SitError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Inquire(#[from] inquire::error::InquireError),
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, SitError>;

impl From<String> for SitError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

impl From<&str> for SitError {
    fn from(message: &str) -> Self {
        Self::Message(message.to_owned())
    }
}
