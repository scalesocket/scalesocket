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

    #[error("Stream from {0} closed")]
    StreamClosed(&'static str),

    #[error("Failed to use channel {0}")]
    ChannelError(&'static str),

    #[error("Failed to spawn process due to `{0}`")]
    ProcessSpawnError(String),

    #[error("Failed to process io due to `{0}`")]
    GenericError(#[from] std::io::Error),
}
