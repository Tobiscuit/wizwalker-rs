# Jules Dispatch Script — Fire per-file porting tasks
# Each task is one Python file -> one Rust file, with full context

$repo = "Tobiscuit/wizwalker-rs"

# Base instructions shared by all tasks
$baseInstructions = @"
CRITICAL RULES:
1. You MUST read the ENTIRE Python source file specified below FIRST
2. Port EVERY line of logic — NO stubs, NO '// ...' comments, NO placeholders, NO todo!()
3. This is Rust 2024 edition. Use :: for path access (e.g. TokenKind::identifier), not Python's dot syntax
4. After writing the Rust file, you MUST also:
   a. Ensure the module is registered in the parent mod.rs if not already
   b. Add any new crate dependencies to gui/tauri/src-tauri/Cargo.toml if needed
   c. Run: cargo check --workspace 2>&1 | head -50
   d. Fix ALL errors before finishing — 0 errors required
5. Use Result<T, E> for error handling, no panics except where Python uses assert
6. Mark known Python bugs with // BUG: (from Python original) but still port the logic
7. The existing Rust automation modules are at gui/tauri/src-tauri/src/automation/
8. The existing Rust deimoslang modules are at gui/tauri/src-tauri/src/deimoslang/
9. Reference existing Rust modules for types that are already ported (combat_objects.rs, combat_math.rs, etc)

IMPORTANT: The file already exists with a partial/hallucinated implementation. You must REPLACE it entirely with a faithful port.
"@

# Define each task: [PythonSource, RustTarget, ExtraContext]
$tasks = @(
    @{
        Name = "tokenizer"
        Python = "deimos-reference/src/deimoslang/tokenizer.py"
        Rust = "gui/tauri/src-tauri/src/deimoslang/tokenizer.rs"
        Lines = 608
        Extra = "This is the lexer. Port ALL token types and the full tokenize_line() method. The existing Rust file has the correct token enum but the tokenizer logic may be incomplete. Verify every branch."
    },
    @{
        Name = "parser"
        Python = "deimos-reference/src/deimoslang/parser.py"
        Rust = "gui/tauri/src-tauri/src/deimoslang/parser.rs"
        Lines = 1381
        Extra = "This is the LARGEST file (1381 lines). Port EVERY parse method, EVERY command expression, EVERY statement kind. The existing Rust file is only 603 lines — it's missing ~800 lines of logic. Types are defined in deimoslang/types.rs. pay special attention to parse_command_expression() and parse_command_stmt() which map token kinds to AST nodes."
    },
    @{
        Name = "ir"
        Python = "deimos-reference/src/deimoslang/ir.py"
        Rust = "gui/tauri/src-tauri/src/deimoslang/ir.rs"
        Lines = 380
        Extra = "IR compiler that converts AST to flat instruction list. Port ALL of compile_stmt(), compile_command(), and process_labels(). The existing Rust file has correct types but the compilation logic is incomplete."
    },
    @{
        Name = "sem"
        Python = "deimos-reference/src/deimoslang/sem.py"
        Rust = "gui/tauri/src-tauri/src/deimoslang/sem.rs"
        Lines = 333
        Extra = "Semantic analyzer with scope management. Port ALL of sem_stmt() and sem_expr() completely. The existing Rust has stubs. PlayerSelector is in deimoslang/types.rs NOT parser.rs."
    },
    @{
        Name = "vm"
        Python = "deimos-reference/src/deimoslang/vm.py"
        Rust = "gui/tauri/src-tauri/src/deimoslang/vm.rs"
        Lines = 1757
        Extra = "This is the BIGGEST file (1757 lines). The VM executes the IR. Port ALL opcodes in step(), ALL expression evaluation in eval(). Note: eval() is recursive and needs Box::pin for async recursion in Rust. The existing Rust file is only 193 lines — it's missing ~1500 lines. Use the Client type from wizwalker::client::Client."
    },
    @{
        Name = "effect_simulation"
        Python = "deimos-reference/src/effect_simulation.py"
        Rust = "gui/tauri/src-tauri/src/automation/effect_simulation.rs"
        Lines = 605
        Extra = "Combat effect simulator. Port ALL functions including sim_damage, sim_heal, sim_steal_health, sim_detonate, sim_effect, and all hanging effect logic. Use types from automation/combat_objects.rs (MagicSchoolIndex, UNIVERSAL_SCHOOL_ID, opposite_school_id). Uses lazy_static for HANGING_EFFECT_PATHS, serde_json::Value for cache, and combat_cache::{cache_get, cache_modify, cache_remove}."
    },
    @{
        Name = "config_combat"
        Python = "deimos-reference/src/config_combat.py"
        Rust = "gui/tauri/src-tauri/src/automation/config_combat.rs"
        Lines = 59
        Extra = "Small file. Port ALL strategy parsing and delegation. Uses serde_yaml for YAML config parsing."
    },
    @{
        Name = "command_parser"
        Python = "deimos-reference/src/command_parser.py"
        Rust = "gui/tauri/src-tauri/src/automation/command_parser.rs"
        Lines = 328
        Extra = "Command parser for /tp, /speed, /flythrough, etc. Port ALL commands and their argument parsing. Uses regex for pattern matching."
    }
)

Write-Host "=== Jules Per-File Dispatch ===" -ForegroundColor Cyan
Write-Host "Firing $($tasks.Count) tasks..." -ForegroundColor Yellow
Write-Host ""

$sessionIds = @()

foreach ($task in $tasks) {
    $prompt = @"
PORT THIS FILE: $($task.Python) -> $($task.Rust)

The Python source file is $($task.Lines) lines. Your Rust output should be of SIMILAR length (within 80%). If significantly shorter, you have missed logic.

$baseInstructions

SPECIFIC FILE INSTRUCTIONS:
$($task.Extra)

STEP-BY-STEP:
1. Read $($task.Python) from start to end
2. Read the existing $($task.Rust) to understand what's there
3. Read deimos-reference/src/deimoslang/types.py and gui/tauri/src-tauri/src/deimoslang/types.rs to understand shared types
4. REPLACE $($task.Rust) with a COMPLETE faithful port
5. Run cargo check --workspace and fix all errors
6. Verify 0 errors before finishing
"@

    Write-Host "Firing: $($task.Name) ($($task.Lines) lines Python)" -ForegroundColor Green
    
    # Fire the Jules task
    $result = jules new --repo $repo $prompt 2>&1
    Write-Host $result
    Write-Host ""
    
    # Small delay between fires to avoid rate limiting  
    Start-Sleep -Seconds 3
}

Write-Host ""
Write-Host "=== All $($tasks.Count) tasks fired! ===" -ForegroundColor Cyan
Write-Host "Monitor at: https://jules.google.com/" -ForegroundColor Yellow
