//! Background event system — pushes real-time data from Rust to the frontend.
//!
//! Spawned inside `tauri::Builder::setup()`. Uses `AppHandle::emit()` to deliver
//! typed payloads that the React frontend listens to via `@tauri-apps/api/event`.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use wizwalker::memory::reader::MemoryReaderExt;

use crate::state::{Position, TelemetryPayload, WizState};

/// Spawn the background telemetry emitter task.
///
/// Runs on a std::thread (not tokio) since we use std::sync::Mutex.
/// Reads position, zone, and combat status every 500ms and emits
/// a `telemetry-update` event to all frontend listeners.
pub fn spawn_telemetry_loop(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            let state = app.state::<Mutex<WizState>>();
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

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}
