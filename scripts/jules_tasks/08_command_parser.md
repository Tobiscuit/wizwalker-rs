PORT THIS FILE: deimos-reference/src/command_parser.py -> gui/tauri/src-tauri/src/automation/command_parser.rs

The Python source is 328 lines. Your Rust output should be SIMILAR length (within 80%).

CRITICAL RULES:
1. Read the ENTIRE Python source file FIRST before writing any code
2. Port EVERY line of logic. NO stubs, NO "// ..." comments, NO placeholders, NO todo!()
3. Rust 2024 edition. Use :: for path access, NOT Python dot syntax
4. After writing, run: cargo check --workspace and fix ALL errors. 0 errors required.
5. Use Result<T,E> for errors, no panics except where Python uses assert
6. Mark known Python bugs with // BUG: but still port the logic
7. The file ALREADY EXISTS with a partial implementation. REPLACE it entirely.

Command parser for /tp, /speed, /flythrough, etc. Port ALL commands and their argument parsing. Uses regex for pattern matching.

STEPS:
1. Read deimos-reference/src/command_parser.py start to end
2. Read the existing gui/tauri/src-tauri/src/automation/command_parser.rs
3. REPLACE command_parser.rs with a COMPLETE faithful line-by-line port
4. cargo check --workspace, fix all errors, 0 errors before finishing
