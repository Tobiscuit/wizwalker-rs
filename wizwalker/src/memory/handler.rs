use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::{Result, WizWalkerError};
use crate::memory::hooks::{
    HookInstance, HookType, SimpleHook,
    PlayerHook, PlayerStatHook, QuestHook, ClientHook,
    RootWindowHook, RenderContextHook,
    MovementTeleportHook, MovementTeleportHookInstance,
    MouselessCursorMoveHook, MouselessCursorHookInstance,
};
use crate::memory::process_reader::ProcessMemoryReader;
use crate::memory::reader::MemoryReader;

/// Byte pattern for the autobot function (used as a hook injection point).
///
/// The autobot is a large, rarely-called function in `WizardGraphicalClient.exe`.
/// We NOP it out and write our hook shellcode into the space.
///
/// # Python equivalent
/// `wizwalker/memory/handler.py` — `AUTOBOT_PATTERN`
pub const AUTOBOT_PATTERN: &[u8] = &[
    0x48, 0x89, 0x5C, 0x24, 0x2E, 0x48, 0x89, 0x74, 0x24, 0x2E, 0x48, 0x89, 0x7C, 0x24, 0x2E,
    0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57,
    0x48, 0x8D, 0xAC, 0x24, 0x2E, 0x2E, 0x2E, 0x2E, 0x48, 0x81, 0xEC, 0x2E, 0x2E, 0x2E, 0x2E,
    0x48, 0x8B, 0x05, 0x2E, 0x2E, 0x2E, 0x2E, 0x48, 0x33, 0xC4, 0x48, 0x89, 0x85, 0x2E, 0x2E, 0x2E, 0x2E,
    0x4C, 0x8B, 0xF1, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x80, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x0F, 0x84, 0x2E, 0x2E, 0x2E, 0x2E,
];

/// Size of the autobot function that we can safely overwrite.
pub const AUTOBOT_SIZE: usize = 4100;

/// Manages memory hooks injected into the Wizard101 process.
///
/// # Lifecycle
/// 1. `attach(reader)` — stores the memory reader
/// 2. `activate_*_hook()` — scans, generates, writes each hook
/// 3. `read_current_*_base()` — reads exported values from hooks
/// 4. `close()` — unhooks all, restores autobot, releases reader
///
/// # Python equivalent
/// `wizwalker/memory/handler.py` — `HookHandler` class.
pub struct HookHandler {
    /// Memory reader for the game process. Set by `attach()`.
    reader: Option<Arc<ProcessMemoryReader>>,

    // ── Autobot state ──────────────────────────────────────────────
    /// Base address of the autobot function in the game process.
    autobot_address: Option<usize>,
    /// Original bytes of the autobot function (for restoration).
    autobot_original_bytes: Option<Vec<u8>>,
    /// Current write position within the autobot function.
    autobot_pos: usize,

    // ── Active hooks ───────────────────────────────────────────────
    /// Standard hooks (Player, PlayerStat, Quest, Client, RootWindow, RenderContext).
    active_hooks: HashMap<HookType, HookInstance>,
    /// Movement teleport hook (has extra state for JE restoration).
    teleport_hook: Option<MovementTeleportHookInstance>,
    /// Mouseless cursor hook (has extra state for bool/SetCursorPos restoration).
    mouseless_hook: Option<MouselessCursorHookInstance>,

    // ── Export address cache ───────────────────────────────────────
    /// Maps export names to allocated addresses for quick lookup.
    export_addrs: HashMap<String, usize>,
}

impl HookHandler {
    pub fn new() -> Self {
        Self {
            reader: None,
            autobot_address: None,
            autobot_original_bytes: None,
            autobot_pos: 0,
            active_hooks: HashMap::new(),
            teleport_hook: None,
            mouseless_hook: None,
            export_addrs: HashMap::new(),
        }
    }

    // ── Reader attachment ───────────────────────────────────────────

    /// Attach a memory reader. Must be called before any hooks.
    pub fn attach(&mut self, reader: Arc<ProcessMemoryReader>) {
        self.reader = Some(reader);
    }

