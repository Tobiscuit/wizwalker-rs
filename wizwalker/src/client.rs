use std::sync::Arc;

use tracing::warn;
use windows::Win32::Foundation::{HANDLE, HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, GetWindowTextW, SetForegroundWindow, SetWindowTextW,
};

use crate::errors::{Result, WizWalkerError};
use crate::memory::handler::HookHandler;
use crate::memory::memory_object::DynamicMemoryObject;
use crate::memory::objects::duel::DynamicDuel;
use crate::memory::objects::game_stats::DynamicGameStats;
use crate::memory::objects::window::CurrentRootWindow;
use crate::memory::process_reader::ProcessMemoryReader;
use crate::memory::reader::MemoryReader;
use crate::mouse_handler::MouseHandler;

/// Represents a single connected Wizard101 game instance.
///
/// Lifecycle:
/// 1. Created by `ClientHandler::get_new_clients()` via `from_handles()`
/// 2. Caller invokes `open()` to attach the memory reader and initialize memory objects
/// 3. Memory objects (`duel`, `game_stats`, `root_window`) are now live
/// 4. `close()` cleans up hooks and releases the reader
/// 5. `Drop` acts as a safety net if `close()` was not called
///
/// # Python equivalent
/// `wizwalker/client.py` — `Client` class.
pub struct Client {
    // ── Win32 handles ───────────────────────────────────────────────
    pub window_handle: HWND,
    pub process_handle: HANDLE,
    pub process_id: u32,

    // ── Memory reader (initialized by open()) ───────────────────────
    /// Concrete reader wrapping `ReadProcessMemory`/`WriteProcessMemory`.
    /// `None` until `open()` is called.
    reader: Option<Arc<ProcessMemoryReader>>,

    // ── Subsystems ──────────────────────────────────────────────────
    pub hook_handler: HookHandler,
    pub mouse_handler: MouseHandler,

    // ── Live memory objects (initialized by open()) ─────────────────
    /// Combat duel state. Set by `open()`.
    pub duel: Option<DynamicDuel>,
    /// Player game stats. Set by `open()`.
    pub game_stats: Option<DynamicGameStats>,
    /// Root UI window for navigating the game's UI tree. Set by `open()`.
    pub root_window: Option<CurrentRootWindow>,

    // ── Internal state ──────────────────────────────────────────────
    /// Whether `close()` has been called.
    closed: bool,
}

// SAFETY: HWND and HANDLE are raw kernel handles; safe to send between threads.
unsafe impl Send for Client {}

impl Client {
    // ── Construction ────────────────────────────────────────────────

    /// Create a `Client` from raw Win32 handles.
    ///
    /// The client is **not yet connected** to game memory — call `open()`
    /// to initialize the `ProcessMemoryReader` and memory objects.
    ///
    /// # Arguments
    /// * `hwnd` — Window handle from `EnumWindows`
    /// * `process_handle` — Handle from `OpenProcess` with VM_READ/WRITE/OPERATION access
    /// * `process_id` — Process ID from `GetWindowThreadProcessId`
    pub fn from_handles(hwnd: HWND, process_handle: HANDLE, process_id: u32) -> Self {
        let hook_handler = HookHandler::new();
        let mouse_handler = MouseHandler::new(hwnd);

        Self {
            window_handle: hwnd,
            process_handle,
            process_id,
            reader: None,
            hook_handler,
            mouse_handler,
            duel: None,
            game_stats: None,
            root_window: None,
            closed: false,
        }
    }

    // ── Lifecycle ───────────────────────────────────────────────────

    /// Attach to the game process and initialize all memory objects.
    ///
    /// This creates the `ProcessMemoryReader` from the held process handle,
    /// then initializes `DynamicDuel` and `DynamicGameStats` with a base
    /// address of 0 (the hook system will provide real addresses once
    /// hooks are activated in Phase 4).
    ///
    /// # Python equivalent
    /// `Client.__init__()` — Python does this in the constructor because pymem
    /// is synchronous. In Rust, we separate construction from initialization
    /// so that `from_handles()` can't fail.
    pub fn open(&mut self) -> Result<()> {
        if self.reader.is_some() {
            return Err(WizWalkerError::Other(
                "Client already opened".into(),
            ));
        }

        // Create the concrete memory reader from the process handle.
        let reader = Arc::new(ProcessMemoryReader::new(self.process_handle));

        // Verify the process is actually running before proceeding.
        if !reader.is_running() {
            return Err(WizWalkerError::ClientClosed);
        }

        // Initialize memory objects with base address 0.
        // Memory objects exist but return errors if read before hooks provide
        // real addresses. Call activate_hooks() to start capturing live data.
        let dyn_reader: Arc<dyn MemoryReader> = reader.clone();

        // DynamicDuel — base address 0 (placeholder until duel hook resolves it)
        if let Ok(inner) = DynamicMemoryObject::new(dyn_reader.clone(), 0) {
            self.duel = Some(DynamicDuel::new(inner));
        }

        // DynamicGameStats — base address 0 (placeholder until stat hook resolves it)
        if let Ok(stats) = DynamicGameStats::new(dyn_reader.clone(), 0) {
            self.game_stats = Some(stats);
        }

        // Attach the reader to the hook handler so it can inject hooks.
        self.hook_handler.attach(reader.clone());

        self.reader = Some(reader);

        Ok(())
    }

