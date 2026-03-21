use std::collections::HashMap;
use std::sync::Arc;

use wizwalker::client::Client;
use wizwalker::client_handler::ClientHandler;

// ── Serializable Error Type ─────────────────────────────────────────────
// Tauri v2 requires command errors to implement `serde::Serialize`.
// This enum maps wizwalker errors to structured frontend messages.

#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum CommandError {
    /// No clients are connected to the handler.
    NoClients(String),
    /// The specified client was not found.
    ClientNotFound(String),
    /// A memory read/write operation failed.
    MemoryError(String),
    /// A hook activation or deactivation failed.
    HookError(String),
    /// A generic/unexpected error occurred.
    Internal(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoClients(msg) => write!(f, "No clients: {msg}"),
            Self::ClientNotFound(msg) => write!(f, "Client not found: {msg}"),
            Self::MemoryError(msg) => write!(f, "Memory error: {msg}"),
            Self::HookError(msg) => write!(f, "Hook error: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

/// Convenience alias for command return types.
pub type CommandResult<T> = Result<T, CommandError>;

// ── Serializable DTOs ───────────────────────────────────────────────────

/// Client information for the frontend.
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub label: String,
    pub pid: u32,
    pub title: String,
    pub hooked: bool,
    pub zone: String,
    pub is_foreground: bool,
    pub is_running: bool,
}

/// XYZ position.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Player stats read from game memory.
#[derive(serde::Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStats {
    pub max_health: i32,
    pub max_mana: i32,
    pub power_pip_chance: f32,
    pub accuracy: f32,
    pub resist: f32,
    pub damage: f32,
    pub critical: i32,
    pub pierce: f32,
}

/// Camera state read from game memory.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct CameraState {
    pub position: Position,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub fov: f32,
    pub distance: f32,
}

/// Real-time telemetry pushed to the frontend via events.
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryPayload {
    pub active_client: Option<String>,
    pub position: Position,
    pub zone: String,
    pub in_combat: bool,
}

// ── Central Application State ───────────────────────────────────────────

/// Shared application state managed by Tauri.
///
/// Uses `std::sync::Mutex` (not tokio) because:
/// 1. All Win32 operations (memory read/write, EnumWindows) are synchronous
/// 2. Tauri commands using std::sync::Mutex can be synchronous (non-async),
///    which avoids the Send requirement on the Future
/// 3. Per Tauri v2 docs: sync commands use `.lock().unwrap()` pattern
pub struct WizState {
    /// Core wizwalker client handler (scans, manages game instances).
    pub handler: ClientHandler,

    /// Currently connected + opened clients, keyed by label ("P1".."P4").
    /// Uses `tokio::sync::Mutex` because `ClientHandler::get_new_clients()` returns
    /// `Arc<tokio::sync::Mutex<Client>>`. Sync commands access via `blocking_lock()`
    /// per tokio docs (context7).
    pub clients: HashMap<String, Arc<tokio::sync::Mutex<Client>>>,

    /// Toggle states for all hooks/features.
    pub toggles: HashMap<String, bool>,

    /// Speed multiplier value (1.0 = normal game speed).
    pub speed_multiplier: f64,

    /// Index of the currently active/focused client (0-based).
    pub active_client_idx: usize,
}

// SAFETY: WizState contains ClientHandler (has HWND — raw kernel handles that are
// safe to send between threads) and Arc<Mutex<Client>> (Send via Client's impl).
// All access is synchronized through std::sync::Mutex.
unsafe impl Send for WizState {}
unsafe impl Sync for WizState {}

impl WizState {
    pub fn new() -> Self {
        let toggles = HashMap::from([
            ("speedhack".into(), false),
            ("auto_combat".into(), false),
            ("auto_dialogue".into(), false),
            ("auto_sigil".into(), false),
            ("auto_questing".into(), false),
            ("pet_trainer".into(), false),
            ("auto_potions".into(), false),
            ("anti_afk".into(), false),
        ]);

        Self {
            handler: ClientHandler::new(),
            clients: HashMap::new(),
            toggles,
            speed_multiplier: 1.0,
            active_client_idx: 0,
        }
    }

    /// Get the label for a client by index (0→"P1", 1→"P2", etc.)
    pub fn client_label(idx: usize) -> String {
        format!("P{}", idx + 1)
    }
}
