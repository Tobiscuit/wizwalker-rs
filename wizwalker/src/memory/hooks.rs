//! Memory hook implementations for injecting shellcode into Wizard101.
//!
//! Each hook:
//! 1. Pattern scans to find a jump target in `WizardGraphicalClient.exe`
//! 2. Allocates "export" memory where the hooks writes captured game state
//! 3. Generates x86-64 shellcode that captures data to the export address
//! 4. Writes a JMP at the jump target → redirects to our shellcode
//! 5. Shellcode executes, then JMPs back to the original code
//!
//! # Python equivalent
//! `wizwalker/memory/hooks.py` — `MemoryHook`, `SimpleHook`, and 8 concrete hooks.




use crate::errors::{Result, WizWalkerError};
use crate::memory::process_reader::ProcessMemoryReader;
use crate::memory::reader::MemoryReader;

// ─── Hook Type Enum ────────────────────────────────────────────────────

/// Identifies which hook type is active, used as a key in the handler's active hooks map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    Player,
    PlayerStat,
    Quest,
    Client,
    RootWindow,
    RenderContext,
    MovementTeleport,
    MouselessCursor,
    DanceGameMoves,
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::Player => write!(f, "Player"),
            HookType::PlayerStat => write!(f, "Player stat"),
            HookType::Quest => write!(f, "Quest"),
            HookType::Client => write!(f, "Client"),
            HookType::RootWindow => write!(f, "Root window"),
            HookType::RenderContext => write!(f, "Render context"),
            HookType::MovementTeleport => write!(f, "Movement teleport"),
            HookType::MouselessCursor => write!(f, "Mouseless cursor"),
            HookType::DanceGameMoves => write!(f, "Dance game moves"),
        }
    }
}

// ─── Hook Instance ─────────────────────────────────────────────────────

/// A live hook instance: tracks addresses and original bytes for cleanup.
///
/// Created when a hook is activated, consumed when it's deactivated.
#[derive(Clone)]
pub struct HookInstance {
    pub hook_type: HookType,
    /// Address where the JMP instruction was written.
    pub jump_address: usize,
    /// Address where the hook shellcode lives (in autobot or allocated memory).
    pub hook_address: usize,
    /// Original bytes at the jump address (restored on unhook).
    pub jump_original_bytes: Vec<u8>,
    /// Addresses of exported values (where the hook writes captured data).
    /// Key = export name, Value = allocated address.
    pub export_addresses: Vec<(String, usize)>,
    /// Addresses that were allocated (for deallocation on unhook).
    pub allocated_addresses: Vec<usize>,
}

impl HookInstance {
    /// Restore original bytes and free allocated memory.
    pub fn unhook(&self, reader: &ProcessMemoryReader) -> Result<()> {
        // Restore original bytes at the jump address.
        reader.write_bytes(self.jump_address, &self.jump_original_bytes)?;

        // Free allocated export addresses.
        for addr in &self.allocated_addresses {
            // Best-effort free — don't fail if one doesn't free.
            let _ = reader.free(*addr);
        }

        Ok(())
    }
}

// ─── Simple Hook Trait ─────────────────────────────────────────────────

/// Describes a "simple" hook that follows the standard JMP + shellcode pattern.
///
/// # Python equivalent
/// `hooks.py` — `SimpleHook` class.
pub trait SimpleHook {
    /// The hook type identifier.
    fn hook_type(&self) -> HookType;

    /// Byte pattern to find the jump target in the game.
    fn pattern(&self) -> &[u8];

    /// Module to scan (always "WizardGraphicalClient.exe" for simple hooks).
    fn module(&self) -> &str {
        "WizardGraphicalClient.exe"
    }

    /// How many bytes the JMP instruction overwrites at the jump address.
    fn instruction_length(&self) -> usize {
        5
    }

    /// Padding NOPs after the JMP instruction.
    fn noops(&self) -> usize {
        0
    }

    /// Offset to add to the pattern scan result to get the actual jump address.
    /// Default: 0. ClientHook overrides this to +1.
    fn jump_address_offset(&self) -> usize {
        0
    }

    /// Export definitions: (name, size_in_bytes).
    fn exports(&self) -> Vec<(&str, usize)>;

    /// Generate the hook shellcode given the packed export addresses.
    ///
    /// Each export address is provided as `(name, packed_8_bytes)`.
    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8>;

    /// Build the JMP bytecode to write at the jump address.
    ///
    /// This creates a relative JMP from `jump_address` to `hook_address`,
    /// followed by NOPs.
    fn build_jump_bytecode(&self, jump_address: usize, hook_address: usize) -> Vec<u8> {
        let distance = hook_address as i64 - jump_address as i64;
        let relative_jump = (distance - 5) as i32;
        let packed = relative_jump.to_le_bytes();

        let mut bytecode = vec![0xE9]; // JMP rel32
        bytecode.extend_from_slice(&packed);
        bytecode.extend(std::iter::repeat(0x90).take(self.noops()));
        bytecode
    }

