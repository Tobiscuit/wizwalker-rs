import { jules } from '@google/jules-sdk';

const REPO = 'Tobiscuit/wizwalker-rs';
const BRANCH = 'main';
const LAURENZ_WW = 'https://github.com/LaurenzLikeThat/wizwalker/blob/development/wizwalker';
const DEIMOS = 'https://github.com/Tobiscuit/Deimos-Wizard101/blob/main';

// Preamble added to every task prompt so Jules uses context7 for up-to-date docs
const CONTEXT7_PREAMBLE = `IMPORTANT: You have access to the context7 MCP server. Before writing ANY Rust code, use context7 to look up the latest documentation for the crates you are using:
- Use "Resolve Context7 Library ID" to find library IDs, then "Query Documentation" to get up-to-date API docs and examples.
- Look up: "windows-rs" (for Win32 API bindings), "tokio" (for async runtime), "thiserror" (for error types), "bitflags" (for flag enums), "byteorder" (for binary parsing).
- For Phase 2 tasks, also look up: "egui" and "eframe" (for GUI), "serde" (for serialization), "configparser" (for INI parsing).
- Always verify crate APIs against context7 docs rather than guessing from memory.

`;

// ============================================================
// PHASE 1: WizWalker Core (all run in parallel)
// ============================================================

