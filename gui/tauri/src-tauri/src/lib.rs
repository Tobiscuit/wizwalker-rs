//! WizWalker GUI — Tauri v2 backend entry point.
//!
//! Uses synchronous commands with `std::sync::Mutex` per Tauri v2 docs.
//! All Win32 operations (memory read/write, EnumWindows) are synchronous,
//! so this avoids the Send requirement on async futures that caused
//! compilation issues with the `windows` crate's HWND type.

mod commands;
mod events;
mod state;
mod automation;
mod deimoslang;

use std::sync::Mutex;

use state::WizState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        // Register shared state — sync Mutex per Tauri v2 state management docs
        .manage(Mutex::new(WizState::new()))
        // Register all IPC command handlers (17 total)
        .invoke_handler(tauri::generate_handler![
            // Client management
            commands::clients::scan_clients,
            commands::clients::get_clients,
            commands::clients::open_client,
            commands::clients::activate_hooks,
            commands::clients::close_client,
            // Hook toggles
            commands::hooks::get_toggle_states,
            commands::hooks::toggle_hook,
            commands::hooks::get_speed_multiplier,
            commands::hooks::set_speed_multiplier,
            // Navigation
            commands::navigation::get_position,
            commands::navigation::teleport_to,
            commands::navigation::xyz_sync,
            // Combat
            commands::combat::get_combat_status,
            commands::combat::get_stats,
            commands::combat::get_cards,
            // Camera
            commands::camera::get_camera,
            commands::camera::set_camera_position,
            commands::camera::set_camera_fov,
            commands::camera::set_camera_rotation,
        ])
        .setup(|app| {
            // Spawn background telemetry event loop (uses std::thread)
            let app_handle = app.handle().clone();
            events::spawn_telemetry_loop(app_handle);

            tracing::info!("WizWalker GUI initialized — 17 IPC commands registered");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
