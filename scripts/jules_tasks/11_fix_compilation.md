FIX COMPILATION ERRORS: Resolve all `cargo check` errors in the Deimos automation and DeimosLang modules — faithful 1:1 port from Python.

USE CONTEXT7 to look up the latest Rust 2024 edition syntax, `async`/`await` patterns, and best practices before making any changes.

## Context
This repo is a Rust port of a Python wizard101 automation bot called "Deimos." We have:
- Python reference code in `deimos-reference/src/` (the ground truth)
- Rust ports in `gui/tauri/src-tauri/src/automation/` and `gui/tauri/src-tauri/src/deimoslang/`
- The core wizwalker Rust crate in `wizwalker/src/` (already ported, has its own API)

Multiple parallel Jules sessions ported files independently, so there are cross-session integration issues.

## Your Job
1. Run `cargo check --workspace` from `gui/tauri/src-tauri/`
2. For EACH error, determine the correct fix by:
   - Reading the Python reference code to understand the INTENDED behavior
   - Reading the Rust wizwalker crate to understand what methods/types ACTUALLY exist
   - Bridging the gap — if the Rust crate has a different API than Python, adapt the call to use what's available
3. If a Python method has NO Rust equivalent in the wizwalker crate, implement a helper/wrapper in the automation module itself, OR add the method to the wizwalker crate if it's a simple getter/setter

## Key Patterns to Fix
These are the most common error types you'll encounter:

### Import Errors (E0432)
- `wizwalker::combat::CombatMember` → `wizwalker::combat::member::CombatMember`
- `wizwalker::combat::CombatCard` → `wizwalker::combat::card::CombatCard`
- `wizwalker::combat::CombatHandler` → `wizwalker::combat::handler::CombatHandler`
- `use futures::future::join_all` → add `futures` to Cargo.toml or use `tokio::join!`

### Method vs Field Access (E0615)
- `client.title` → `client.title()` (it's a method, not a field)
- `client.root_window` is a FIELD, not a method — remove `()`
- `client.mouse_handler` is a FIELD, not a method — remove `()`

### Missing Methods (E0599)
- `client.body()` — check if Client has this in `wizwalker/src/client.rs`. If not, check what equivalent exists
- `client.stats()` — check the Client struct for actual method names
- `activate_hook()` / `deactivate_hook()` — check HookHandler for actual API
- `read_hook_export()` — check HookHandler for actual API

### send_key Signature (E0061)
- Python: `client.send_key(key, seconds)` — takes key + delay
- Rust: `client.send_key(key)` — takes ONLY the key, no delay. It's also SYNC (not async), so remove `.await`
- Fix: `c.send_key(Keycode::X, 0.1).await.ok()` → `c.send_key(Keycode::X).ok()`

### Type Annotations (E0282)
- Add explicit type annotations where the compiler can't infer types
- Check what `id_to_member()`, `id_to_card()` etc. return and annotate accordingly

### Async Recursion (E0733)
- Use `Box::pin` for async recursion in vm.rs `eval` method

### Non-Exhaustive Patterns (E0004)
- Add missing match arms for new Expression variants

## Critical Rules
1. **DO NOT use stubs, todo!(), or placeholder code.** Every fix must be a real, working implementation.
2. **Read the Python reference** to understand what the code SHOULD do.
3. **Read the Rust wizwalker crate** to understand what's AVAILABLE.
4. **If a method doesn't exist in Rust**, either:
   - Add it to the wizwalker crate (if it's a simple accessor)
   - Create a helper function in the automation module
   - Adapt the logic to use existing Rust APIs that achieve the same result
5. **Preserve the 1:1 logic faithfulness** — the Rust code must behave identically to the Python code.
6. **DO NOT worry about achieving zero errors if some errors are genuinely blocked** by unported wizwalker subsystems. Mark those with `// TODO: requires wizwalker crate port of XYZ` so we know what's left.
7. If you need clarification or are unsure about something, ASK ME in your response rather than guessing.

## Files to Focus On (by error count)
1. `gui/tauri/src-tauri/src/deimoslang/vm.rs` — 74 errors
2. `gui/tauri/src-tauri/src/automation/command_parser.rs` — 60 errors
3. `gui/tauri/src-tauri/src/automation/combat_objects.rs` — 10 errors
4. `gui/tauri/src-tauri/src/automation/sprinty_client.rs` — 11 errors
5. `gui/tauri/src-tauri/src/automation/dance_game_hook.rs` — 6 errors
6. All other files with 1-3 errors each

## Python Reference Files
- `deimos-reference/src/deimoslang/vm.py`
- `deimos-reference/src/command_parser.py`
- `deimos-reference/src/combat/combat_objects.py`
- `deimos-reference/src/sprinty_client.py`
- `deimos-reference/src/dance_game_hook.py`

START by running `cargo check` to see the current errors, THEN fix them systematically file by file.