const phase1Tasks = [
  {
    name: 'Task 1: Project Skeleton + Errors + Constants + Types',
    prompt: `Port the WizWalker foundation files from Python to Rust in this repo.

Source files:
- ${LAURENZ_WW}/errors.py → src/errors.rs (convert all exception types to a Rust enum using thiserror)
- ${LAURENZ_WW}/constants.py → src/constants.rs (convert all constants, preserve exact values)
- ${LAURENZ_WW}/__init__.py → update src/lib.rs with public module declarations and re-exports

Also create src/types.rs with shared types:
- XYZ struct (x: f32, y: f32, z: f32) with Debug, Clone, Copy derives
- Orient struct (pitch: f32, roll: f32, yaw: f32)
- Keycode enum matching the Python version exactly

Preserve all doc comments. The existing Cargo.toml already has the right dependencies configured.
Update lib.rs to declare all modules (pub mod errors; pub mod constants; pub mod types; etc).`
  },
  {
    name: 'Task 2: Memory Reader',
    prompt: `Port the WizWalker memory reader from Python to Rust.

Source: ${LAURENZ_WW}/memory/memory_reader.py → wizwalker/src/memory/reader.rs

Implement a MemoryReader struct wrapping a Windows process handle (from the windows crate) with these async methods:
- read_bytes(address, size) via ReadProcessMemory
- write_bytes(address, data) via WriteProcessMemory  
- read_typed<T>(address) for i8/i16/i32/i64/u8/u16/u32/u64/f32/f64/bool
- write_typed<T>(address, value)
- read_null_terminated_string(address, max_len)
- read_wide_string(address, max_len) (UTF-16)
- pattern_scan(pattern, module) — byte pattern scan with wildcard support (wildcard = 0x2E). Use EnumProcessModulesEx + GetModuleInformation.
- allocate(size) via VirtualAllocEx
- free(address) via VirtualFreeEx

CRITICAL: Copy all byte patterns EXACTLY as they appear in the Python source. Do not modify any hex values.
Use the windows crate for Win32 API. All methods async via tokio. Update memory/mod.rs.`
  },
  {
    name: 'Task 3: Hook System',
    prompt: `Port the WizWalker hook system from Python to Rust. This is the MOST CRITICAL task.

Source files:
- ${LAURENZ_WW}/memory/hooks.py → wizwalker/src/memory/hooks.rs
- ${LAURENZ_WW}/memory/handler.py → wizwalker/src/memory/handler.rs

For hooks.rs: Create a MemoryHook trait with methods (hook, unhook, bytes_to_write, pattern, pattern_offset, hook_size). Implement concrete hooks: PlayerHook, QuestHook, ClientHook, RootWindowHook, RenderContextHook, MouselessCursorMoveHook, MovementTeleportHook, PlayerStatHook.

CRITICAL: Copy ALL byte patterns, shellcode bytes, jump instructions, and offsets EXACTLY from the Python source. These are raw x86-64 machine code. Use byte array constants: const PATTERN: &[u8] = &[0x48, 0x89, 0x5C, ...];

For handler.rs: Port HookHandler including AUTOBOT_PATTERN/AUTOBOT_SIZE constants (exact bytes), prepare/rewrite autobot lifecycle, allocate_autobot_bytes, activate/deactivate hooks, hook base address reading, wait_for_value polling, close().

Every single byte must match the Python source exactly. This is injected machine code.`
  },
  {
    name: 'Task 4: Base Memory Object + Instance Finder',
    prompt: `Port the WizWalker base memory object system from Python to Rust.

Source files:
- ${LAURENZ_WW}/memory/memory_object.py → wizwalker/src/memory/memory_object.rs
- ${LAURENZ_WW}/memory/instance_finder.py → wizwalker/src/memory/instance_finder.rs

MemoryObject: struct holding reference to MemoryReader + base_address. Methods for reading typed values at offsets, reading pointers (read address at offset then read value at that address), reading dynamic-length lists from game memory.

InstanceFinder: locates game objects in memory.

These form the foundation all memory_objects build on. Use generic traits to replicate Python's dynamic property pattern. Update memory/mod.rs.`
  },
  {
    name: 'Task 5: Memory Objects — Player & Core (Batch 1)',
    prompt: `Port the first batch of WizWalker memory object wrappers from Python to Rust.

Source files (all from ${LAURENZ_WW}/memory/memory_objects/):
- actor_body.py → objects/actor_body.rs
- client_object.py → objects/client_object.rs
- core_object.py → objects/core_object.rs
- core_template.py → objects/core_template.rs
- game_client.py → objects/game_client.rs
- client_zone.py → objects/client_zone.rs
- enums.py → objects/enums.rs

Each wraps a MemoryObject with async methods reading typed values at specific offsets. CRITICAL: Preserve ALL offset values exactly. Convert Python IntEnum/Flag to Rust enums with repr(i32) or bitflags! macro. Update objects/mod.rs.`
  },
  {
    name: 'Task 6: Memory Objects — Combat & Spells (Batch 2)',
    prompt: `Port combat and spell memory objects from Python to Rust.

Source files (all from ${LAURENZ_WW}/memory/memory_objects/):
- combat_action.py → objects/combat_action.rs
- combat_participant.py → objects/combat_participant.rs
- combat_resolver.py → objects/combat_resolver.rs
- client_duel_manager.py → objects/client_duel_manager.rs
- duel.py → objects/duel.rs
- spell.py → objects/spell.rs
- spell_effect.py → objects/spell_effect.rs
- spell_rank.py → objects/spell_rank.rs
- spell_template.py → objects/spell_template.rs
- play_deck.py → objects/play_deck.rs
- pip_count.py → objects/pip_count.rs

Same MemoryObject wrapper pattern. Preserve ALL offset values exactly. Handle linked-list and array-style reads. Update objects/mod.rs.`
  },
  {
    name: 'Task 7: Memory Objects — Equipment, Behavior, Stats (Batch 3)',
    prompt: `Port equipment, behavior, and stats memory objects from Python to Rust.

Source files (all from ${LAURENZ_WW}/memory/memory_objects/):
- behavior_instance.py → objects/behavior_instance.rs
- behavior_template.py → objects/behavior_template.rs
- equipment_behavior.py → objects/equipment_behavior.rs
- equipment_set.py → objects/equipment_set.rs
- equipped_item_info.py → objects/equipped_item_info.rs
- equipped_slot_info.py → objects/equipped_slot_info.rs
- game_stats.py → objects/game_stats.rs
- inventory_behavior.py → objects/inventory_behavior.rs
- character_registry.py → objects/character_registry.rs
- game_object_template.py → objects/game_object_template.rs

Same MemoryObject wrapper pattern. Preserve ALL offset values exactly. Update objects/mod.rs.`
  },
  {
    name: 'Task 8: Memory Objects — Camera, Quest, UI, Misc (Batch 4)',
    prompt: `Port remaining memory objects from Python to Rust.

Source files (all from ${LAURENZ_WW}/memory/memory_objects/):
camera_controller.py, cam_view.py, gamebryo_camera.py, gamebryo_presenter.py, quest_client_manager.py, quest_data.py, quest_position.py, goal_data.py, window.py, render_context.py, scene_manager.py, teleport_helper.py, conditionals.py, madlib_arg.py, madlib_block.py, fish.py, fish_template.py, fishing_manager.py, client_tag_list.py

Each maps to a .rs file in wizwalker/src/memory/objects/. Same MemoryObject wrapper pattern. Preserve ALL offset values exactly. Create/update objects/mod.rs to re-export everything.`
  },
  {
    name: 'Task 9: Client + ClientHandler',
    prompt: `Port WizWalker client and client handler from Python to Rust.

Source files:
- ${LAURENZ_WW}/client.py → wizwalker/src/client.rs
- ${LAURENZ_WW}/client_handler.py → wizwalker/src/client_handler.rs

Client struct: process handle, window handle, HookHandler ref, title, process_id. Methods: open(), close(), teleport(), send_key(), zone_name(), body position access, freecam methods (camera_freecam, camera_elastic). Properties delegating to memory objects.

ClientHandler: discovers Wizard101 windows via FindWindowExW/EnumWindows/GetWindowThreadProcessId/OpenProcess. Creates and manages Client instances. All async.`
  },
  {
    name: 'Task 10: Mouse Handler + Hotkey Listener',
    prompt: `Port WizWalker input simulation from Python to Rust.

Source files:
- ${LAURENZ_WW}/mouse_handler.py → wizwalker/src/mouse_handler.rs
- ${LAURENZ_WW}/hotkey.py → wizwalker/src/hotkey.rs

MouseHandler: simulates mouse input via PostMessageW/SendMessageW (WM_LBUTTONDOWN/UP, WM_MOUSEMOVE). RAII guard pattern.

HotkeyListener: RegisterHotKey/UnregisterHotKey, async callbacks, modifier key support, runs on own thread dispatching to tokio. Use windows crate for all Win32 calls.`
  },
  {
    name: 'Task 11: File Readers',
    prompt: `Port WizWalker file readers from Python to Rust.

Source files:
- ${LAURENZ_WW}/file_readers/wad.py → wizwalker/src/file_readers/wad.rs (WAD archive parser)
- ${LAURENZ_WW}/file_readers/nif.py → wizwalker/src/file_readers/nif.rs (NIF 3D file parser)
- ${LAURENZ_WW}/file_readers/cache_handler.py → wizwalker/src/file_readers/cache_handler.rs

Pure data parsers, no Win32 dependency. Use std::io + byteorder crate. Update file_readers/mod.rs.`
  },
  {
    name: 'Task 12: Combat Module + Utils',
    prompt: `Port WizWalker combat module and utils from Python to Rust.

Source files:
- ${LAURENZ_WW}/combat/card.py → wizwalker/src/combat/card.rs
- ${LAURENZ_WW}/combat/handler.py → wizwalker/src/combat/handler.rs
- ${LAURENZ_WW}/combat/member.py → wizwalker/src/combat/member.rs
- ${LAURENZ_WW}/utils.py → wizwalker/src/utils.rs

Utils: get_all_wizard_handles, get_foreground_window etc using windows crate (GetForegroundWindow, EnumWindows, GetWindowTextW). Update combat/mod.rs.`
  },
];