    /// Full hook lifecycle: scan → allocate → generate → write.
    fn hook(
        &self,
        reader: &ProcessMemoryReader,
        autobot_alloc: &mut dyn FnMut(usize) -> Result<usize>,
    ) -> Result<HookInstance> {
        // 1. Pattern scan to find the jump address.
        let scan_results = reader.pattern_scan(self.pattern(), Some(self.module()), false)?;
        let jump_address = scan_results[0] + self.jump_address_offset();

        // 2. Allocate hook space in the autobot function.
        let hook_address = autobot_alloc(50)?;

        // 3. Allocate export addresses and pack them.
        let mut export_addresses = Vec::new();
        let mut allocated_addresses = Vec::new();
        let mut packed_exports = Vec::new();

        for (name, size) in self.exports() {
            let addr = reader.allocate(size)?;
            allocated_addresses.push(addr);
            export_addresses.push((name.to_string(), addr));
            packed_exports.push((name, (addr as u64).to_le_bytes()));
        }

        // 4. Generate the hook bytecode.
        let mut hook_bytecode = self.generate_bytecode(&packed_exports);

        // 5. Append return JMP back to original code.
        let return_addr = jump_address + self.instruction_length();
        let relative_return = return_addr as i64 - (hook_address as i64 + hook_bytecode.len() as i64) - 5;
        hook_bytecode.push(0xE9); // JMP rel32
        hook_bytecode.extend_from_slice(&(relative_return as i32).to_le_bytes());

        // 6. Save original bytes at the jump address.
        let jump_bytecode = self.build_jump_bytecode(jump_address, hook_address);
        let jump_original_bytes = reader.read_bytes(jump_address, jump_bytecode.len())?;

        // 7. Write hook bytecode, then JMP.
        reader.write_bytes(hook_address, &hook_bytecode)?;
        reader.write_bytes(jump_address, &jump_bytecode)?;

        Ok(HookInstance {
            hook_type: self.hook_type(),
            jump_address,
            hook_address,
            jump_original_bytes,
            export_addresses,
            allocated_addresses,
        })
    }
}

// ─── Concrete Hook Implementations ─────────────────────────────────────

// ── PlayerHook ──────────────────────────────────────────────────────────
/// Captures the player struct base address.
///
/// Python class: `PlayerHook(SimpleHook)`
/// Pattern: `\xF2\x0F\x10\x40\x58\xF2`
/// Export: `player_struct` (8 bytes)
pub struct PlayerHook;

impl SimpleHook for PlayerHook {
    fn hook_type(&self) -> HookType { HookType::Player }

    fn pattern(&self) -> &[u8] {
        &[0xF2, 0x0F, 0x10, 0x40, 0x58, 0xF2]
    }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("player_struct", 8)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (hooks.py:211-224):
        // push rcx
        // mov ecx,[rax+474]           ; check object type
        // cmp ecx,08                  ; is player?
        // pop rcx
        // jne 10 down                 ; skip if not player
        // mov(abs) [export], rax      ; store player struct
        // movsd xmm0,[rax+58]         ; original code
        let mut bc = Vec::new();
        bc.push(0x51);                                                 // push rcx
        bc.extend_from_slice(&[0x8B, 0x88, 0x74, 0x04, 0x00, 0x00]); // mov ecx,[rax+474]
        bc.extend_from_slice(&[0x83, 0xF9, 0x08]);                    // cmp ecx,08
        bc.push(0x59);                                                 // pop rcx
        bc.extend_from_slice(&[0x0F, 0x85, 0x0A, 0x00, 0x00, 0x00]); // jne 10 down
        bc.extend_from_slice(&[0x48, 0xA3]);                           // mov(abs) [addr], rax
        bc.extend_from_slice(&export_addr);
        bc.extend_from_slice(&[0xF2, 0x0F, 0x10, 0x40, 0x58]);       // movsd xmm0,[rax+58]
        bc
    }
}

// ── PlayerStatHook ──────────────────────────────────────────────────────
/// Captures the game stats base address.
///
/// Python class: `PlayerStatHook(SimpleHook)`
/// Pattern: `\x2B\xD8\xB8....\x0F\x49\xC3\x48\x83\xC4\x20\x5B\xC3`
/// Export: `stat_addr` (8 bytes)
pub struct PlayerStatHook;

impl SimpleHook for PlayerStatHook {
    fn hook_type(&self) -> HookType { HookType::PlayerStat }

    fn pattern(&self) -> &[u8] {
        &[0x2B, 0xD8, 0xB8, 0x2E, 0x2E, 0x2E, 0x2E, 0x0F, 0x49, 0xC3,
          0x48, 0x83, 0xC4, 0x20, 0x5B, 0xC3]
    }

