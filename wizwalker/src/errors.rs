use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Value error: {0}")]
    Value(String),
}

pub type Result<T> = std::result::Result<T, Error>;