// ============================================================
// PHASE 2: Deimos Application  
// ============================================================

const phase2Tasks = [
  {
    name: 'Task 13: Deimos GUI (egui)',
    prompt: `Port the Deimos GUI from PySimpleGUI to Rust egui/eframe.

Source: ${DEIMOS}/src/deimosgui.py → deimos/src/gui/

Create mod.rs (main App struct implementing eframe::App), tabs.rs (all 9 tabs), commands.rs (GUICommand enum), console.rs (colored log output).

Port all tabs, buttons, toggles, input fields. Use tokio::sync::mpsc for GUI↔backend communication. Dark theme with purple buttons (#4a019e). eframe and egui are already in Cargo.toml.`
  },
  {
    name: 'Task 14: Deimos Config + Hotkeys',
    prompt: `Port Deimos configuration and hotkey system from Python to Rust.

Source: ${DEIMOS}/Deimos.py (lines 1-250) + ${DEIMOS}/Deimos-config.ini

Create deimos/src/config.rs (parse INI with configparser crate) and deimos/src/hotkeys.rs (register hotkeys, toggle functions for each feature, map to GUICommand messages). All settings: speed, potions, RPC, hotkeys, GUI, sigil, questing, combat, auto-pet.`
  },
  {
    name: 'Task 15: Deimos Combat + Questing Logic',
    prompt: `Port Deimos bot automation logic from Python to Rust.

Source files from ${DEIMOS}/src/:
combat_new.py, combat_objects.py, combat_math.py, combat_utils.py, combat_cache.py, config_combat.py, effect_simulation.py, questing.py, sigil.py

Create corresponding .rs files in deimos/src/. Core automation loops using wizwalker Client. All loops as async tokio tasks.`
  },
  {
    name: 'Task 16: Deimos Navigation + Pet + Misc',
    prompt: `Port remaining Deimos bot modules from Python to Rust.

Source files from ${DEIMOS}/src/:
teleport_math.py, collision.py, collision_math.py, auto_pet.py, camera_utils.py, stat_viewer.py, drop_logger.py, command_parser.py, gui_inputs.py, paths.py, sprinty_client.py, utils.py, deck_encoder.py, dance_game_hook.py, discsdk.py (use discord-rich-presence crate)

Create corresponding .rs files in deimos/src/. collision/teleport_math are pure math.`
  },
  {
    name: 'Task 17: DeimosLang Scripting VM',
    prompt: `Port the DeimosLang custom scripting language from Python to Rust.

Source files from ${DEIMOS}/src/deimoslang/:
tokenizer.py, parser.py, ir.py, sem.py, types.py, vm.py
Also: ${DEIMOS}/src/tokenizer.py (top-level tokenizer)

Create deimos/src/deimoslang/ with corresponding .rs files. Self-contained pipeline: tokenizer→parser→IR→semantic analysis→VM. Use Rust enums for token types and AST nodes.`
  },
  {
    name: 'Task 18: Main Entry Point + Integration',
    prompt: `Port the Deimos main entry point and wire everything together.

Source: ${DEIMOS}/Deimos.py (lines 250-2215, main() function and all loops)

Create deimos/src/main.rs that: reads config, opens GUI in separate task, initializes wizwalker ClientHandler, registers hotkeys, runs main event loop (polls GUI queue, updates client info, manages tokio tasks for combat/dialogue/sigil/questing/auto_pet/speed loops, handles toggles/teleport/camera/flythrough/bot scripts, clean shutdown).

Use tokio::select! for multiplexing GUI events and game loop.`
  },
];

