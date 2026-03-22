AUDIT TASK: Verify 1:1 faithfulness of ALL Rust automation modules against Python originals, and fix Rust 2024 syntax issues.

USE CONTEXT7 to look up the latest Rust 2024 edition syntax and best practices before making any changes.

This repo contains Python reference code in deimos-reference/src/ and Rust ports in gui/tauri/src-tauri/src/automation/.

YOUR JOB:
For EACH of the following file pairs, read BOTH files and verify the Rust is a faithful 1:1 port of the Python. Report any:
- Missing functions (in Python but not in Rust)
- Wrong logic (Rust doesn't match Python behavior)
- Stub/placeholder code (// ... or todo!())
- Rust 2024 edition syntax issues (use :: not ., no explicit ref in patterns, Box::pin for async recursion)

FILE PAIRS TO AUDIT:
1. deimos-reference/src/auto_pet.py -> gui/tauri/src-tauri/src/automation/auto_pet.rs
2. deimos-reference/src/camera_utils.py -> gui/tauri/src-tauri/src/automation/camera_utils.rs
3. deimos-reference/src/collision.py -> gui/tauri/src-tauri/src/automation/collision.rs
4. deimos-reference/src/combat_cache.py -> gui/tauri/src-tauri/src/automation/combat_cache.rs
5. deimos-reference/src/combat_math.py -> gui/tauri/src-tauri/src/automation/combat_math.rs
6. deimos-reference/src/combat_objects.py -> gui/tauri/src-tauri/src/automation/combat_objects.rs
7. deimos-reference/src/combat_utils.py -> gui/tauri/src-tauri/src/automation/combat_utils.rs
8. deimos-reference/src/deck_encoder.py -> gui/tauri/src-tauri/src/automation/deck_encoder.rs
9. deimos-reference/src/questing.py -> gui/tauri/src-tauri/src/automation/questing.rs
10. deimos-reference/src/sprinty_client.py -> gui/tauri/src-tauri/src/automation/sprinty_client.rs
11. deimos-reference/src/stat_viewer.py -> gui/tauri/src-tauri/src/automation/stat_viewer.rs
12. deimos-reference/src/teleport_math.py -> gui/tauri/src-tauri/src/automation/teleport_math.rs
13. deimos-reference/src/paths.py -> gui/tauri/src-tauri/src/automation/paths.rs
14. deimos-reference/src/utils.py -> gui/tauri/src-tauri/src/automation/utils.rs
15. deimos-reference/src/sigil.py -> gui/tauri/src-tauri/src/automation/sigil.rs

FOR EACH FILE:
1. Read the Python file completely
2. Read the Rust file completely
3. Compare function by function
4. If any function is missing, add it
5. If any logic is wrong, fix it
6. If any stubs exist, replace with real logic

After all fixes:
- Run cargo check --workspace and fix ALL errors
- 0 errors required before finishing

USE CONTEXT7 for any Rust syntax questions (especially 2024 edition patterns, async/await, error handling).
