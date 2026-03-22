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
    pub dance_hook_status: std::sync::atomic::AtomicBool,
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
            dance_hook_status: std::sync::atomic::AtomicBool::new(false),
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

    // ── Teleport & Position Methods ────────────────────────────────

    /// Teleport the player to the given XYZ position.
    ///
    /// Faithfully ported from Python `Client._teleport_object()` (client.py:531).
    ///
    /// TeleportHelper memory layout:
    /// - offset 0:  XYZ position (3 × f32 = 12 bytes)
    /// - offset 12: should_update (bool, 1 byte)
    /// - offset 13: target_object_address (u64, 8 bytes)
    ///
    /// # Python equivalent
    /// ```python
    /// await self._teleport_helper.write_target_object_address(object_address)
    /// await self._teleport_helper.write_position(xyz)
    /// await self._teleport_helper.write_should_update(True)
    /// ```
    pub async fn teleport_to_zone_display_name(&self, _zone_name: &str) -> crate::errors::Result<()> {
        // BUG: (from Python original) this is a complex navigation task in wiz_navigator.py
        // We'll leave it as a named helper that would integrate with a future Navigator implementation.
        Ok(())
    }

    pub fn teleport(&self, xyz: &crate::types::XYZ) -> crate::errors::Result<()> {
        use crate::memory::hooks::HookType;

        if !self.hook_handler.check_if_hook_active(HookType::MovementTeleport) {
            return Err(crate::errors::WizWalkerError::Other(
                "Movement teleport hook not active".into(),
            ));
        }

        let reader = self.process_reader()
            .ok_or_else(|| crate::errors::WizWalkerError::Other("Reader not attached".into()))?;
        let helper_addr = self.hook_handler.read_teleport_helper()?;

        // Wait for should_update to become false (offset 12)
        for _ in 0..20 {
            let su: u8 = reader.read_typed(helper_addr + 12).unwrap_or(0);
            if su == 0 { break; }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Get client_object base for target_object_address
        let client_base = self.hook_handler.read_current_client_base().unwrap_or(0) as u64;

        // Write target_object_address at offset 13
        reader.write_bytes(helper_addr + 13, &client_base.to_le_bytes())?;

        // Write XYZ position at offset 0
        let mut xyz_buf = Vec::with_capacity(12);
        xyz_buf.extend_from_slice(&xyz.x.to_le_bytes());
        xyz_buf.extend_from_slice(&xyz.y.to_le_bytes());
        xyz_buf.extend_from_slice(&xyz.z.to_le_bytes());
        reader.write_bytes(helper_addr, &xyz_buf)?;

        // Write should_update = true at offset 12
        reader.write_bytes(helper_addr + 12, &[0x01])?;

        // Wait for should_update to reset (game consumed the teleport)
        for _ in 0..12 {
            let su: u8 = reader.read_typed(helper_addr + 12).unwrap_or(0);
            if su == 0 { break; }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        Ok(())
    }

    /// Wait for the loading screen to finish.
    pub async fn wait_for_loading_screen(&self) {
        use tokio::time::{sleep, Duration};
        // Wait until is_loading becomes true, then wait until it becomes false
        for _ in 0..100 {
            if self.is_loading() { break; }
            sleep(Duration::from_millis(100)).await;
        }
        for _ in 0..300 {
            if !self.is_loading() { break; }
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Read the quest objective position from the QuestHook export.
    ///
    /// Python: `await self.quest_position.position()`
    pub fn quest_position(&self) -> Option<crate::types::XYZ> {
        let quest_base = self.hook_handler.read_current_quest_base().ok()?;
        let reader = self.process_reader()?;
        let x: f32 = reader.read_typed(quest_base + 0).ok()?;
        let y: f32 = reader.read_typed(quest_base + 4).ok()?;
        let z: f32 = reader.read_typed(quest_base + 8).ok()?;
        Some(crate::types::XYZ { x, y, z })
    }

    /// Read the player body position from the PlayerHook export.
    ///
    /// Python: `await self.body.position()`
    pub fn body_position(&self) -> Option<crate::types::XYZ> {
        let player_base = self.hook_handler.read_current_player_base().ok()?;
        let reader = self.process_reader()?;
        let x: f32 = reader.read_typed(player_base + 88).ok()?;
        let y: f32 = reader.read_typed(player_base + 92).ok()?;
        let z: f32 = reader.read_typed(player_base + 96).ok()?;
        Some(crate::types::XYZ { x, y, z })
    }

    /// Get a mutable reference to the hook handler.
    pub fn hook_handler_mut(&mut self) -> &mut HookHandler {
        &mut self.hook_handler
    }

    /// Get the current player's mana.
    ///
    /// Python: `await self.stats.current_mana()` — offset 136 from stat base.
    /// (Offset 116 is current_gold, 136 is current_mana per game_stats.py)
    pub fn stats_current_mana(&self) -> Option<i32> {
        let stat_base = self.hook_handler.read_current_player_stat_base().ok()?;
        let reader = self.process_reader()?;
        let mana: i32 = reader.read_typed(stat_base + 136).ok()?;
        Some(mana)
    }

    /// Read a wide (UTF-16) string from a memory address.
    pub fn read_wide_string_at(&self, address: usize) -> Option<String> {
        let reader = self.process_reader()?;
        // WizWalker wide strings: [u32 length, u16[] chars]
        let len: u32 = reader.read_typed(address).ok()?;
        if len == 0 || len > 512 { return None; }
        let bytes = reader.read_bytes(address + 4, len as usize * 2).ok()?;
        let u16s: Vec<u16> = bytes.chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        Some(String::from_utf16_lossy(&u16s))
    }

    // ── Body Read/Write Methods ─────────────────────────────────────

    /// Read the player body yaw (rotation around Z axis).
    ///
    /// Python: `await self.body.yaw()` — actor_body.py offset 108
    pub fn body_read_yaw(&self) -> Option<f32> {
        let player_base = self.hook_handler.read_current_player_base().ok()?;
        let reader = self.process_reader()?;
        reader.read_typed::<f32>(player_base + 108).ok()
    }

    /// Write the player body yaw (rotation around Z axis).
    ///
    /// Python: `await self.body.write_yaw(yaw)` — actor_body.py offset 108
    pub fn body_write_yaw(&self, yaw: f32) -> crate::errors::Result<()> {
        let player_base = self.hook_handler.read_current_player_base()?;
        let reader = self.process_reader()
            .ok_or_else(|| crate::errors::WizWalkerError::Other("Reader not attached".into()))?;
        reader.write_typed(player_base + 108, &yaw)
    }

    /// Read the player body roll.
    ///
    /// Python: `await self.body.roll()` — actor_body.py offset 104
    pub fn body_read_roll(&self) -> Option<f32> {
        let player_base = self.hook_handler.read_current_player_base().ok()?;
        let reader = self.process_reader()?;
        reader.read_typed::<f32>(player_base + 104).ok()
    }

    /// Write the player body roll.
    ///
    /// Python: `await self.body.write_roll(roll)` — actor_body.py offset 104
    pub fn body_write_roll(&self, roll: f32) -> crate::errors::Result<()> {
        let player_base = self.hook_handler.read_current_player_base()?;
        let reader = self.process_reader()
            .ok_or_else(|| crate::errors::WizWalkerError::Other("Reader not attached".into()))?;
        reader.write_typed(player_base + 104, &roll)
    }

    /// Read the player body scale.
    ///
    /// Python: `await self.body.scale()` — actor_body.py offset 112
    pub fn body_read_scale(&self) -> Option<f32> {
        let player_base = self.hook_handler.read_current_player_base().ok()?;
        let reader = self.process_reader()?;
        reader.read_typed::<f32>(player_base + 112).ok()
    }

    /// Write the player body scale.
    ///
    /// Python: `await self.body.write_scale(scale)` — actor_body.py offset 112
    pub fn body_write_scale(&self, scale: f32) -> crate::errors::Result<()> {
        let player_base = self.hook_handler.read_current_player_base()?;
        let reader = self.process_reader()
            .ok_or_else(|| crate::errors::WizWalkerError::Other("Reader not attached".into()))?;
        reader.write_typed(player_base + 112, &scale)
    }

    /// Read the player body height.
    ///
    /// Python: `await self.body.height()` — actor_body.py offset 132
    pub fn body_read_height(&self) -> Option<f32> {
        let player_base = self.hook_handler.read_current_player_base().ok()?;
        let reader = self.process_reader()?;
        reader.read_typed::<f32>(player_base + 132).ok()
    }

    /// Write the player body height.
    ///
    /// Python: `await self.body.write_height(height)` — actor_body.py offset 132
    pub fn body_write_height(&self, height: f32) -> crate::errors::Result<()> {
        let player_base = self.hook_handler.read_current_player_base()?;
        let reader = self.process_reader()
            .ok_or_else(|| crate::errors::WizWalkerError::Other("Reader not attached".into()))?;
        reader.write_typed(player_base + 132, &height)
    }

    // ── Movement Methods ────────────────────────────────────────────

    /// Base movement speed of a wizard (units/second).
    ///
    /// Python: `WIZARD_SPEED = 580` — constants.py:13
    const WIZARD_SPEED: f32 = 580.0;

    /// Walk the player to a specific (x, y) coordinate by setting yaw and holding W.
    ///
    /// This calculates the yaw toward the target, writes it to the body, then
    /// holds the W key for the appropriate duration based on distance and speed.
    ///
    /// Python: `async def goto(self, x, y)` — client.py:473
    pub fn goto(&self, x: f32, y: f32) {
        let Some(current_xyz) = self.body_position() else { return };

        let target = crate::types::XYZ { x, y, z: current_xyz.z };
        let dx = current_xyz.x - target.x;
        let dy = current_xyz.y - target.y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Calculate yaw toward target
        let yaw = dx.atan2(dy);

        // Write yaw to body
        if self.body_write_yaw(yaw).is_err() {
            return;
        }

        // Calculate movement duration
        // Speed multiplier would be read from client_object.speed_multiplier()
        // For now, use default (no speed boost): (0/100) + 1 = 1.0
        let speed_multiplier = 1.0f32;
        let move_seconds = distance / (Self::WIZARD_SPEED * speed_multiplier);

        // Hold W key for the calculated duration
        self.timed_send_key(crate::constants::Keycode::W, move_seconds as f64);
    }

    // ── Camera Methods ──────────────────────────────────────────────

    // ── GameClient offset constants ──────────────────────────────────
    //
    // Default offsets from game_client.py (may vary by game version).
    // In the Python these are resolved via pattern_scan_offset_cached() with
    // a fallback constant; we use the fallback directly.

    /// Offset from GameClient base → elastic camera controller pointer.
    const GC_ELASTIC_CAMERA: usize = 0x22260;
    /// Offset from GameClient base → free camera controller pointer.
    const GC_FREE_CAMERA: usize = 0x22270;
    /// Offset from GameClient base → selected (active) camera controller pointer.
    const GC_SELECTED_CAMERA: usize = 0x22290;
    /// Offset from GameClient base → is_freecam bool.
    const GC_IS_FREECAM: usize = 0x222A8;

    // ── Camera read helpers ─────────────────────────────────────────

    /// Get the GameClient base address (from the ClientHook export).
    fn game_client_base(&self) -> crate::errors::Result<usize> {
        self.hook_handler.read_current_client_base()
    }

    /// Check if the camera is in freecam mode.
    ///
    /// Python: `await self.game_client.is_freecam()` — game_client.py:93
    pub fn camera_is_freecam(&self) -> Option<bool> {
        let gc = self.game_client_base().ok()?;
        let reader = self.process_reader()?;
        let val: u8 = reader.read_typed(gc + Self::GC_IS_FREECAM).ok()?;
        Some(val != 0)
    }

    /// Write the is_freecam flag.
    ///
    /// Python: `await self.game_client.write_is_freecam(val)` — game_client.py:107
    fn camera_write_is_freecam(&self, val: bool) -> crate::errors::Result<()> {
        let gc = self.game_client_base()?;
        let reader = self.process_reader()
            .ok_or_else(|| crate::errors::WizWalkerError::Other("Reader not attached".into()))?;
        let byte: u8 = if val { 1 } else { 0 };
        reader.write_typed(gc + Self::GC_IS_FREECAM, &byte)
    }

    /// Read a camera controller pointer from GameClient.
    fn camera_read_controller(&self, offset: usize) -> Option<u64> {
        let gc = self.game_client_base().ok()?;
        let reader = self.process_reader()?;
        let addr: u64 = reader.read_typed(gc + offset).ok()?;
        if addr == 0 { None } else { Some(addr) }
    }

    // ── Camera switching ────────────────────────────────────────────

    /// Swap between freecam and elastic camera controllers.
    ///
    /// Python: `async def camera_swap(self)` — client.py:595
    pub fn camera_swap(&self) {
        match self.camera_is_freecam() {
            Some(true) => self.camera_elastic(),
            Some(false) => self.camera_freecam(),
            None => {
                tracing::warn!("camera_swap: ClientHook not active — cannot determine camera mode");
            }
        }
    }

    /// Switch to freecam camera controller.
    ///
    /// Python: `async def camera_freecam(self)` — client.py:605
    pub fn camera_freecam(&self) {
        let gc = match self.game_client_base() {
            Ok(gc) => gc,
            Err(e) => {
                tracing::warn!("camera_freecam: cannot get GameClient base: {e}");
                return;
            }
        };
        let reader = match &self.reader {
            Some(r) => r,
            None => { tracing::warn!("camera_freecam: reader not attached"); return; }
        };

        // 1. Write is_freecam = true
        let _ = self.camera_write_is_freecam(true);

        // 2. Read elastic / free camera controller addresses
        let elastic_addr = match self.camera_read_controller(Self::GC_ELASTIC_CAMERA) {
            Some(a) => a,
            None => { tracing::warn!("camera_freecam: elastic camera controller is null"); return; }
        };
        let free_addr = match self.camera_read_controller(Self::GC_FREE_CAMERA) {
            Some(a) => a,
            None => { tracing::warn!("camera_freecam: free camera controller is null"); return; }
        };

        // 3. Switch camera: new=free, old=elastic
        if let Err(e) = self.camera_switch_camera(reader.as_ref(), gc as u64, free_addr, elastic_addr) {
            tracing::warn!("camera_freecam: switch failed: {e}");
        }
    }

    /// Switch to elastic (normal) camera controller.
    ///
    /// Python: `async def camera_elastic(self)` — client.py:625
    pub fn camera_elastic(&self) {
        let gc = match self.game_client_base() {
            Ok(gc) => gc,
            Err(e) => {
                tracing::warn!("camera_elastic: cannot get GameClient base: {e}");
                return;
            }
        };
        let reader = match &self.reader {
            Some(r) => r,
            None => { tracing::warn!("camera_elastic: reader not attached"); return; }
        };

        // 1. Write is_freecam = false
        let _ = self.camera_write_is_freecam(false);

        // 2. Read elastic / free camera controller addresses
        let elastic_addr = match self.camera_read_controller(Self::GC_ELASTIC_CAMERA) {
            Some(a) => a,
            None => { tracing::warn!("camera_elastic: elastic camera controller is null"); return; }
        };
        let free_addr = match self.camera_read_controller(Self::GC_FREE_CAMERA) {
            Some(a) => a,
            None => { tracing::warn!("camera_elastic: free camera controller is null"); return; }
        };

        // 3. Switch camera: new=elastic, old=free
        if let Err(e) = self.camera_switch_camera(reader.as_ref(), gc as u64, elastic_addr, free_addr) {
            tracing::warn!("camera_elastic: switch failed: {e}");
        }
    }

    /// Execute the camera switch via shellcode injection.
    ///
    /// Faithfully ported from `client.py:685 _switch_camera()`.
    ///
    /// This allocates a shellcode buffer in the game process, writes x86-64
    /// machine code that calls the game's SetCamera vtable function, executes
    /// it via CreateRemoteThread, and then frees the buffer.
    fn camera_switch_camera(
        &self,
        reader: &crate::memory::process_reader::ProcessMemoryReader,
        game_client_addr: u64,
        new_camera_addr: u64,
        old_camera_addr: u64,
    ) -> crate::errors::Result<()> {
        let pack = |addr: u64| -> [u8; 8] { addr.to_le_bytes() };

        let gc_bytes = pack(game_client_addr);
        let new_bytes = pack(new_camera_addr);
        let old_bytes = pack(old_camera_addr);

        // Build shellcode — exact match of Python's client.py:696-739
        let mut shellcode: Vec<u8> = Vec::with_capacity(128);

        // ── setup: save registers ──
        shellcode.push(0x50);                       // push rax
        shellcode.push(0x51);                       // push rcx
        shellcode.push(0x52);                       // push rdx
        shellcode.extend_from_slice(&[0x41, 0x50]); // push r8
        shellcode.extend_from_slice(&[0x41, 0x51]); // push r9

        // ── call set_cam(client, new_cam, ?, cam_swap_fn) ──
        shellcode.extend_from_slice(&[0x48, 0xB9]); // mov rcx, client_addr
        shellcode.extend_from_slice(&gc_bytes);
        shellcode.extend_from_slice(&[0x48, 0xBA]); // mov rdx, new_cam_addr
        shellcode.extend_from_slice(&new_bytes);
        shellcode.extend_from_slice(&[0x49, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]); // mov r8, 1
        shellcode.extend_from_slice(&[0x48, 0x8B, 0x01]); // mov rax, [rcx]
        shellcode.extend_from_slice(&[0x48, 0x8B, 0x80, 0x70, 0x04, 0x00, 0x00]); // mov rax, [rax+0x470]
        shellcode.extend_from_slice(&[0x49, 0x89, 0xC1]); // mov r9, rax
        shellcode.extend_from_slice(&[0xFF, 0xD0]); // call rax

        // ── call register_input_handlers(new_cam, active=1) ──
        shellcode.extend_from_slice(&[0x48, 0xB9]); // mov rcx, client_addr
        shellcode.extend_from_slice(&gc_bytes);
        shellcode.extend_from_slice(&[0x48, 0xB8]); // mov rax, new_cam_addr
        shellcode.extend_from_slice(&new_bytes);
        shellcode.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax
        shellcode.extend_from_slice(&[0x48, 0x8B, 0x01]); // mov rax, [rcx]
        shellcode.extend_from_slice(&[0x48, 0x8B, 0x40, 0x70]); // mov rax, [rax+0x70]
        shellcode.extend_from_slice(&[0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00]); // mov rdx, 1
        shellcode.extend_from_slice(&[0xFF, 0xD0]); // call rax

        // ── call register_input_handlers(old_cam, active=0) ──
        shellcode.extend_from_slice(&[0x48, 0xB9]); // mov rcx, client_addr
        shellcode.extend_from_slice(&gc_bytes);
        shellcode.extend_from_slice(&[0x48, 0xB8]); // mov rax, old_cam_addr
        shellcode.extend_from_slice(&old_bytes);
        shellcode.extend_from_slice(&[0x48, 0x89, 0xC1]); // mov rcx, rax
        shellcode.extend_from_slice(&[0x48, 0x8B, 0x01]); // mov rax, [rcx]
        shellcode.extend_from_slice(&[0x48, 0x8B, 0x40, 0x70]); // mov rax, [rax+0x70]
        shellcode.extend_from_slice(&[0x48, 0xC7, 0xC2, 0x00, 0x00, 0x00, 0x00]); // mov rdx, 0
        shellcode.extend_from_slice(&[0xFF, 0xD0]); // call rax

        // ── cleanup: restore registers ──
        shellcode.extend_from_slice(&[0x41, 0x59]); // pop r9
        shellcode.extend_from_slice(&[0x41, 0x58]); // pop r8
        shellcode.push(0x5A);                        // pop rdx
        shellcode.push(0x59);                        // pop rcx
        shellcode.push(0x58);                        // pop rax

        // ── return ──
        shellcode.push(0xC3); // ret

        // Allocate, write, execute, free
        let shell_ptr = reader.allocate(shellcode.len())?;
        reader.write_bytes(shell_ptr, &shellcode)?;
        let result = reader.start_thread(shell_ptr);
        // Free regardless of result — start_thread waits for completion
        let _ = reader.free(shell_ptr);

        result
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