    /// Disconnect from the game process and clean up all hooks.
    ///
    /// Safe to call multiple times — subsequent calls are no-ops.
    ///
    /// # Python equivalent
    /// `Client.close()` — unhooks all active hooks, restores original bytes.
    pub fn close(&mut self) {
        if self.closed {
            return;
        }

        // If the game is still running, close hooks gracefully.
        // This restores original bytes, frees allocated memory, and
        // rewrites the autobot function.
        if self.is_running() {
            self.hook_handler.close();
        }

        // Drop memory objects (they hold Arc<dyn MemoryReader> refs).
        self.duel = None;
        self.game_stats = None;
        self.root_window = None;

        // Release the reader.
        self.reader = None;

        self.closed = true;
    }

    /// Activate all memory hooks (Player, Stat, Quest, Client, RootWindow,
    /// RenderContext, MovementTeleport).
    ///
    /// Must be called after `open()`. This injects shellcode into the game
    /// process that captures live game state. Once hooks are active, the
    /// export addresses will contain real pointers for memory object reads.
    ///
    /// # Python equivalent
    /// `HookHandler.activate_all_hooks()`
    pub fn activate_hooks(&mut self) -> Result<()> {
        self.hook_handler.activate_all_hooks()
    }

    /// Deactivate all hooks without closing the client.
    pub fn deactivate_hooks(&mut self) {
        self.hook_handler.close();
    }

    // ── Memory Reader Access ────────────────────────────────────────

    /// Get the memory reader as a trait object for constructing memory objects.
    ///
    /// Returns `None` if `open()` has not been called yet.
    pub fn reader(&self) -> Option<Arc<dyn MemoryReader>> {
        self.reader.as_ref().map(|r| r.clone() as Arc<dyn MemoryReader>)
    }

    /// Get the concrete `ProcessMemoryReader` for operations that need it directly.
    ///
    /// Returns `None` if `open()` has not been called yet.
    pub fn process_reader(&self) -> Option<Arc<ProcessMemoryReader>> {
        self.reader.clone()
    }

    // ── Game State Queries ──────────────────────────────────────────

    /// Whether the client is currently in a combat encounter.
    ///
    /// Reads the duel phase from memory. Returns `false` if:
    /// - The client is not opened
    /// - The duel memory object is not initialized
    /// - The duel phase is `Ended` or unreadable
    ///
    /// # Python equivalent
    /// ```python
    /// async def in_battle(self) -> bool:
    ///     try:
    ///         duel_phase = await self.duel.duel_phase()
    ///     except (ReadingEnumFailed, MemoryReadError):
    ///         return False
    ///     else:
    ///         return duel_phase is not DuelPhase.ended
    /// ```
    pub fn in_battle(&self) -> bool {
        let Some(duel) = &self.duel else {
            return false;
        };

        match duel.duel_phase() {
            Ok(phase) => {
                use crate::memory::objects::duel::DuelPhase;
                phase != DuelPhase::Ended
            }
            Err(_) => false,
        }
    }

    /// Current zone name, if available.
    ///
    /// Reads from the client object's client zone memory.
    /// Returns `None` if the zone can't be read.
    pub fn zone_name(&self) -> Option<String> {
        // TODO: Read from client_object -> client_zone -> zone_name
        // Requires ClientObject to be wired up (needs hooks for base address).
        None
    }

    // ── Win32 Window Operations ─────────────────────────────────────

    /// Get the window title.
    pub fn title(&self) -> String {
        let mut title_buf = [0u16; 256];
        unsafe {
            let len = GetWindowTextW(self.window_handle, &mut title_buf);
            String::from_utf16_lossy(&title_buf[..len as usize])
        }
    }

    /// Set the window title.
    pub fn set_title(&self, new_title: &str) {
        let mut title_u16: Vec<u16> = new_title.encode_utf16().collect();
        title_u16.push(0);
        unsafe {
            let _ = SetWindowTextW(
                self.window_handle,
                windows::core::PCWSTR(title_u16.as_ptr()),
            );
        }
    }

    /// Whether this client's window is the foreground window.
    pub fn is_foreground(&self) -> bool {
        unsafe { GetForegroundWindow() == self.window_handle }
    }

    /// Bring this client's window to the foreground.
    pub fn set_foreground(&self) {
        unsafe {
            let _ = SetForegroundWindow(self.window_handle);
        }
    }

    /// Get the window rectangle (position + size).
    pub fn window_rectangle(&self) -> Option<RECT> {
        let mut rect = RECT::default();
        unsafe {
            if GetWindowRect(self.window_handle, &mut rect).is_ok() {
                Some(rect)
            } else {
                None
            }
        }
    }

    /// Whether the game process is still running.
    ///
    /// Checks via `IsWindow` (window handle validity) as a fast path.
    /// If a reader exists, also checks the process exit code.
    pub fn is_running(&self) -> bool {
        // Fast check: is the window handle still valid?
        let window_alive = unsafe {
            windows::Win32::UI::WindowsAndMessaging::IsWindow(Some(self.window_handle)).as_bool()
        };

        if !window_alive {
            return false;
        }

        // If we have a reader, also verify the process itself.
        if let Some(reader) = &self.reader {
            return reader.is_running();
        }

        // No reader yet — window exists, assume running.
        true
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if !self.closed && self.reader.is_some() {
            warn!(
                pid = self.process_id,
                "Client dropped without calling close() — cleaning up"
            );
            self.close();
        }
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("window_handle", &self.window_handle)
            .field("process_id", &self.process_id)
            .field("opened", &self.reader.is_some())
            .field("closed", &self.closed)
            .finish()
    }
}
