use thiserror::Error;

pub type AppResult<T> = ::std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to acquire process `{0}`")]
    ProcessStdIOError(&'static str),

    #[error("Failed to stream {0}")]
    StreamError(&'static str),

    #[error("Failed to use channel {0}")]
    ChannelError(&'static str),

    #[error("Failed to process io")]
    GenericError(#[from] std::io::Error),
}
