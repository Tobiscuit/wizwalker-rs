PORT THIS FILE: deimos-reference/src/deimoslang/parser.py -> gui/tauri/src-tauri/src/deimoslang/parser.rs

The Python source is 1381 lines. Your Rust output should be SIMILAR length (within 80%). The existing Rust file is only 603 lines — it is MISSING about 800 lines of logic.

CRITICAL RULES:
1. Read the ENTIRE Python source file FIRST before writing any code
2. Port EVERY line of logic. NO stubs, NO "// ..." comments, NO placeholders, NO todo!()
3. Rust 2024 edition. Use :: for path access (TokenKind::identifier), NOT Python dot syntax
4. After writing, run: cargo check --workspace and fix ALL errors. 0 errors required.
5. Use Result<T,E> for errors, no panics except where Python uses assert
6. Mark known Python bugs with // BUG: but still port the logic
7. The file ALREADY EXISTS with a partial implementation. REPLACE it entirely with a faithful port.

This is the LARGEST file (1381 lines). Port EVERY parse method, EVERY command expression case, EVERY statement kind. Pay special attention to parse_command_expression() and parse_command_stmt() which map token kinds to AST nodes. Types are in deimoslang/types.rs. PlayerSelector is in types.rs NOT parser.rs.

STEPS:
1. Read deimos-reference/src/deimoslang/parser.py start to end (ALL 1381 lines)
2. Read the existing gui/tauri/src-tauri/src/deimoslang/parser.rs
3. Read gui/tauri/src-tauri/src/deimoslang/types.rs for shared types
4. Read gui/tauri/src-tauri/src/deimoslang/tokenizer.rs for Token/TokenKind types
5. REPLACE parser.rs with a COMPLETE faithful line-by-line port
6. cargo check --workspace, fix all errors, 0 errors before finishing