    /// Get the reader, if attached.
    pub fn reader(&self) -> Option<&Arc<ProcessMemoryReader>> {
        self.reader.as_ref()
    }

    /// Get the reader as a trait object.
    pub fn dyn_reader(&self) -> Option<Arc<dyn MemoryReader>> {
        self.reader.as_ref().map(|r| r.clone() as Arc<dyn MemoryReader>)
    }

    // ── Autobot lifecycle ──────────────────────────────────────────

    /// Prepare the autobot function for shellcode storage.
    ///
    /// Pattern scans to find it, saves original bytes, NOPs it out.
    fn prepare_autobot(&mut self) -> Result<()> {
        if self.autobot_address.is_some() {
            return Ok(()); // Already prepared.
        }

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?;

        // Pattern scan to find the autobot function.
        let results = reader.pattern_scan(
            AUTOBOT_PATTERN,
            Some("WizardGraphicalClient.exe"),
            false,
        )?;
        let address = results[0];

        // Save original bytes (just the pattern length, enough for restoration).
        let original_bytes = reader.read_bytes(address, AUTOBOT_PATTERN.len())?;

        // NOP the entire function to make room for shellcode.
        let nops = vec![0x00u8; AUTOBOT_SIZE];
        reader.write_bytes(address, &nops)?;

        self.autobot_address = Some(address);
        self.autobot_original_bytes = Some(original_bytes);
        self.autobot_pos = 0;

        Ok(())
    }

    /// Allocate bytes within the autobot function.
    fn allocate_autobot(&mut self, size: usize) -> Result<usize> {
        let base = self.autobot_address
            .ok_or_else(|| WizWalkerError::Other("Autobot not prepared".into()))?;

        if self.autobot_pos + size > AUTOBOT_SIZE {
            return Err(WizWalkerError::Other(
                "Somehow went over autobot size".into()
            ));
        }

        let addr = base + self.autobot_pos;
        self.autobot_pos += size;
        Ok(addr)
    }

    /// Restore autobot function original bytes.
    fn rewrite_autobot(&mut self) -> Result<()> {
        if let (Some(address), Some(original_bytes)) =
            (self.autobot_address, &self.autobot_original_bytes)
        {
            if let Some(reader) = &self.reader {
                // Check if the original bytes are already there.
                if let Ok(current_bytes) = reader.read_bytes(address, original_bytes.len()) {
                    if current_bytes != *original_bytes {
                        let _ = reader.write_bytes(address, original_bytes);
                    }
                }
            }
        }
        Ok(())
    }

    // ── Hook state queries ──────────────────────────────────────────

    pub fn check_if_hook_active(&self, hook_type: HookType) -> bool {
        match hook_type {
            HookType::MovementTeleport => self.teleport_hook.is_some(),
            HookType::MouselessCursor => self.mouseless_hook.is_some(),
            _ => self.active_hooks.contains_key(&hook_type),
        }
    }

    /// Returns true if any hooks are currently active.
    pub fn has_any_hooks(&self) -> bool {
        !self.active_hooks.is_empty()
            || self.teleport_hook.is_some()
            || self.mouseless_hook.is_some()
    }

    // ── Helper: read 8-byte pointer from export address ─────────────

    fn read_hook_base_addr(&self, export_name: &str, hook_name: &str) -> Result<usize> {
        let addr = self.export_addrs.get(export_name)
            .ok_or_else(|| WizWalkerError::HookNotActive(hook_name.into()))?;

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?;

        let bytes = reader.read_bytes(*addr, 8)?;
        let value = i64::from_le_bytes(bytes[..8].try_into().unwrap_or([0; 8]));

        if value == 0 {
            return Err(WizWalkerError::HookNotReady(hook_name.into()));
        }

        Ok(value as usize)
    }

    // ── Generic simple hook activation ──────────────────────────────

    fn activate_simple_hook(&mut self, hook: &dyn SimpleHook) -> Result<()> {
        let hook_type = hook.hook_type();

        if self.check_if_hook_active(hook_type) {
            return Err(WizWalkerError::HookAlreadyActive(hook_type.to_string()));
        }

        self.prepare_autobot()?;

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?
            .clone();

        // Create the autobot allocator closure.
        let instance = hook.hook(
            &reader,
            &mut |size| self.allocate_autobot(size),
        )?;

        // Store export addresses.
        for (name, addr) in &instance.export_addresses {
            self.export_addrs.insert(name.clone(), *addr);
        }

        self.active_hooks.insert(hook_type, instance);
        Ok(())
    }

