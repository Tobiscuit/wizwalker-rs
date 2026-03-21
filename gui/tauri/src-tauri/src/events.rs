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

use wizwalker::memory::reader::MemoryReaderExt;

use crate::state::{Position, TelemetryPayload, WizState};

/// Spawn the background telemetry + auto-management loop.
///
/// Runs on a std::thread (not tokio) since we use std::sync::Mutex for WizState.
pub fn spawn_telemetry_loop(app: AppHandle) {
    std::thread::spawn(move || {
        // Counter to rate-limit scanning (scan every 2s, not every 100ms)
        let mut scan_cooldown: u32 = 0;

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
                                tracing::info!("Auto-scan: found client {label}");
                                wiz.clients.insert(label, client_arc);
                            }

                            if !wiz.clients.is_empty() {
                                // Emit a client-update event so the frontend
                                // refreshes its client list immediately.
                                let _ = app.emit("clients-changed", true);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Auto-scan failed: {e}");
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
            {
                let wiz = state.lock().unwrap();
                for (label, client_arc) in &wiz.clients {
                    let mut client = client_arc.blocking_lock();
                    // Only activate if not already hooked and client is running.
                    if !client.hook_handler.has_any_hooks() && client.is_running() {
                        tracing::info!("Auto-hook: activating hooks for {label}");
                        match client.activate_hooks() {
                            Ok(()) => {
                                tracing::info!("Auto-hook: hooks activated for {label}");
                            }
                            Err(e) => {
                                tracing::warn!("Auto-hook: failed for {label}: {e}");
                            }
                        }
                    }
                }
            }

            // ── Phase 3: Read + emit telemetry ──────────────────────────
            let payload = {
                let wiz = state.lock().unwrap();
                let label = WizState::client_label(wiz.active_client_idx);

                if let Some(client_arc) = wiz.clients.get(&label) {
                    let client = client_arc.blocking_lock();

                    // Read position from the teleport helper hook export
                    let position =
                        if let Ok(addr) = client.hook_handler.read_teleport_helper() {
                            if let Some(reader) = client.reader() {
                                Position {
                                    x: reader.read_typed::<f32>(addr).unwrap_or(0.0),
                                    y: reader.read_typed::<f32>(addr + 4).unwrap_or(0.0),
                                    z: reader.read_typed::<f32>(addr + 8).unwrap_or(0.0),
                                }
                            } else {
                                Position::default()
                            }
                        } else {
                            Position::default()
                        };

                    TelemetryPayload {
                        active_client: Some(label),
                        position,
                        zone: client
                            .zone_name()
                            .unwrap_or_else(|| "Unknown".into()),
                        in_combat: client.in_battle(),
                    }
                } else {
                    TelemetryPayload {
                        active_client: None,
                        position: Position::default(),
                        zone: "—".into(),
                        in_combat: false,
                    }
                }
            };

            // Emit to all frontend listeners
            if let Err(e) = app.emit("telemetry-update", &payload) {
                tracing::warn!("Failed to emit telemetry: {e}");
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}