    fn instruction_length(&self) -> usize { 7 }
    fn noops(&self) -> usize { 2 }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("stat_addr", 8)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (hooks.py:234-246):
        // push rax
        // mov rax, rcx
        // mov qword ptr [stat_export], rax
        // pop rax
        // sub ebx, eax                ; original code
        // mov eax, 0                  ; original code
        let mut bc = Vec::new();
        bc.push(0x50);                                     // push rax
        bc.extend_from_slice(&[0x48, 0x89, 0xC8]);        // mov rax, rcx
        bc.extend_from_slice(&[0x48, 0xA3]);               // mov [stat_export], rax
        bc.extend_from_slice(&export_addr);
        bc.push(0x58);                                     // pop rax
        bc.extend_from_slice(&[0x2B, 0xD8]);               // sub ebx, eax
        bc.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x00]); // mov eax, 0
        bc
    }
}

// ── QuestHook ───────────────────────────────────────────────────────────
/// Captures the quest coordinate struct address.
///
/// Python class: `QuestHook(SimpleHook)`
/// Pattern: `\xF3\x41\x0F\x10.\xFC\x0C\x00\x00\xF3\x0F\x11`
/// Export: `cord_struct` (4 bytes — just the address pointer)
pub struct QuestHook;

impl SimpleHook for QuestHook {
    fn hook_type(&self) -> HookType { HookType::Quest }

    fn pattern(&self) -> &[u8] {
        &[0xF3, 0x41, 0x0F, 0x10, 0x2E, 0xFC, 0x0C, 0x00, 0x00, 0xF3, 0x0F, 0x11]
    }

    fn noops(&self) -> usize { 4 }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("cord_struct", 4)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (hooks.py:254-265):
        // push rax
        // lea rax,[r15+00000CFC]
        // mov [export],rax
        // pop rax
        // movss xmm0,[r15+CFC]       ; original code
        let mut bc = Vec::new();
        bc.push(0x50);                                                         // push rax
        bc.extend_from_slice(&[0x49, 0x8D, 0x87, 0xFC, 0x0C, 0x00, 0x00]);   // lea rax,[r15+CFC]
        bc.extend_from_slice(&[0x48, 0xA3]);                                   // mov [export],rax
        bc.extend_from_slice(&export_addr);
        bc.push(0x58);                                                         // pop rax
        bc.extend_from_slice(&[0xF3, 0x41, 0x0F, 0x10, 0x87, 0xFC, 0x0C, 0x00, 0x00]); // original
        bc
    }
}

// ── ClientHook ──────────────────────────────────────────────────────────
/// Captures the current client object address.
///
/// Python class: `ClientHook(SimpleHook)`
/// Pattern starts with 0x18 but jump_address is +1 from the scan result.
/// Export: `current_client_addr` (8 bytes)
pub struct ClientHook;

impl SimpleHook for ClientHook {
    fn hook_type(&self) -> HookType { HookType::Client }

    fn pattern(&self) -> &[u8] {
        &[0x18, 0x48, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
          0x48, 0x8B, 0x7C, 0x24, 0x38, 0x48, 0x85, 0xFF,
          0x74, 0x29, 0x8B, 0xC6, 0xF0, 0x0F, 0xC1, 0x47,
          0x08, 0x83, 0xF8, 0x01, 0x75, 0x1D,
          0x48, 0x8B, 0x07, 0x48, 0x8B, 0xCF, 0xFF, 0x50,
          0x08, 0xF0, 0x0F, 0xC1, 0x77, 0x0C]
    }

    fn instruction_length(&self) -> usize { 7 }
    fn noops(&self) -> usize { 2 }

    /// The 0x18 byte at the start is tacked on — actual jump is +1.
    fn jump_address_offset(&self) -> usize { 1 }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("current_client_addr", 8)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (hooks.py:285-297):
        // push rax
        // mov rax,rdi
        // mov [current_client], rax
        // pop rax
        // mov rbx,[rbx+1C0]           ; original instruction
        let mut bc = Vec::new();
        bc.push(0x50);                                                         // push rax
        bc.extend_from_slice(&[0x48, 0x8B, 0xC7]);                           // mov rax,rdi
        bc.extend_from_slice(&[0x48, 0xA3]);                                   // mov [current_client], rax
        bc.extend_from_slice(&export_addr);
        bc.push(0x58);                                                         // pop rax
        bc.extend_from_slice(&[0x48, 0x8B, 0x9B, 0xC0, 0x01, 0x00, 0x00]);   // original instruction
        bc
    }
}

// ── RootWindowHook ──────────────────────────────────────────────────────
/// Captures the root UI window address.
///
/// Python class: `RootWindowHook(SimpleHook)`
/// Export: `current_root_window_addr` (8 bytes)
pub struct RootWindowHook;

impl SimpleHook for RootWindowHook {
    fn hook_type(&self) -> HookType { HookType::RootWindow }

    fn pattern(&self) -> &[u8] {
        &[0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
          0x48, 0x8B, 0x01,
          0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
          0xFF, 0x50, 0x70, 0x84]
    }

