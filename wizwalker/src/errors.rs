use thiserror::Error;

#[derive(Error, Debug)]
pub enum WizError {
    #[error("Process not found")]
    ProcessNotFound,
    #[error("Windows API error: {0}")]
    WindowsError(#[from] windows::core::Error),
}
