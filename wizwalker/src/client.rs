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
use crate::memory::reader::MemoryReaderExt;
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
    /// Reads via: ClientHook export → client_object base → offset 304
    /// → client_zone pointer → offset 88 → zone name string.
    ///
    /// # Python equivalent
    /// ```python
    /// async def zone_name(self) -> Optional[str]:
    ///     client_zone = await self.client_object.client_zone()
    ///     if client_zone is not None:
    ///         return await client_zone.zone_name()
    /// ```
    pub fn zone_name(&self) -> Option<String> {
        // Get the client object base address from the ClientHook export
        let client_base = self.hook_handler.read_current_client_base().ok()?;
        let reader = self.process_reader()?;

        // Read client_zone pointer at offset 304 (per Python client_object.py)
        let client_zone_addr: u64 = reader.read_typed(client_base + 304).ok()?;
        if client_zone_addr == 0 {
            return None;
        }

        // Read zone name string at offset 88 (null-terminated UTF-8)
        // Per Python client_zone.py: `read_string_from_offset(88)`
        let zone_base = client_zone_addr as usize + 88;
        let mut string_bytes = Vec::new();
        let chunk = reader.read_bytes(zone_base, 128).ok()?;
        for &byte in chunk.iter() {
            if byte == 0 { break; }
            string_bytes.push(byte);
        }
        String::from_utf8(string_bytes).ok()
    }

    /// Read a shared_vector (start/end pointer pair) from the client object base.
    /// Returns a list of u64 addresses. Shared pointers are 16 bytes each.
    ///
    /// Python: `read_shared_vector(offset)` on MemoryObject
    fn read_shared_vector(&self, base_addr: usize, offset: usize) -> Vec<u64> {
        let reader = match self.process_reader() {
            Some(r) => r,
            None => return Vec::new(),
        };

        let start: u64 = reader.read_typed(base_addr + offset).unwrap_or(0);
        let end: u64 = reader.read_typed(base_addr + offset + 8).unwrap_or(0);

        if start == 0 || end == 0 || end <= start {
            return Vec::new();
        }

        let size = (end - start) as usize;
        let element_count = size / 16; // 16 bytes per shared pointer

        if element_count > 5000 {
            return Vec::new(); // Sanity check
        }

        let data = match reader.read_bytes(start as usize, size) {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let mut addrs = Vec::with_capacity(element_count);
        for i in 0..element_count {
            let off = i * 16;
            if off + 8 <= data.len() {
                let addr = u64::from_le_bytes(
                    data[off..off + 8].try_into().unwrap_or([0; 8]),
                );
                if addr != 0 {
                    addrs.push(addr);
                }
            }
        }
        addrs
    }

    /// Get the list of all game entities (WizClientObjects) currently loaded.
    ///
    /// Walks up the parent chain from client_object to find the root
    /// (the entity whose object_template is null), then reads its children
    /// via shared_vector at offset 392.
    ///
    /// # Python equivalent
    /// ```python
    /// async def get_base_entity_list(self):
    ///     root_client = await self.client_object.parent()
    ///     while not (await _is_root_object(root_client)):
    ///         root_client = await root_client.parent()
    ///     return await root_client.children()
    /// ```
    ///
    /// Returns: List of entity base addresses (u64).
    pub fn get_base_entity_list(&self) -> Vec<u64> {
        let reader = match self.process_reader() {
            Some(r) => r,
            None => return Vec::new(),
        };

        // Get client_object base from ClientHook export
        let client_base = match self.hook_handler.read_current_client_base() {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };

        // Walk up parent chain (offset 208 per CoreObject) to find root
        // Root is the entity with no object_template (offset 88 == 0)
        let mut current = client_base;
        for _ in 0..20 { // Safety limit
            let template: i64 = reader.read_typed(current + 88).unwrap_or(0);
            if template == 0 {
                break; // Found the root
            }
            let parent: i64 = reader.read_typed(current + 208).unwrap_or(0);
            if parent == 0 {
                break; // No more parents, use current
            }
            current = parent as usize;
        }

        // Read children from root via shared_vector at offset 392
        self.read_shared_vector(current, 392)
    }

    /// Get entities matching a specific object name.
    ///
    /// Python equivalent: `get_base_entities_with_name(name)`
    pub fn get_base_entities_with_name(&self, name: &str) -> Vec<u64> {
        let reader = match self.process_reader() {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut matching = Vec::new();
        for entity_addr in self.get_base_entity_list() {
            // Read object_template pointer at offset 88
            let template_addr: i64 = reader.read_typed(entity_addr as usize + 88).unwrap_or(0);
            if template_addr == 0 {
                continue;
            }

            // Read object_name from template (offset 80, null-terminated string)
            let name_base = template_addr as usize + 80;
            let mut name_bytes = Vec::new();
            if let Ok(chunk) = reader.read_bytes(name_base, 128) {
                for &byte in chunk.iter() {
                    if byte == 0 { break; }
                    name_bytes.push(byte);
                }
            }
            if let Ok(obj_name) = String::from_utf8(name_bytes) {
                if obj_name == name {
                    matching.push(entity_addr);
                }
            }
        }
        matching
    }

    /// Whether the client is currently in a loading screen.
    ///
    /// Checks for "TransitionWindow" or "PageFlip" in the root window children.
    ///
    /// # Python equivalent
    /// ```python
    /// async def is_loading(self) -> bool:
    ///     view = await self.get_world_view_window()
    ///     try:
    ///         await view.get_child_by_name("TransitionWindow")
    ///     except ValueError:
    ///         await self.root_window.get_child_by_name("PageFlip")
    /// ```
    pub fn is_loading(&self) -> bool {
        if let Some(root) = &self.root_window {
            // Check for TransitionWindow or PageFlip
            if let Ok(windows) = root.window.get_windows_with_name("TransitionWindow") {
                if !windows.is_empty() {
                    return true;
                }
            }
            if let Ok(windows) = root.window.get_windows_with_name("PageFlip") {
                if !windows.is_empty() {
                    return true;
                }
            }
        }
        false
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

    // ── Input Sending ───────────────────────────────────────────────

    /// Send a single key press (down + up) to the game window.
    ///
    /// # Python equivalent
    /// ```python
    /// user32.SendMessageW(window_handle, 0x100, key.value, 0)  # WM_KEYDOWN
    /// user32.SendMessageW(window_handle, 0x101, key.value, 0)  # WM_KEYUP
    /// ```
    pub fn send_key(&self, key: crate::constants::Keycode) {
        use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
        use windows::Win32::Foundation::{WPARAM, LPARAM};

        let wparam = Some(WPARAM(key.value() as usize));
        let lparam = Some(LPARAM(0));

        unsafe {
            SendMessageW(self.window_handle, 0x0100, wparam, lparam);
            SendMessageW(self.window_handle, 0x0101, wparam, lparam);
        }
    }

    /// Send a key press held for a duration (blocks the current thread).
    ///
    /// Sends repeated WM_KEYDOWN messages every 50ms for the specified
    /// duration, then sends a single WM_KEYUP.
    ///
    /// # Python equivalent
    /// ```python
    /// async def timed_send_key(window_handle, key, seconds):
    ///     keydown_task = asyncio.create_task(_send_keydown_forever(window_handle, key))
    ///     await asyncio.sleep(seconds)
    ///     keydown_task.cancel()
    ///     user32.SendMessageW(window_handle, 0x101, key.value, 0)
    /// ```
    pub fn timed_send_key(&self, key: crate::constants::Keycode, seconds: f64) {
        use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
        use windows::Win32::Foundation::{WPARAM, LPARAM};

        let wparam = Some(WPARAM(key.value() as usize));
        let lparam = Some(LPARAM(0));

        let duration = std::time::Duration::from_secs_f64(seconds);
        let start = std::time::Instant::now();

        // Send WM_KEYDOWN repeatedly every 50ms
        while start.elapsed() < duration {
            unsafe {
                SendMessageW(self.window_handle, 0x0100, wparam, lparam);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Send WM_KEYUP
        unsafe {
            SendMessageW(self.window_handle, 0x0101, wparam, lparam);
        }
    }

    /// Send a key with optional modifiers (Shift, Ctrl, Alt).
    ///
    /// Presses modifiers down, sends the main key, then releases modifiers.
    ///
    /// # Python equivalent: `utils.send_key_with_modifiers`
    pub fn send_key_with_modifiers(&self, key: crate::constants::Keycode, modifiers: &[crate::constants::Keycode]) {
        use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
        use windows::Win32::Foundation::{WPARAM, LPARAM};

        // Press modifiers down
        for modifier in modifiers {
            unsafe {
                SendMessageW(self.window_handle, 0x0100, Some(WPARAM(modifier.value() as usize)), Some(LPARAM(0)));
            }
        }

        // Send main key
        unsafe {
            SendMessageW(self.window_handle, 0x0100, Some(WPARAM(key.value() as usize)), Some(LPARAM(0)));
            SendMessageW(self.window_handle, 0x0101, Some(WPARAM(key.value() as usize)), Some(LPARAM(0)));
        }

        // Release modifiers
        for modifier in modifiers {
            unsafe {
                SendMessageW(self.window_handle, 0x0101, Some(WPARAM(modifier.value() as usize)), Some(LPARAM(0)));
            }
        }
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
