//! Hook toggle and feature control commands.
//!
//! Synchronous pattern per Tauri v2 docs: `State<'_, Mutex<T>>` + `.lock().unwrap()`.

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::State;

use crate::state::{CommandResult, WizState};

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
    // Features without Rust implementations yet are log-only stubs.
    match name.as_str() {
        "speedhack" => {
            tracing::info!("Speedhack toggled: {enabled}");
        }
        "auto_combat" => {
            tracing::info!("Auto Combat toggled: {enabled}");
        }
        "auto_dialogue" => {
            tracing::info!("Auto Dialogue toggled: {enabled} (stub)");
        }
        "auto_sigil" => {
            tracing::info!("Auto Sigil toggled: {enabled} (stub)");
        }
        "auto_questing" => {
            tracing::info!("Auto Questing toggled: {enabled} (stub)");
        }
        "pet_trainer" => {
            tracing::info!("Pet Trainer toggled: {enabled} (stub)");
        }
        "auto_potions" => {
            tracing::info!("Auto Potions toggled: {enabled} (stub)");
        }
        "anti_afk" => {
            tracing::info!("Anti-AFK toggled: {enabled} (stub)");
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
