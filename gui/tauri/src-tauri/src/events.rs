//! Background event system — pushes real-time data from Rust to the frontend.
//!
//! Spawned inside `tauri::Builder::setup()`. Uses `AppHandle::emit()` to deliver
//! typed payloads that the React frontend listens to via `@tauri-apps/api/event`.
//!
//! ## Auto-scan + Auto-hook (Deimos-style)
//!
//! Instead of requiring the user to manually click "Scan" then "Hook," the
//! telemetry loop performs these automatically:
//!
//! 1. **No clients?** → call `get_new_clients()` every 2 seconds
//! 2. **Clients found but not hooked?** → call `activate_hooks()` on each
//! 3. **Clients hooked?** → stream live telemetry every 100ms
//!
//! This matches Python Deimos behavior where you just open the app and it works.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use wizwalker::memory::reader::{MemoryReader, MemoryReaderExt};

use crate::state::{Position, TelemetryPayload, WizState};

/// Spawn the background telemetry + auto-management loop.
///
/// Runs on a std::thread (not tokio) since we use std::sync::Mutex for WizState.
pub fn spawn_telemetry_loop(app: AppHandle) {
    std::thread::spawn(move || {
        // Counter to rate-limit scanning (scan every 2s, not every 100ms)
        let mut scan_cooldown: u32 = 0;
        // Counter to rate-limit hook retries (retry every 5s on failure)
        let mut hook_cooldown: u32 = 0;
        // Counter for telemetry log rate-limiting
        let mut telem_counter: u32 = 0;
        // Tracking active combat tasks to avoid duplicate spawns
        let mut active_combat: std::collections::HashSet<String> = std::collections::HashSet::new();

        loop {
            let state = app.state::<Mutex<WizState>>();

            // ── Phase 1: Auto-scan for new clients ──────────────────────
            // Only scan if no clients are registered and cooldown elapsed.
            {
                let mut wiz = state.lock().unwrap();
                if wiz.clients.is_empty() && scan_cooldown == 0 {
                    match wiz.handler.get_new_clients() {
                        Ok(new_clients) => {
                            for client_arc in new_clients {
                                let idx = wiz.clients.len();
                                let label = WizState::client_label(idx);
                                eprintln!("[arcane] Auto-scan: found client {label}");
                                wiz.clients.insert(label, client_arc);
                            }

                            if !wiz.clients.is_empty() {
                                // Emit a client-update event so the frontend
                                // refreshes its client list immediately.
                                let _ = app.emit("clients-changed", true);
                            }
                        }
                        Err(e) => {
                            eprintln!("[arcane] Auto-scan failed: {e}");
                        }
                    }

                    // Reset cooldown: don't scan again for 2 seconds (20 × 100ms).
                    scan_cooldown = 20;
                }

                if scan_cooldown > 0 {
                    scan_cooldown -= 1;
                }
            }

            // ── Phase 2: Auto-hook unhooked clients ─────────────────────
            if hook_cooldown == 0 {
                let wiz = state.lock().unwrap();
                for (label, client_arc) in &wiz.clients {
                    let mut client = client_arc.blocking_lock();
                    // Only activate if not already hooked and client is running.
                    if !client.hook_handler.has_any_hooks() && client.is_running() {
                        eprintln!("[arcane] Auto-hook: activating hooks for {label}");
                        match client.activate_hooks() {
                            Ok(()) => {
                                eprintln!("[arcane] Auto-hook: hooks activated for {label} ✓");
                            }
                            Err(e) => {
                                eprintln!("[arcane] Auto-hook: FAILED for {label}: {e}");
                                // Don't retry for 5 seconds (50 × 100ms)
                                hook_cooldown = 50;
                            }
                        }
                    }
                }
            }

            if hook_cooldown > 0 {
                hook_cooldown -= 1;
            }

            // ── Phase 3: Read + emit telemetry ──────────────────────────
            let payload = {
                let wiz = state.lock().unwrap();

                // Find the first client that has hooks activated
                let mut found_payload = None;
                for (label, client_arc) in &wiz.clients {
                    let client = client_arc.blocking_lock();
                    if client.hook_handler.has_any_hooks() {
                        // Read position from the PlayerHook export
                        // Chain: player_struct export → deref → player base → +88 = XYZ
                        let position =
                            if let Ok(player_base) = client.hook_handler.read_current_player_base() {
                                if let Some(reader) = client.reader() {
                                    Position {
                                        x: reader.read_typed::<f32>(player_base + 88).unwrap_or(0.0),
                                        y: reader.read_typed::<f32>(player_base + 92).unwrap_or(0.0),
                                        z: reader.read_typed::<f32>(player_base + 96).unwrap_or(0.0),
                                    }
                                } else {
                                    Position::default()
                                }
                            } else {
                                Position::default()
                            };

                        // Log the first non-zero position for debugging
                        if position.x != 0.0 || position.y != 0.0 || position.z != 0.0 {
                            if telem_counter % 100 == 0 {
                                eprintln!("[arcane] Telemetry: {} pos=({:.1}, {:.1}, {:.1})",
                                    label, position.x, position.y, position.z);
                            }
                        }

                        found_payload = Some(TelemetryPayload {
                            active_client: Some(label.clone()),
                            position,
                            zone: client
                                .zone_name()
                                .unwrap_or_else(|| "Unknown".into()),
                            in_combat: client.in_battle(),
                        });
                        break;
                    }
                }

                found_payload.unwrap_or(TelemetryPayload {
                    active_client: None,
                    position: Position::default(),
                    zone: "—".into(),
                    in_combat: false,
                })
            };

            // Emit to all frontend listeners
            if let Err(e) = app.emit("telemetry-update", &payload) {
                tracing::warn!("Failed to emit telemetry: {e}");
            }

            // ── Phase 4: Auto-automation dispatch ────────────────────────
            // Matches Python Deimos.py main loop behavior:
            //   - speed_switching() re-applies speed every ~300ms
            //   - dialogue_loop() checks advance_dialog_path every 100ms
            //   - combat_loop() enters SprintyCombat when in_battle()
            //   - anti_afk sends periodic movement
            {
                let wiz = state.lock().unwrap();
                let speedhack = wiz.toggles.get("speedhack").copied().unwrap_or(false);
                let auto_dialogue = wiz.toggles.get("auto_dialogue").copied().unwrap_or(false);
                let auto_combat = wiz.toggles.get("auto_combat").copied().unwrap_or(false);
                let anti_afk = wiz.toggles.get("anti_afk").copied().unwrap_or(false);

                // Speed re-application: every ~300ms (every 3rd tick)
                // Python Deimos.py:878-888 — speed_switching() continuously
                // re-writes speed because the game resets it on zone/realm change
                let should_reapply_speed = speedhack && telem_counter % 3 == 0;

                // Automation: every ~500ms (every 5th tick)
                let should_run_auto = telem_counter % 5 == 0;

                for (_label, client_arc) in &wiz.clients {
                    let client = client_arc.blocking_lock();
                    if !client.hook_handler.has_any_hooks() || !client.is_running() {
                        continue;
                    }

                    // ── Speed re-application (Deimos.py:878-888) ──────
                    // Game resets speed_multiplier to 0 on zone change.
                    // We re-read and re-write if it differs from target.
                    if should_reapply_speed {
                        let target_speed = ((wiz.speed_multiplier - 1.0) * 100.0) as i16;
                        if let Some(game_client) = client.game_client() {
                            let gc_base = game_client.read_base_address().unwrap_or(0);
                            if gc_base != 0 {
                                let arc_reader = client.hook_handler.reader().unwrap().as_ref();
                                let client_obj_ptr_bytes = arc_reader.read_bytes((gc_base + 0x21318) as usize, 8).unwrap_or(vec![0; 8]);
                                let mut ptr_arr = [0u8; 8];
                                ptr_arr.copy_from_slice(&client_obj_ptr_bytes[..8]);
                                let client_obj_ptr = u64::from_ne_bytes(ptr_arr);
                                if client_obj_ptr != 0 {
                                    if let Ok(current_bytes) = arc_reader.read_bytes((client_obj_ptr + 0x190) as usize, 2) {
                                        let mut current_arr = [0u8; 2];
                                        current_arr.copy_from_slice(&current_bytes[..2]);
                                        let current = i16::from_ne_bytes(current_arr);
                                        if current != target_speed {
                                            let _ = arc_reader.write_typed((client_obj_ptr + 0x190) as usize, &target_speed);
                                            tracing::info!("[arcane] Speed re-apply (bypass): {}→{} for {}", current, target_speed, _label);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !should_run_auto {
                        continue;
                    }

                    // ── Auto Dialogue (Deimos.py:928-942) ─────────────
                    // Python checks advance_dialog_path visibility, then:
                    //   - If decline_quest_path visible & side_quests off → ESC
                    //   - Otherwise → SPACEBAR
                    if auto_dialogue {
                        crate::automation::dialogue::try_advance_dialogue(&client);
                    }

                    // ── Auto Combat (SprintyCombat AoeHandler) ───────────────
                    if auto_combat {
                        if client.in_battle() {
                            if !active_combat.contains(_label) {
                                active_combat.insert(_label.clone());
                                let client_clone = std::sync::Arc::new(client.clone());
                                let label_clone = _label.clone();
                                tauri::async_runtime::spawn(async move {
                                    tracing::info!("[arcane] Starting auto-combat for {}", label_clone);
                                    let handler = std::sync::Arc::new(wizwalker::combat::handler::CombatHandler::new(client_clone));
                                    let aoe_handler = wizwalker::combat::handler::AoeHandler::new(handler);
                                    let _ = aoe_handler.handle_combat().await;
                                    tracing::info!("[arcane] Auto-combat finished for {}", label_clone);
                                });
                            }
                        } else {
                            active_combat.remove(_label);
                        }
                    } else if active_combat.contains(_label) {
                        active_combat.remove(_label);
                    }

                    // ── Anti-AFK (Deimos.py:118, camera jiggle) ───────
                    // Every ~30 seconds (telem_counter 300 = 30s at 100ms tick)
                    if anti_afk && telem_counter % 300 == 0 {
                        crate::automation::anti_afk::send_anti_afk(&client);
                    }
                }
            }

            telem_counter = telem_counter.wrapping_add(1);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}