    fn instruction_length(&self) -> usize { 7 }
    fn noops(&self) -> usize { 2 }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("current_root_window_addr", 8)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (hooks.py:306-317):
        // push rax
        // mov rax,[r13+D8]
        // mov [current_root_window_addr], rax
        // pop rax
        // mov rcx,[r13+D8]            ; original instruction
        let mut bc = Vec::new();
        bc.push(0x50);                                                         // push rax
        bc.extend_from_slice(&[0x49, 0x8B, 0x85, 0xD8, 0x00, 0x00, 0x00]);   // mov rax,[r13+D8]
        bc.extend_from_slice(&[0x48, 0xA3]);                                   // mov [addr], rax
        bc.extend_from_slice(&export_addr);
        bc.push(0x58);                                                         // pop rax
        bc.extend_from_slice(&[0x49, 0x8B, 0x8D, 0xD8, 0x00, 0x00, 0x00]);   // original instruction
        bc
    }
}

// ── RenderContextHook ───────────────────────────────────────────────────
/// Captures the render context address.
///
/// Python class: `RenderContextHook(SimpleHook)`
/// Export: `current_render_context_addr` (8 bytes)
pub struct RenderContextHook;

impl SimpleHook for RenderContextHook {
    fn hook_type(&self) -> HookType { HookType::RenderContext }

    fn pattern(&self) -> &[u8] {
        &[0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
          0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
          0xF3, 0x41, 0x0F, 0x10, 0x28,
          0xF3, 0x0F, 0x10, 0x56, 0x04,
          0x48, 0x63, 0xC1]
    }

    fn instruction_length(&self) -> usize { 9 }
    fn noops(&self) -> usize { 4 }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("current_render_context_addr", 8)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (hooks.py:326-337):
        // push rax
        // mov rax,rbx
        // mov [current_render_context_addr],rax
        // pop rax
        // movss xmm1,[rbx+98]        ; original instruction
        let mut bc = Vec::new();
        bc.push(0x50);                                                         // push rax
        bc.extend_from_slice(&[0x48, 0x89, 0xD8]);                           // mov rax,rbx
        bc.extend_from_slice(&[0x48, 0xA3]);                                   // mov [addr],rax
        bc.extend_from_slice(&export_addr);
        bc.push(0x58);                                                         // pop rax
        bc.extend_from_slice(&[0xF3, 0x44, 0x0F, 0x10, 0x8B, 0x98, 0x00, 0x00, 0x00]); // original
        bc
    }
}

// ── MovementTeleportHook ────────────────────────────────────────────────
/// Enables teleportation by intercepting the movement update function.
///
/// This hook is more complex than the simple hooks:
/// - It has a prehook phase that NOPs out collision JE instructions
/// - Its shellcode compares the target object to the local client object
/// - It writes position data and a "should update" bool
///
/// Python class: `MovementTeleportHook(SimpleHook)`
/// Export: `teleport_helper` (21 bytes = 12 position + 1 bool + 8 target addr)
pub struct MovementTeleportHook;

impl MovementTeleportHook {
    /// The teleport hook's byte pattern.
    pub const PATTERN: &[u8] = &[
        0x57, 0x48, 0x83, 0xEC, 0x2E, 0x48, 0x8B, 0x99,
        0x2E, 0x2E, 0x2E, 0x2E, 0x48, 0x85, 0xDB, 0x74,
        0x2E, 0x4C, 0x8B, 0x43, 0x2E, 0x48, 0x8B, 0x5B,
        0x2E, 0x48, 0x85, 0xDB, 0x74, 0x2E, 0xF0, 0xFF,
        0x43, 0x2E, 0x4D, 0x85, 0xC0, 0x74, 0x2E, 0xF2,
        0x0F, 0x10, 0x02, 0xF2, 0x41, 0x0F, 0x11, 0x40,
        0x2E, 0x8B, 0x42, 0x2E, 0x41, 0x89, 0x40, 0x2E,
        0x41, 0xC6, 0x80, 0x2E,
    ];

    /// Collision JE patterns for prehook patching.
    const INSIDE_EVENT_JE_PATTERN: &[u8] = &[
        0x74, 0x2E, 0xF3, 0x0F, 0x10, 0x55, 0xA8,
    ];

    const EVENT_DISPATCH_JE_PATTERN: &[u8] = &[
        0x74, 0x2E, 0xF3, 0x0F, 0x10, 0x44, 0x24, 0x54, 0xF3, 0x0F,
    ];

    /// Movement state pattern for finding JE forward/backwards addresses.
    const MOVEMENT_STATE_PATTERN: &[u8] = &[
        0x8B, 0x5F, 0x70, 0xF3,
    ];

