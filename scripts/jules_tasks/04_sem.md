PORT THIS FILE: deimos-reference/src/deimoslang/sem.py -> gui/tauri/src-tauri/src/deimoslang/sem.rs

The Python source is 333 lines. Your Rust output should be SIMILAR length (within 80%). The existing Rust file is only 173 lines — it is MISSING about 160 lines of logic.

CRITICAL RULES:
1. Read the ENTIRE Python source file FIRST before writing any code
2. Port EVERY line of logic. NO stubs, NO "// ..." comments, NO placeholders, NO todo!()
3. Rust 2024 edition. Use :: for path access, NOT Python dot syntax
4. After writing, run: cargo check --workspace and fix ALL errors. 0 errors required.
5. Use Result<T,E> for errors, no panics except where Python uses assert
6. Mark known Python bugs with // BUG: but still port the logic
7. The file ALREADY EXISTS with a partial implementation. REPLACE it entirely.

Port ALL of sem_stmt() and sem_expr() completely. Scope management, variable tracking, mixin handling, block definitions. PlayerSelector is in deimoslang/types.rs NOT parser.rs.

STEPS:
1. Read deimos-reference/src/deimoslang/sem.py start to end
2. Read the existing gui/tauri/src-tauri/src/deimoslang/sem.rs
3. Read gui/tauri/src-tauri/src/deimoslang/types.rs for all shared types
4. REPLACE sem.rs with a COMPLETE faithful line-by-line port
5. cargo check --workspace, fix all errors, 0 errors before finishing
