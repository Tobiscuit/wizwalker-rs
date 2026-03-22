PORT THIS FILE: deimos-reference/src/effect_simulation.py -> gui/tauri/src-tauri/src/automation/effect_simulation.rs

The Python source is 605 lines. Your Rust output should be SIMILAR length (within 80%). The existing Rust file is only 305 lines — it is MISSING about 300 lines of logic.

CRITICAL RULES:
1. Read the ENTIRE Python source file FIRST before writing any code
2. Port EVERY line of logic. NO stubs, NO "// ..." comments, NO placeholders, NO todo!()
3. Rust 2024 edition. Use :: for path access, NOT Python dot syntax
4. After writing, run: cargo check --workspace and fix ALL errors. 0 errors required.
5. Use Result<T,E> for errors, no panics except where Python uses assert
6. Mark known Python bugs with // BUG: but still port the logic
7. The file ALREADY EXISTS with a partial implementation. REPLACE it entirely.

Combat effect simulator. Port ALL functions including sim_damage, sim_heal, sim_steal_health, sim_detonate, sim_effect, and all hanging effect logic. Use types from automation/combat_objects.rs (MagicSchoolIndex with .0 for usize access, UNIVERSAL_SCHOOL_ID, opposite_school_id). Uses lazy_static for HANGING_EFFECT_PATHS, serde_json::Value for cache, and combat_cache::{cache_get, cache_get_multi, cache_modify, cache_remove}. Uses combat_math::curve_stat.

STEPS:
1. Read deimos-reference/src/effect_simulation.py start to end
2. Read the existing gui/tauri/src-tauri/src/automation/effect_simulation.rs
3. Read gui/tauri/src-tauri/src/automation/combat_objects.rs for MagicSchoolIndex, school IDs
4. Read gui/tauri/src-tauri/src/automation/combat_cache.rs for cache functions
5. Read gui/tauri/src-tauri/src/automation/combat_math.rs for curve_stat
6. REPLACE effect_simulation.rs with a COMPLETE faithful line-by-line port
7. cargo check --workspace, fix all errors, 0 errors before finishing
