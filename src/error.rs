use thiserror::Error;

pub type AppResult<T> = ::std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
}
