use thiserror::Error;

/// Convenience type alias used throughout wizwalker
pub type Result<T> = std::result::Result<T, WizWalkerError>;

/// All errors that wizwalker can raise
#[derive(Debug, Error)]
pub enum WizWalkerError {
    #[error("{msg}")]
    ExceptionalTimeout {
        msg: String,
        possible_exception: Option<String>,
    },

    #[error("Client must be running to preform this action.")]
    ClientClosedError,

    #[error("{hook_name} is not active.")]
    HookNotActive { hook_name: String },

    #[error("{hook_name} was already activated.")]
    HookAlreadyActivated { hook_name: String },

    #[error("Memory error")]
    WizWalkerMemoryError,

    #[error("Pattern has more than one result")]
    PatternMultipleResults,

    #[error("Pattern {pattern} failed. You most likely need to restart the client.")]
    PatternFailed { pattern: String },

    #[error("Memory invalidated")]
    MemoryInvalidated,

    #[error("{message}")]
    MemoryReadError { message: String },

    #[error("Address {address} out of bounds")]
    AddressOutOfRange { address: usize },

    #[error("Unable to write memory at address {address}.")]
    MemoryWriteError { address: usize },

    #[error("Error reading enum: {value} is not a vaid {enum_name}.")]
    ReadingEnumFailed { enum_name: String, value: String },

    #[error("{hook_name} has not run yet and is not ready.")]
    HookNotReady { hook_name: String },

    #[error("Combat error")]
    WizWalkerCombatError,

    #[error("Not in combat")]
    NotInCombat,

    #[error("Not enough pips")]
    NotEnoughPips,

    #[error("Not enough mana")]
    NotEnoughMana,

    #[error("That card is already enchanted.")]
    CardAlreadyEnchanted,

    #[error("{key} already registered")]
    HotkeyAlreadyRegistered { key: String },

    #[error("{0}")]
    Other(String),

    #[error("Windows API error: {0}")]
    WindowsError(#[from] windows::core::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
