use thiserror::Error;

/// Base wizwalker exception, all exceptions raised should inherit from this
#[derive(Debug, Error)]
pub enum WizWalkerError {
    #[error("{msg}")]
    ExceptionalTimeout {
        msg: String,
        // Using a String representation or specific Boxed error if we don't want to type erase.
        // For simplicity and common practice, we'll keep the possible exception as an optional String or a Box<dyn std::error::Error>.
        // Since we are porting a general python `Exception`, let's just use an optional string description.
        possible_exception: Option<String>,
    },

    /// Raised when trying to do an action that requires a running client
    #[error("Client must be running to preform this action.")]
    ClientClosedError,

    /// Raised when doing something that requires a hook to be active,
    /// but it is not
    ///
    /// Attributes:
    ///     hook_name: Name of the hook that is not active
    #[error("{hook_name} is not active.")]
    HookNotActive { hook_name: String },

    /// Raised when trying to activate an active hook
    ///
    /// Attributes:
    ///     hook_name: Name of the hook that is already active
    #[error("{hook_name} was already activated.")]
    HookAlreadyActivated { hook_name: String },

    /// Raised to error with reading/writing memory
    #[error("Memory error")]
    WizWalkerMemoryError,

    /// Raised when a pattern has more than one result
    #[error("Pattern has more than one result")]
    PatternMultipleResults,

    /// Raised when the pattern scan fails
    #[error("Pattern {pattern} failed. You most likely need to restart the client.")]
    PatternFailed { pattern: String },

    /// Raised when trying to read memory that has deallocated
    #[error("Memory invalidated")]
    MemoryInvalidated,

    /// Raised when we couldn't read some memory
    #[error("{message}")]
    MemoryReadError { message: String },

    #[error("Address {address} out of bounds")]
    AddressOutOfRange { address: usize },

    /// Raised when we couldn't write to some memory
    #[error("Unable to write memory at address {address}.")]
    MemoryWriteError { address: usize },

    /// Raised when the value passed to an enum is not valid
    #[error("Error reading enum: {value} is not a vaid {enum_name}.")]
    ReadingEnumFailed { enum_name: String, value: String },

    /// Raised when trying to use a value from a hook before hook has run
    ///
    /// Attributes:
    ///     hook_name: Name of the hook that is not ready
    #[error("{hook_name} has not run yet and is not ready.")]
    HookNotReady { hook_name: String },

    /// Raised for errors relating to combat
    #[error("Combat error")]
    WizWalkerCombatError,

    /// Raised when trying to do an action that requires the client
    /// to be in combat
    #[error("Not in combat")]
    NotInCombat,

    /// Raised when trying to use a card that costs more pips then
    /// are available
    #[error("Not enough pips")]
    NotEnoughPips,

    /// Raised when trying to use a card that cost more mana than
    /// is available
    #[error("Not enough mana")]
    NotEnoughMana,

    /// Raised when trying to enchant an already enchanted card
    #[error("That card is already enchanted.")]
    CardAlreadyEnchanted,

    // TODO: remove in 2.0
    #[error("{key} already registered")]
    HotkeyAlreadyRegistered { key: String },
}