    fn deactivate_simple_hook(&mut self, hook_type: HookType) -> Result<()> {
        if !self.check_if_hook_active(hook_type) {
            return Err(WizWalkerError::HookNotActive(hook_type.to_string()));
        }

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?;

        let instance = self.active_hooks.remove(&hook_type)
            .ok_or_else(|| WizWalkerError::HookNotActive(hook_type.to_string()))?;

        // Remove export addresses.
        for (name, _) in &instance.export_addresses {
            self.export_addrs.remove(name);
        }

        instance.unhook(reader)
    }

    // ── Per-hook activation/deactivation ────────────────────────────

    pub fn activate_player_hook(&mut self) -> Result<()> {
        self.activate_simple_hook(&PlayerHook)
    }

    pub fn deactivate_player_hook(&mut self) -> Result<()> {
        self.deactivate_simple_hook(HookType::Player)
    }

    pub fn read_current_player_base(&self) -> Result<usize> {
        self.read_hook_base_addr("player_struct", "Player")
    }

    pub fn activate_player_stat_hook(&mut self) -> Result<()> {
        self.activate_simple_hook(&PlayerStatHook)
    }

    pub fn deactivate_player_stat_hook(&mut self) -> Result<()> {
        self.deactivate_simple_hook(HookType::PlayerStat)
    }

    pub fn read_current_player_stat_base(&self) -> Result<usize> {
        self.read_hook_base_addr("stat_addr", "Player stat")
    }

    pub fn activate_quest_hook(&mut self) -> Result<()> {
        self.activate_simple_hook(&QuestHook)
    }

    pub fn deactivate_quest_hook(&mut self) -> Result<()> {
        self.deactivate_simple_hook(HookType::Quest)
    }

    pub fn read_current_quest_base(&self) -> Result<usize> {
        self.read_hook_base_addr("cord_struct", "Quest")
    }

    pub fn activate_client_hook(&mut self) -> Result<()> {
        self.activate_simple_hook(&ClientHook)
    }

    pub fn deactivate_client_hook(&mut self) -> Result<()> {
        self.deactivate_simple_hook(HookType::Client)
    }

    pub fn read_current_client_base(&self) -> Result<usize> {
        self.read_hook_base_addr("current_client_addr", "Client")
    }

    pub fn activate_root_window_hook(&mut self) -> Result<()> {
        self.activate_simple_hook(&RootWindowHook)
    }

    pub fn deactivate_root_window_hook(&mut self) -> Result<()> {
        self.deactivate_simple_hook(HookType::RootWindow)
    }

    pub fn read_current_root_window_base(&self) -> Result<usize> {
        self.read_hook_base_addr("current_root_window_addr", "Root window")
    }

    pub fn activate_render_context_hook(&mut self) -> Result<()> {
        self.activate_simple_hook(&RenderContextHook)
    }

    pub fn deactivate_render_context_hook(&mut self) -> Result<()> {
        self.deactivate_simple_hook(HookType::RenderContext)
    }

    pub fn read_current_render_context_base(&self) -> Result<usize> {
        self.read_hook_base_addr("current_render_context_addr", "Render context")
    }

    // ── Movement Teleport Hook ─────────────────────────────────────

    pub fn activate_movement_teleport_hook(&mut self) -> Result<()> {
        if self.teleport_hook.is_some() {
            return Err(WizWalkerError::HookAlreadyActive("Movement teleport".into()));
        }

        self.prepare_autobot()?;

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?
            .clone();

        let hook = MovementTeleportHook;
        let instance = hook.hook(
            &reader,
            &mut |size| self.allocate_autobot(size),
        )?;

        // Store export addresses.
        for (name, addr) in &instance.base.export_addresses {
            self.export_addrs.insert(name.clone(), *addr);
        }

        self.teleport_hook = Some(instance);
        Ok(())
    }

