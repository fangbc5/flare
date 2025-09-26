use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlareError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error("Email address error: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("Email build error: {0}")]
    Lettre(#[from] lettre::error::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("String error: {0}")]
    String(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type FlareResult<T> = Result<T, FlareError>;
