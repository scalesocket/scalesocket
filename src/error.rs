use thiserror::Error;

pub type AppResult<T> = ::std::result::Result<T, AppError>;

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to connect to {0} due to {1}")]
    NetworkError(String, String),

    #[error("Failed to acquire process `{0}`")]
    ProcessStdIOError(&'static str),

    #[error("Failed to stream {0}")]
    StreamError(&'static str),

    #[error("Failed to use channel {0}")]
    ChannelError(&'static str),

    #[error("Failed to process io")]
    GenericError(#[from] std::io::Error),
}
