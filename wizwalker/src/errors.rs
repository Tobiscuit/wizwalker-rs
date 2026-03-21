use thiserror::Error;

/// Unified error type for the WizWalker crate.
/// Consolidates WizError, WizWalkerError, and WizWalkerMemoryError
/// previously scattered across separate modules.
#[derive(Error, Debug)]
pub enum WizWalkerError {
    #[error("Process not found")]
    ProcessNotFound,

    #[error("Windows API error: {0}")]
    WindowsApi(#[from] windows::core::Error),

    #[error("Failed to read memory at address 0x{address:X}")]
    MemoryRead { address: usize },

    #[error("Failed to write memory at address 0x{address:X}")]
    MemoryWrite { address: usize },

    #[error("Address out of range: 0x{0:X}")]
    AddressOutOfRange(usize),

    #[error("Pattern scan failed: {0}")]
    PatternNotFound(String),

    #[error("Pattern scan returned multiple results: {0}")]
    PatternMultipleResults(String),

    #[error("Hook already active: {0}")]
    HookAlreadyActive(String),

    #[error("Hook not active: {0}")]
    HookNotActive(String),

    #[error("Hook not ready: {0}")]
    HookNotReady(String),

    #[error("Failed to read enum '{enum_name}' with value {value}")]
    ReadingEnumFailed { enum_name: String, value: String },

    #[error("Memory invalidated: {0}")]
    MemoryInvalidated(MemoryInvalidated),

    #[error("Client closed")]
    ClientClosed,

    #[error("{0}")]
    Other(String),
}

/// Wrapper for memory invalidation messages.
#[derive(Debug, Clone)]
pub struct MemoryInvalidated(pub String);

impl std::fmt::Display for MemoryInvalidated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Result alias using the unified WizWalkerError.
pub type Result<T> = std::result::Result<T, WizWalkerError>;

// Legacy aliases for backward compatibility during migration.
// These allow existing code using the old names to compile without changes.
pub type WizError = WizWalkerError;
pub type WizWalkerMemoryError = WizWalkerError;
pub type MemoryReadError = WizWalkerError;
pub type ReadingEnumFailed = WizWalkerError;
