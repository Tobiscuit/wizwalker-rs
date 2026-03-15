use thiserror::Error;

#[derive(Error, Debug)]
pub enum WizWalkerError {
    #[error("Address out of range: {0:#X}")]
    AddressOutOfRange(u64),

    #[error("Client closed error")]
    ClientClosedError,

    #[error("Memory read error at address {0:#X}")]
    MemoryReadError(u64),

    #[error("Memory write error at address {0:#X}")]
    MemoryWriteError(u64),

    #[error("Pattern failed to match: {0:?}")]
    PatternFailed(Vec<u8>),

    #[error("Pattern returned multiple results: {0}")]
    PatternMultipleResults(String),

    #[error("Failed to read enum {0} with value {1}")]
    ReadingEnumFailed(String, i32),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, WizWalkerError>;