// ============================================================
// PHASE 3: Verification
// ============================================================

const phase3Tasks = [
  {
    name: 'Task 19: Cross-Comparison Audit',
    prompt: `Perform a READ-ONLY cross-comparison audit between the Python WizWalker source and the Rust port in this repo.

Python source: ${LAURENZ_WW}/
Rust port: wizwalker/src/

For EACH file, verify:
1. ALL byte patterns (hex arrays) are IDENTICAL between Python and Rust
2. ALL memory offsets are IDENTICAL
3. ALL enum values match
4. ALL Win32 API calls use correct functions and parameters
5. Hook lifecycle is functionally equivalent
6. No functionality was accidentally dropped

Priority files: memory/hooks.rs, memory/handler.rs, memory/reader.rs, all objects/*.rs

Output a report as AUDIT_REPORT.md listing ✅ verified, ⚠️ differences, ❌ missing functionality.
Do NOT modify any code files.`
  },
];

// ============================================================
// LAUNCH
// ============================================================

async function launchPhase(name, tasks) {
  console.log(`\n${'='.repeat(60)}`);
  console.log(`🚀 Launching ${name} (${tasks.length} tasks)`);
  console.log('='.repeat(60));

  const sessions = await jules.all(
    tasks,
    (task) => ({
      prompt: CONTEXT7_PREAMBLE + task.prompt,
      title: task.name,
      source: { github: REPO, baseBranch: BRANCH },
      interactive: false,
      autoPr: true,
    }),
    {
      concurrency: 60,    // Ultra plan
      stopOnError: false,
      delayMs: 1000,
    }
  );

  console.log(`✅ Created ${sessions.length} sessions for ${name}`);
  
  for (const [i, session] of sessions.entries()) {
    console.log(`  [${i + 1}] ${tasks[i].name} — Session ID: ${session.id}`);
  }

  return sessions;
}

async function main() {
  const args = process.argv.slice(2);
  const phase = args[0] || 'all';

  console.log('🧙 WizWalker-RS Jules Launch Script');
  console.log(`📦 Repo: ${REPO}`);
  console.log(`🌿 Branch: ${BRANCH}`);
  console.log(`📋 Phase: ${phase}`);

  if (phase === '1' || phase === 'all') {
    await launchPhase('Phase 1: WizWalker Core', phase1Tasks);
  }

  if (phase === '2' || phase === 'all') {
    await launchPhase('Phase 2: Deimos Application', phase2Tasks);
  }

  if (phase === '3' || phase === 'all') {
    await launchPhase('Phase 3: Verification', phase3Tasks);
  }

  if (!['1', '2', '3', 'all'].includes(phase)) {
    console.log('Usage: node launch-jules.mjs [1|2|3|all]');
    console.log('  1   - Launch Phase 1 only (WizWalker core, 12 tasks)');
    console.log('  2   - Launch Phase 2 only (Deimos app, 6 tasks)');
    console.log('  3   - Launch Phase 3 only (Verification, 1 task)');
    console.log('  all - Launch ALL phases (19 tasks)');
  }
}

main().catch(console.error);
