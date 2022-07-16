use std::convert::Infallible;
use thiserror::Error;

pub type AppResult<T> = ::std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to acquire process `{0}`")]
    ProcessStdIOError(&'static str),

    #[error("Failed the impossible")]
    Infallible(#[from] Infallible),

    #[error("Generic IO error")]
    Generic(#[from] std::io::Error),
}