    pub fn deactivate_movement_teleport_hook(&mut self) -> Result<()> {
        let instance = self.teleport_hook.take()
            .ok_or_else(|| WizWalkerError::HookNotActive("Movement teleport".into()))?;

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?;

        // Remove export addresses.
        for (name, _) in &instance.base.export_addresses {
            self.export_addrs.remove(name);
        }

        instance.unhook(reader)
    }

    pub fn read_teleport_helper(&self) -> Result<usize> {
        let addr = self.export_addrs.get("teleport_helper")
            .ok_or_else(|| WizWalkerError::HookNotActive("Movement teleport".into()))?;
        Ok(*addr)
    }

    // ── Mouseless Cursor Hook ──────────────────────────────────────

    pub fn activate_mouseless_cursor_hook(&mut self) -> Result<()> {
        if self.mouseless_hook.is_some() {
            return Err(WizWalkerError::HookAlreadyActive("Mouseless cursor".into()));
        }

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?
            .clone();

        let hook = MouselessCursorMoveHook;
        let instance = hook.hook(&reader)?;

        // Store export addresses.
        for (name, addr) in &instance.base.export_addresses {
            self.export_addrs.insert(name.clone(), *addr);
        }

        self.mouseless_hook = Some(instance);

        // Initialize mouse position to (0, 0).
        self.write_mouse_position(0, 0)?;

        Ok(())
    }

    pub fn deactivate_mouseless_cursor_hook(&mut self) -> Result<()> {
        let instance = self.mouseless_hook.take()
            .ok_or_else(|| WizWalkerError::HookNotActive("Mouseless cursor".into()))?;

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?;

        // Remove export addresses.
        for (name, _) in &instance.base.export_addresses {
            self.export_addrs.remove(name);
        }

        instance.unhook(reader)
    }

    pub fn write_mouse_position(&self, x: i32, y: i32) -> Result<()> {
        let addr = self.export_addrs.get("mouse_position")
            .ok_or_else(|| WizWalkerError::HookNotActive("Mouseless cursor".into()))?;

        let reader = self.reader.as_ref()
            .ok_or_else(|| WizWalkerError::Other("Reader not attached".into()))?;

        let mut packed = Vec::with_capacity(8);
        packed.extend_from_slice(&x.to_le_bytes());
        packed.extend_from_slice(&y.to_le_bytes());

        reader.write_bytes(*addr, &packed)
    }

    // ── Activate all hooks ─────────────────────────────────────────

    /// Activate all hooks except mouseless cursor.
    ///
    /// # Python equivalent
    /// `HookHandler.activate_all_hooks()`
    pub fn activate_all_hooks(&mut self) -> Result<()> {
        self.activate_player_hook()?;
        self.activate_quest_hook()?;
        self.activate_player_stat_hook()?;
        self.activate_client_hook()?;
        self.activate_root_window_hook()?;
        self.activate_render_context_hook()?;
        self.activate_movement_teleport_hook()?;
        Ok(())
    }

    // ── Close ──────────────────────────────────────────────────────

    /// Close all hooks, restore autobot function, release reader.
    ///
    /// # Python equivalent
    /// `HookHandler.close()`
    pub fn close(&mut self) {
        if let Some(reader) = &self.reader {
            // Unhook all standard hooks.
            let hook_types: Vec<HookType> = self.active_hooks.keys().copied().collect();
            for hook_type in hook_types {
                if let Some(instance) = self.active_hooks.remove(&hook_type) {
                    let _ = instance.unhook(reader);
                }
            }

            // Unhook teleport.
            if let Some(instance) = self.teleport_hook.take() {
                let _ = instance.unhook(reader);
            }

            // Unhook mouseless.
            if let Some(instance) = self.mouseless_hook.take() {
                let _ = instance.unhook(reader);
            }
        }

        // Restore autobot.
        let _ = self.rewrite_autobot();

        // Clear state.
        self.active_hooks.clear();
        self.export_addrs.clear();
        self.autobot_address = None;
        self.autobot_original_bytes = None;
        self.autobot_pos = 0;
        self.reader = None;
    }
}