    /// Full hook lifecycle for the teleport hook (custom, not using SimpleHook trait).
    pub fn hook(
        &self,
        reader: &ProcessMemoryReader,
        autobot_alloc: &mut dyn FnMut(usize) -> Result<usize>,
    ) -> Result<MovementTeleportHookInstance> {
        // 1. Pattern scan for jump address.
        let scan_results = reader.pattern_scan(Self::PATTERN, Some("WizardGraphicalClient.exe"), false)?;
        let jump_address = scan_results[0];

        // 2. Allocate hook space (200 bytes — this hook is bigger).
        let hook_address = autobot_alloc(200)?;

        // 3. Allocate teleport helper export (21 bytes).
        // Layout: [x: f32][y: f32][z: f32][should_update: bool][target_addr: u64]
        //          0       4       8       12                   13
        let teleport_helper_addr = reader.allocate(21)?;
        let packed_export = (teleport_helper_addr as u64).to_le_bytes();

        // 4. Find JE instruction addresses for forward/backwards movement.
        let movement_state_results = reader.pattern_scan(
            Self::MOVEMENT_STATE_PATTERN,
            Some("WizardGraphicalClient.exe"),
            false,
        )?;
        let movement_state_addr = movement_state_results[0];
        let je_forward = movement_state_addr + 15;
        let je_backwards = movement_state_addr + 24;

        // 5. Read current JE bytes (for restoration later).
        let old_je_forward_bytes = reader.read_bytes(je_forward, 8)?;
        let old_je_backwards_bytes = reader.read_bytes(je_backwards, 8)?;

        // 6. Prehook: NOP out collision JE instructions.
        let inside_event_results = reader.pattern_scan(
            Self::INSIDE_EVENT_JE_PATTERN,
            Some("WizardGraphicalClient.exe"),
            false,
        )?;
        let inside_event_je_addr = inside_event_results[0];
        let old_inside_event_bytes = reader.read_bytes(inside_event_je_addr, 2)?;

        let event_dispatch_results = reader.pattern_scan(
            Self::EVENT_DISPATCH_JE_PATTERN,
            Some("WizardGraphicalClient.exe"),
            false,
        )?;
        let event_dispatch_je_addr = event_dispatch_results[0];
        let old_event_dispatch_bytes = reader.read_bytes(event_dispatch_je_addr, 2)?;

        // NOP the collision JEs.
        reader.write_bytes(inside_event_je_addr, &[0x90, 0x90])?;
        reader.write_bytes(event_dispatch_je_addr, &[0x90, 0x90])?;

        // 7. Set page protection for JE addresses (read/write/execute).
        // In Python this uses VirtualProtectEx; we'll handle this via write_bytes
        // which should work for in-game code sections that are already RWX
        // after the autobot NOP-out.

        // 8. Generate hook bytecode.
        // Compute packed addresses for shellcode.
        let mut packed_should_update = packed_export;
        // should_update is at offset 12 within teleport_helper
        let should_update_addr = teleport_helper_addr + 12;
        let packed_should_update_addr = (should_update_addr as u64).to_le_bytes();

        let z_addr = teleport_helper_addr + 8;
        let packed_z = (z_addr as u64).to_le_bytes();

        let target_obj_addr = teleport_helper_addr + 13;
        let packed_target_addr = (target_obj_addr as u64).to_le_bytes();

        let packed_je_forward = (je_forward as u64).to_le_bytes();
        let packed_je_backwards = (je_backwards as u64).to_le_bytes();

        let mut bc = Vec::new();

        // push rax
        bc.push(0x50);
        // mov rax,[target_object_addr]
        bc.extend_from_slice(&[0x48, 0xA1]);
        bc.extend_from_slice(&packed_target_addr);
        // cmp rcx,rax
        bc.extend_from_slice(&[0x48, 0x39, 0xC1]);
        // pop rax
        bc.push(0x58);
        // je down 5 (local client object)
        bc.extend_from_slice(&[0x0F, 0x84, 0x05, 0x00, 0x00, 0x00]);
        // jmp (not local client object) — skip to end
        bc.extend_from_slice(&[0xE9, 0x6E, 0x00, 0x00, 0x00]);
        // push rax
        bc.push(0x50);
        // mov al,[should_update_bool]
        bc.push(0xA0);
        bc.extend_from_slice(&packed_should_update_addr);
        // test al,al
        bc.extend_from_slice(&[0x84, 0xC0]);
        // pop rax
        bc.push(0x58);
        // jne 5 (should_update is True)
        bc.extend_from_slice(&[0x0F, 0x85, 0x05, 0x00, 0x00, 0x00]);
        // jmp (should_update is False) — skip to end
        bc.extend_from_slice(&[0xE9, 0x56, 0x00, 0x00, 0x00]);
        // push rax
        bc.push(0x50);
        // mov rax, [new_pos]
        bc.extend_from_slice(&[0x48, 0xA1]);
        bc.extend_from_slice(&packed_export);
        // mov [rdx], rax
        bc.extend_from_slice(&[0x48, 0x89, 0x02]);
        // mov eax, [z_addr]
        bc.push(0xA1);
        bc.extend_from_slice(&packed_z);
        // mov [rdx+08], eax
        bc.extend_from_slice(&[0x89, 0x42, 0x08]);
        // mov rax, 0 (clear should_update)
        bc.extend_from_slice(&[0x48, 0xB8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // mov [should_update_bool], al
        bc.push(0xA2);
        bc.extend_from_slice(&packed_should_update_addr);
        // Restore JE bytes: mov rax, old_je_forward_bytes; mov [je_forward], rax
        bc.extend_from_slice(&[0x48, 0xB8]);
        bc.extend_from_slice(&old_je_forward_bytes);
        bc.extend_from_slice(&[0x48, 0xA3]);
        bc.extend_from_slice(&packed_je_forward);
        // Restore JE bytes: mov rax, old_je_backwards_bytes; mov [je_backwards], rax
        bc.extend_from_slice(&[0x48, 0xB8]);
        bc.extend_from_slice(&old_je_backwards_bytes);
        bc.extend_from_slice(&[0x48, 0xA3]);
        bc.extend_from_slice(&packed_je_backwards);
        // pop rax
        bc.push(0x58);
        // push rdi (original bytes)
        bc.push(0x57);
        // sub rsp,20 (original bytes)
        bc.extend_from_slice(&[0x48, 0x83, 0xEC, 0x20]);

        // 9. Append return JMP.
        let instruction_length = 5;
        let return_addr = jump_address + instruction_length;
        let relative_return = return_addr as i64 - (hook_address as i64 + bc.len() as i64) - 5;
        bc.push(0xE9);
        bc.extend_from_slice(&(relative_return as i32).to_le_bytes());

        // 10. Build jump bytecode.
        let distance = hook_address as i64 - jump_address as i64;
        let relative_jump = (distance - 5) as i32;
        let mut jump_bytecode = vec![0xE9];
        jump_bytecode.extend_from_slice(&relative_jump.to_le_bytes());

        let jump_original_bytes = reader.read_bytes(jump_address, jump_bytecode.len())?;

        // 11. Write it all.
        reader.write_bytes(hook_address, &bc)?;
        reader.write_bytes(jump_address, &jump_bytecode)?;

        Ok(MovementTeleportHookInstance {
            base: HookInstance {
                hook_type: HookType::MovementTeleport,
                jump_address,
                hook_address,
                jump_original_bytes,
                export_addresses: vec![("teleport_helper".to_string(), teleport_helper_addr)],
                allocated_addresses: vec![teleport_helper_addr],
            },
            je_forward,
            je_backwards,
            old_je_forward_bytes,
            old_je_backwards_bytes,
            inside_event_je_addr,
            old_inside_event_bytes,
            event_dispatch_je_addr,
            old_event_dispatch_bytes,
        })
    }
}

/// Extended hook instance for MovementTeleportHook with JE restoration data.
#[derive(Clone)]
pub struct MovementTeleportHookInstance {
    pub base: HookInstance,
    pub je_forward: usize,
    pub je_backwards: usize,
    pub old_je_forward_bytes: Vec<u8>,
    pub old_je_backwards_bytes: Vec<u8>,
    pub inside_event_je_addr: usize,
    pub old_inside_event_bytes: Vec<u8>,
    pub event_dispatch_je_addr: usize,
    pub old_event_dispatch_bytes: Vec<u8>,
}

impl MovementTeleportHookInstance {
    /// Unhook: restore JE bytes, collision bytes, then base unhook.
    pub fn unhook(&self, reader: &ProcessMemoryReader) -> Result<()> {
        // Restore JE forward/backwards bytes.
        reader.write_bytes(self.je_forward, &self.old_je_forward_bytes)?;
        reader.write_bytes(self.je_backwards, &self.old_je_backwards_bytes)?;

        // Restore collision JE bytes.
        reader.write_bytes(self.inside_event_je_addr, &self.old_inside_event_bytes)?;
        reader.write_bytes(self.event_dispatch_je_addr, &self.old_event_dispatch_bytes)?;

        // Base unhook (restore jump, free exports).
        self.base.unhook(reader)
    }
}

// ── MouselessCursorMoveHook ─────────────────────────────────────────────
/// Hooks `GetCursorPos` from user32.dll to inject custom mouse position.
///
/// Uses a different injection mechanism than the autobot-based hooks:
/// instead of using the game's autobot function for shellcode storage,
/// it hijacks `user32.dll`'s `GetClassInfoExA` function.
///
/// Python class: `MouselessCursorMoveHook(User32GetClassInfoBaseHook)`
pub struct MouselessCursorMoveHook;

impl MouselessCursorMoveHook {
    /// GetClassInfoExA function size we can safely overwrite.
    const GETCLASSINFOEX_SIZE: usize = 1200;

    /// Pattern to find `GetClassInfoExA` autobot in user32.dll's loaded image.
    const GETCLASSINFOEX_PATTERN: &[u8] = &[
        0x48, 0x89, 0x5C, 0x24, 0x20, 0x55, 0x56, 0x57,
        0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57,
        0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
        0x48, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E,
        0x48, 0x8B, 0x05, 0x2E, 0x2E, 0x2E, 0x2E,
        0x48, 0x33, 0xC4, 0x2E, 0x2E,
        0x48, 0x8B, 0xDA, 0x4C,
    ];

    /// Bool toggle patterns for cursor visibility.
    const BOOL_ONE_PATTERN: &[u8] = &[
        0x00, 0xFF, 0x50, 0x18, 0x66, 0xC7,
    ];

    const BOOL_TWO_PATTERN: &[u8] = &[
        0xC6, 0x86, 0x2E, 0x2E, 0x2E, 0x00, 0x00, 0x33, 0xFF, 0x89,
    ];

    /// Full hook lifecycle for the mouseless cursor hook.
    pub fn hook(
        &self,
        reader: &ProcessMemoryReader,
    ) -> Result<MouselessCursorHookInstance> {
        // 1. Find GetClassInfoExA in user32.dll for shellcode storage.
        let getclassinfo_addr = Self::get_symbol_address(reader, "user32.dll", "GetClassInfoExA")?;

        // Save original bytes so we can restore on unhook.
        let original_bytes = reader.read_bytes(getclassinfo_addr, Self::GETCLASSINFOEX_SIZE)?;

        // NOP the entire function to make room for shellcode.
        reader.write_bytes(getclassinfo_addr, &vec![0x00; Self::GETCLASSINFOEX_SIZE])?;

        // 2. Find GetCursorPos (this is where we write the JMP).
        let get_cursor_pos_addr = Self::get_symbol_address(reader, "user32.dll", "GetCursorPos")?;

        // 3. Allocate mouse position storage (8 bytes = x:i32 + y:i32).
        let mouse_pos_addr = reader.allocate(8)?;

        // 4. Generate hook shellcode.
        let packed_mouse_pos = (mouse_pos_addr as u64).to_le_bytes();
        let hook_address = getclassinfo_addr; // Write shellcode at start of GetClassInfoExA.

        let mut bc = Vec::new();
        bc.push(0x50);                                     // push rax
        bc.extend_from_slice(&[0x48, 0xA1]);               // mov rax, [mouse_pos]
        bc.extend_from_slice(&packed_mouse_pos);
        bc.extend_from_slice(&[0x48, 0x89, 0x01]);        // mov [rcx], rax
        bc.push(0x58);                                     // pop rax
        bc.push(0xC3);                                     // ret

        // 5. Build JMP bytecode from GetCursorPos to our shellcode.
        let distance = hook_address as i64 - get_cursor_pos_addr as i64;
        let relative_jump = (distance - 5) as i32;
        let mut jump_bytecode = vec![0xE9]; // JMP rel32
        jump_bytecode.extend_from_slice(&relative_jump.to_le_bytes());

        let jump_original_bytes = reader.read_bytes(get_cursor_pos_addr, jump_bytecode.len())?;

        // 6. Write shellcode and JMP.
        reader.write_bytes(hook_address, &bc)?;
        reader.write_bytes(get_cursor_pos_addr, &jump_bytecode)?;

        // 7. Posthook: find and toggle bool addresses, NOP SetCursorPos.
        let bool_one_results = reader.pattern_scan(
            Self::BOOL_ONE_PATTERN,
            Some("WizardGraphicalClient.exe"),
            false,
        )?;
        let bool_one_addr = bool_one_results[0];

        let bool_two_results = reader.pattern_scan(
            Self::BOOL_TWO_PATTERN,
            Some("WizardGraphicalClient.exe"),
            false,
        )?;
        // Bool two is 6 bytes after the pattern match.
        let bool_two_addr = bool_two_results[0] + 6;

        // Set toggle bools to 1.
        reader.write_bytes(bool_one_addr, &[0x01])?;
        reader.write_bytes(bool_two_addr, &[0x01])?;

        // NOP SetCursorPos (ret + 5 nops).
        let set_cursor_pos_addr = Self::get_symbol_address(reader, "user32.dll", "SetCursorPos")?;
        let set_cursor_pos_original = reader.read_bytes(set_cursor_pos_addr, 6)?;
        reader.write_bytes(set_cursor_pos_addr, &[0xC3, 0x90, 0x90, 0x90, 0x90, 0x90])?;

        Ok(MouselessCursorHookInstance {
            base: HookInstance {
                hook_type: HookType::MouselessCursor,
                jump_address: get_cursor_pos_addr,
                hook_address,
                jump_original_bytes,
                export_addresses: vec![("mouse_position".to_string(), mouse_pos_addr)],
                allocated_addresses: vec![mouse_pos_addr],
            },
            getclassinfo_addr,
            getclassinfo_original_bytes: original_bytes,
            bool_one_addr,
            bool_two_addr,
            set_cursor_pos_addr,
            set_cursor_pos_original_bytes: set_cursor_pos_original,
        })
    }

    /// Resolve a symbol address from a loaded DLL in the target process.
    ///
    /// Uses `GetProcAddress` on the local copy (since system DLLs are loaded
    /// at the same base across processes on Windows), then adjusts for the
    /// remote module base.
    fn get_symbol_address(
        reader: &ProcessMemoryReader,
        module_name: &str,
        symbol_name: &str,
    ) -> Result<usize> {
        use std::ffi::CString;
        use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
        use windows::core::PCSTR;

        let module_cstr = CString::new(module_name)
            .map_err(|e| WizWalkerError::Other(format!("Invalid module name: {}", e)))?;
        let symbol_cstr = CString::new(symbol_name)
            .map_err(|e| WizWalkerError::Other(format!("Invalid symbol name: {}", e)))?;

        // Get the local module handle (system DLLs like user32.dll are loaded).
        let local_handle = unsafe {
            GetModuleHandleA(PCSTR::from_raw(module_cstr.as_ptr() as *const u8))
        }.map_err(|e| WizWalkerError::Other(format!(
            "GetModuleHandleA('{}') failed: {}", module_name, e
        )))?;

        // Get the symbol's address in our local process.
        let local_addr = unsafe {
            GetProcAddress(local_handle, PCSTR::from_raw(symbol_cstr.as_ptr() as *const u8))
        }.ok_or_else(|| WizWalkerError::Other(format!(
            "GetProcAddress('{}', '{}') returned null", module_name, symbol_name
        )))?;

        let local_addr = local_addr as usize;
        let local_base = local_handle.0 as usize;

        // Get the remote module base address via our process reader.
        let (remote_base, _size) = reader.find_module(module_name)?;

        // The symbol's offset within the module is the same local and remote.
        let offset = local_addr - local_base;
        Ok(remote_base + offset)
    }
}

/// Extended hook instance for MouselessCursorMoveHook.
#[derive(Clone)]
pub struct MouselessCursorHookInstance {
    pub base: HookInstance,
    pub getclassinfo_addr: usize,
    pub getclassinfo_original_bytes: Vec<u8>,
    pub bool_one_addr: usize,
    pub bool_two_addr: usize,
    pub set_cursor_pos_addr: usize,
    pub set_cursor_pos_original_bytes: Vec<u8>,
}

impl MouselessCursorHookInstance {
    /// Unhook: restore everything.
    pub fn unhook(&self, reader: &ProcessMemoryReader) -> Result<()> {
        // Restore GetCursorPos original bytes.
        self.base.unhook(reader)?;

        // Restore GetClassInfoExA original bytes.
        reader.write_bytes(self.getclassinfo_addr, &self.getclassinfo_original_bytes)?;

        // Restore toggle bools.
        reader.write_bytes(self.bool_one_addr, &[0x00])?;
        reader.write_bytes(self.bool_two_addr, &[0x00])?;

        // Restore SetCursorPos.
        reader.write_bytes(self.set_cursor_pos_addr, &self.set_cursor_pos_original_bytes)?;

        Ok(())
    }
}

// ── DanceGameMovesHook ─────────────────────────────────────────────────
/// Captures the current dance game move sequence for pet training.
///
/// Python class: `DanceGameMovesHook(SimpleHook)` (from deimos/dance_game_hook.py)
/// Pattern: `\x48\x8B\xF8\x48\x39\x70\x10`
/// Export: `dance_game_moves` (8 bytes — pointer to move data)
///
/// Shellcode logic:
///   mov rdi, rax              ; original: 48 8B F8
///   mov rax, [rax]            ; deref to get dance moves string ptr
///   mov [export], rax         ; store to export
///   mov rax, rdi              ; restore rax
///   cmp [rax+10], rsi         ; original: 48 39 70 10
pub struct DanceGameMovesHook;

impl SimpleHook for DanceGameMovesHook {
    fn hook_type(&self) -> HookType { HookType::DanceGameMoves }

    fn pattern(&self) -> &[u8] {
        &[0x48, 0x8B, 0xF8, 0x48, 0x39, 0x70, 0x10]
    }

    fn instruction_length(&self) -> usize { 7 }
    fn noops(&self) -> usize { 2 }

    fn exports(&self) -> Vec<(&str, usize)> {
        vec![("dance_game_moves", 8)]
    }

    fn generate_bytecode(&self, packed_exports: &[(&str, [u8; 8])]) -> Vec<u8> {
        let export_addr = packed_exports[0].1;

        // Python bytecode_generator (dance_game_hook.py:19-26):
        // mov rdi, rax                ; "\x48\x8B\xF8"
        // mov rax, [rax]              ; "\x48\x8B\x00"
        // mov qword ptr [export], rax ; "\x48\xA3" + packed_exports[0][1]
        // mov rax, rdi                ; "\x48\x8B\xC7"
        // cmp [rax+10], rsi           ; "\x48\x39\x70\x10"
        let mut bc = Vec::new();
        bc.extend_from_slice(&[0x48, 0x8B, 0xF8]);        // mov rdi, rax
        bc.extend_from_slice(&[0x48, 0x8B, 0x00]);        // mov rax, [rax]
        bc.extend_from_slice(&[0x48, 0xA3]);               // mov [export], rax
        bc.extend_from_slice(&export_addr);
        bc.extend_from_slice(&[0x48, 0x8B, 0xC7]);        // mov rax, rdi
        bc.extend_from_slice(&[0x48, 0x39, 0x70, 0x10]);  // cmp [rax+10], rsi
        bc
    }
}

