//! Hook toggle and feature control commands.
//!
//! Synchronous pattern per Tauri v2 docs: `State<'_, Mutex<T>>` + `.lock().unwrap()`.

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;

use crate::state::{CommandResult, WizState};

use wizwalker::memory::reader::MemoryReaderExt;

/// Get the current state of all toggles.
#[tauri::command]
pub fn get_toggle_states(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<HashMap<String, bool>> {
    let wiz = state.lock().unwrap();
    Ok(wiz.toggles.clone())
}

/// Toggle a hook/feature on or off. Returns the new toggle state.
#[tauri::command]
pub fn toggle_hook(
    state: State<'_, Mutex<WizState>>,
    name: String,
    enabled: bool,
) -> CommandResult<bool> {
    let mut wiz = state.lock().unwrap();
    wiz.toggles.insert(name.clone(), enabled);

    // Dispatch to the appropriate backend action.
    match name.as_str() {
        "speedhack" => {
            // Write speed multiplier to game memory via ClientHook export
            // Python: client_object.speed_multiplier (CoreObject offset 192, i16)
            let speed_val = if enabled { wiz.speed_multiplier as i16 } else { 100i16 };
            for (_label, client_arc) in &wiz.clients {
                if let Ok(client) = client_arc.try_lock() {
                    if let Ok(client_base) = client.hook_handler.read_current_client_base() {
                        if let Some(reader) = client.process_reader() {
                            let _ = reader.write_typed::<i16>(client_base + 192, &speed_val);
                            eprintln!("[arcane] Speedhack: wrote speed={} to client base 0x{:X}+192",
                                speed_val, client_base);
                        }
                    }
                }
            }
        }
        "auto_combat" => {
            tracing::info!("Auto Combat toggled: {enabled}");
            // Toggle is stored; the telemetry loop checks wiz.toggles["auto_combat"]
        }
        "auto_dialogue" => {
            tracing::info!("Auto Dialogue toggled: {enabled}");
        }
        "auto_sigil" => {
            tracing::info!("Auto Sigil toggled: {enabled}");
        }
        "auto_questing" => {
            tracing::info!("Auto Questing toggled: {enabled}");
        }
        "pet_trainer" => {
            tracing::info!("Pet Trainer toggled: {enabled}");
        }
        "auto_potions" => {
            tracing::info!("Auto Potions toggled: {enabled}");
        }
        "anti_afk" => {
            tracing::info!("Anti-AFK toggled: {enabled}");
            // Toggle is stored; the telemetry loop handles periodic camera jiggle
        }
        other => {
            tracing::warn!("Unknown toggle: {other}");
        }
    }

    Ok(enabled)
}

/// Get the current speed multiplier.
#[tauri::command]
pub fn get_speed_multiplier(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<f64> {
    let wiz = state.lock().unwrap();
    Ok(wiz.speed_multiplier)
}

/// Set the speed multiplier value.
#[tauri::command]
pub fn set_speed_multiplier(
    state: State<'_, Mutex<WizState>>,
    value: f64,
) -> CommandResult<f64> {
    let mut wiz = state.lock().unwrap();
    wiz.speed_multiplier = value;
    tracing::info!("Speed multiplier set to {value}");
    Ok(value)
}
