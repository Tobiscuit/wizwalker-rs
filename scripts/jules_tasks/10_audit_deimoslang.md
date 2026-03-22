AUDIT TASK: Verify 1:1 faithfulness of ALL DeimosLang Rust modules against Python originals, and fix Rust 2024 syntax issues.

USE CONTEXT7 to look up the latest Rust 2024 edition syntax and best practices before making any changes.

This repo contains Python reference code in deimos-reference/src/deimoslang/ and Rust ports in gui/tauri/src-tauri/src/deimoslang/.

YOUR JOB:
For EACH of the following file pairs, read BOTH files and verify the Rust is a faithful 1:1 port of the Python. Report any:
- Missing functions (in Python but not in Rust)
- Wrong logic (Rust doesn't match Python behavior)
- Stub/placeholder code (// ... or todo!())
- Rust 2024 edition syntax issues (use :: not ., no explicit ref in patterns, Box::pin for async recursion)

FILE PAIRS TO AUDIT:
1. deimos-reference/src/deimoslang/tokenizer.py -> gui/tauri/src-tauri/src/deimoslang/tokenizer.rs
2. deimos-reference/src/deimoslang/parser.py -> gui/tauri/src-tauri/src/deimoslang/parser.rs
3. deimos-reference/src/deimoslang/ir.py -> gui/tauri/src-tauri/src/deimoslang/ir.rs
4. deimos-reference/src/deimoslang/sem.py -> gui/tauri/src-tauri/src/deimoslang/sem.rs
5. deimos-reference/src/deimoslang/vm.py -> gui/tauri/src-tauri/src/deimoslang/vm.rs
6. deimos-reference/src/deimoslang/types.py -> gui/tauri/src-tauri/src/deimoslang/types.rs

For types.rs specifically: verify that ALL enums, structs, and type aliases from the Python types.py are present in the Rust types.rs. If any are missing, ADD them.

FOR EACH FILE:
1. Read the Python file completely
2. Read the Rust file completely
3. Compare function by function, class by class
4. If any function is missing, add it
5. If any logic is wrong, fix it
6. If any stubs exist (// ... comments), replace with real ported logic

After all fixes:
- Run cargo check --workspace and fix ALL errors
- 0 errors required before finishing

USE CONTEXT7 for any Rust syntax questions (especially 2024 edition patterns, async/await, error handling).
