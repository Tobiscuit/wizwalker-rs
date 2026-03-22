# Jules Task: Port DeimosLang + Combat Config to Rust

## Overview

This repository (`wizwalker-rs`) is a Rust rewrite of a Python game automation tool for Wizard101. The core library (`wizwalker/`) and most automation modules (`gui/tauri/src-tauri/src/automation/`) have been ported. **The following Python modules still need to be ported to Rust:**

1. **`deimos-reference/src/deimoslang/`** — A custom scripting language VM (tokenizer + parser + executor)
2. **`deimos-reference/src/command_parser.py`** — Parses user text commands into game actions
3. **`deimos-reference/src/config_combat.py`** — Reads YAML combat strategy config files
4. **`deimos-reference/src/effect_simulation.py`** — Combat effect simulation (832 lines, unfinished, has known bugs)

## Instructions

**Use context7 MCP for the latest Rust syntax, docs, and patterns.** All code should follow idiomatic Rust 2024 edition conventions.

---

## 1. DeimosLang VM (`deimos-reference/src/deimoslang/`)

This is a small domain-specific language that lets users write combat strategies. It has:

- **`tokenizer.py`** (in `deimos-reference/src/tokenizer.py`) — Lexer that converts text into tokens
- **`deimoslang/__init__.py`** — Package init
- **`deimoslang/vm.py`** — The virtual machine that executes tokenized DeimosLang scripts

### What to port

Create a new Rust module at `gui/tauri/src-tauri/src/deimoslang/` with:

```
deimoslang/
├── mod.rs          -- pub mod tokenizer; pub mod vm;
├── tokenizer.rs    -- Token types + lexer
└── vm.rs           -- VM executor
```

### Key considerations

- The tokenizer converts text like `"IF health < 50 THEN heal ELSE attack"` into a token stream
- The VM executes these tokens against live game state (combat cache, card hand, etc.)
- The VM imports `delegate_combat_configs` and `default_config` from `config_combat.py` — so `config_combat.rs` must be ported first or simultaneously
- Use Rust enums for token types, `match` for dispatch, `Result<T, E>` for error handling
- The VM needs access to combat state which is provided by `gui/tauri/src-tauri/src/automation/combat_cache.rs` and `combat_objects.rs` (already ported)

---

## 2. Command Parser (`deimos-reference/src/command_parser.py`)

Parses user-entered text commands (from a GUI text input or chat) and maps them to game actions.

### What to port

Create `gui/tauri/src-tauri/src/automation/command_parser.rs` and register it in `gui/tauri/src-tauri/src/automation/mod.rs`.

### Key considerations

- Should parse commands like `"/tp 100 200 300"`, `"/speed 5"`, `"/flythrough zone1 zone2"`
- Returns structured command enums that the caller can dispatch
- Uses `execute_flythrough` and `parse_command` functions
- Integrate with existing `wizwalker::constants::Keycode` for key-related commands

---

## 3. Combat Config (`deimos-reference/src/config_combat.py`)

Reads YAML config files that define combat strategies (which spells to use, priority, targeting).

### What to port

Create `gui/tauri/src-tauri/src/automation/config_combat.rs` and register it in mod.rs.

### Key considerations

- In Python, this uses YAML parsing. In Rust, use `serde` + `serde_yaml` crate
- Define the config struct with `#[derive(Deserialize, Serialize, Default)]`
- The key types are `StrCombatConfigProvider`, `delegate_combat_configs`, and `default_config`
- This module is imported by `deimoslang/vm.py` — port this FIRST

### Example Rust pattern

```rust
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct CombatConfig {
    pub strategy: String,
    pub spell_priority: Vec<String>,
    pub target_priority: String,
    // ... match the Python fields
}

pub fn load_config(path: &str) -> Result<CombatConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: CombatConfig = serde_yaml::from_str(&contents)?;
    Ok(config)
}
```

---

## 4. Effect Simulation (`deimos-reference/src/effect_simulation.py`)

An **unfinished** combat effect simulator (832 lines with known bugs). This simulates what spells/effects would do to predict optimal plays.

### What to port

Create `gui/tauri/src-tauri/src/automation/effect_simulation.rs` and register it in mod.rs.

### Key considerations

- This is 832 lines of Python with **known bugs** — port faithfully but add `// TODO: known bug from Python` comments where the Python code has issues
- Uses combat objects from `combat_objects.rs` (already ported) and combat math from `combat_math.rs` (already ported)
- Heavy use of lists and dictionaries — use `Vec` and `HashMap` in Rust

---

## Project Architecture Context

### Key directories

```
wizwalker-rs/
├── wizwalker/                    -- Core Rust library (memory hooks, process reader, etc.)
│   └── src/
│       ├── client.rs             -- Client struct with game interaction methods  
│       ├── memory/
│       │   ├── hooks.rs          -- 9 memory hooks (Player, Quest, Client, etc.)
│       │   ├── handler.rs        -- HookHandler manages hook lifecycle
│       │   ├── process_reader.rs -- ReadProcessMemory/WriteProcessMemory wrapper
│       │   └── reader.rs         -- MemoryReader trait + MemoryReaderExt
│       └── constants.rs          -- Keycode enum, game constants
├── gui/tauri/src-tauri/src/
│   ├── automation/               -- Game automation modules (ALREADY PORTED)
│   │   ├── combat_cache.rs       -- Combat state cache
│   │   ├── combat_math.rs        -- Damage/resist/critical calculations
│   │   ├── combat_objects.rs     -- CombatMember, SpellEffect, etc.
│   │   ├── combat_utils.rs       -- Combat helper functions
│   │   ├── dialogue.rs           -- Auto dialogue
│   │   ├── anti_afk.rs           -- Anti-AFK
│   │   └── paths.rs              -- UI window path constants
│   ├── commands/                 -- Tauri IPC command handlers
│   ├── events.rs                 -- Background telemetry + automation loop
│   └── state.rs                  -- Application state (WizState)
├── deimos-reference/             -- PYTHON SOURCE (reference for porting)
│   ├── Deimos.py                 -- Main loop (2215 lines)
│   └── src/
│       ├── deimoslang/           -- ← PORT THIS
│       │   └── vm.py
│       ├── tokenizer.py          -- ← PORT THIS  
│       ├── command_parser.py     -- ← PORT THIS
│       ├── config_combat.py      -- ← PORT THIS
│       └── effect_simulation.py  -- ← PORT THIS
└── wizwalker-reference/          -- Python wizwalker library (reference)
```

### Already-ported modules you can reference

- `combat_cache.rs` — Shows how combat state is stored/accessed in Rust
- `combat_objects.rs` — Has `CombatMember`, `SpellEffect`, `CombatCard` structs
- `combat_math.rs` — Damage calculations, already ported from Python
- `dialogue.rs` — Example of simple automation module structure

### Dependencies already in Cargo.toml

- `serde`, `serde_json` — serialization
- `flate2` — compression  
- `base64` — encoding
- `bitflags` — flag enums

### Dependencies you may need to add

- `serde_yaml` — for config_combat.rs YAML parsing

---

## Build & Verify

After porting, run:

```bash
cargo check --workspace
```

All code must compile with **0 errors**. Warnings are acceptable.

## Coding Standards

- Use `Result<T, E>` for fallible operations, not panics
- Use `pub(crate)` where appropriate for internal-only APIs
- Match the Python function signatures as closely as possible
- Add doc comments (`///`) with `# Python equivalent` references
- Register all new modules in their parent `mod.rs`
- Use context7 for the latest Rust syntax and patterns
