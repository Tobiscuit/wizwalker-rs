//! Navigation and teleportation commands (synchronous).

use std::sync::Mutex;

use tauri::State;

use wizwalker::memory::reader::MemoryReaderExt;

use crate::state::{CommandError, CommandResult, Position, WizState};

/// Get the current XYZ position of the active client.
#[tauri::command]
pub fn get_position(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<Position> {
    let wiz = state.lock().unwrap();
    let label = WizState::client_label(wiz.active_client_idx);

    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::NoClients("No active client connected".into())
    })?;

    let client = client_arc.blocking_lock();

    // Read position from PlayerHook export → player base → offset 88 (XYZ)
    if let Ok(player_base) = client.hook_handler.read_current_player_base() {
        if let Some(reader) = client.reader() {
            let x = reader.read_typed::<f32>(player_base + 88).unwrap_or(0.0);
            let y = reader.read_typed::<f32>(player_base + 92).unwrap_or(0.0);
            let z = reader.read_typed::<f32>(player_base + 96).unwrap_or(0.0);
            return Ok(Position { x, y, z });
        }
    }

    Ok(Position::default())
}

/// Teleport the active client to the specified XYZ coordinates.
///
/// Writes to the teleport helper export (position + should_update flag).
/// Python equivalent: `teleport_helper.write_position(xyz); teleport_helper.write_should_update(True)`
#[tauri::command]
pub fn teleport_to(
    state: State<'_, Mutex<WizState>>,
    x: f32,
    y: f32,
    z: f32,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();
    let label = WizState::client_label(wiz.active_client_idx);

    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::NoClients("No active client connected".into())
    })?;

    let client = client_arc.blocking_lock();

    let addr = client.hook_handler.read_teleport_helper().map_err(|e| {
        CommandError::HookError(format!("Teleport hook not active: {e}"))
    })?;

    let reader = client.reader().ok_or_else(|| {
        CommandError::MemoryError("Client not opened".into())
    })?;

    reader.write_typed(addr, &x).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write X: {e}"))
    })?;
    reader.write_typed(addr + 4, &y).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write Y: {e}"))
    })?;
    reader.write_typed(addr + 8, &z).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write Z: {e}"))
    })?;
    // Set should_update flag (offset 12 per Python teleport_helper.py)
    reader.write_typed(addr + 12, &1u8).map_err(|e| {
        CommandError::MemoryError(format!("Failed to set update flag: {e}"))
    })?;

    tracing::info!("Teleported {label} to ({x}, {y}, {z})");
    Ok(())
}

/// Teleport all connected clients to the foreground client's position.
///
/// Python equivalent: reads foreground client's body.position(), then
/// writes that position to all other clients' teleport helpers.
#[tauri::command]
pub fn xyz_sync(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();

    // Find the foreground client and read its position
    let mut fg_pos: Option<(f32, f32, f32)> = None;
    let mut fg_label = String::new();

    for (label, client_arc) in &wiz.clients {
        if let Ok(client) = client_arc.try_lock() {
            if client.is_foreground() {
                if let Ok(player_base) = client.hook_handler.read_current_player_base() {
                    if let Some(reader) = client.reader() {
                        let x = reader.read_typed::<f32>(player_base + 88).unwrap_or(0.0);
                        let y = reader.read_typed::<f32>(player_base + 92).unwrap_or(0.0);
                        let z = reader.read_typed::<f32>(player_base + 96).unwrap_or(0.0);
                        fg_pos = Some((x, y, z));
                        fg_label = label.clone();
                    }
                }
                break;
            }
        }
    }

    let (x, y, z) = fg_pos.ok_or_else(|| {
        CommandError::NoClients("No foreground client found".into())
    })?;

    // Teleport all other clients to that position
    let mut synced = 0u32;
    for (label, client_arc) in &wiz.clients {
        if *label == fg_label {
            continue;
        }
        if let Ok(client) = client_arc.try_lock() {
            if let Ok(addr) = client.hook_handler.read_teleport_helper() {
                if let Some(reader) = client.reader() {
                    let _ = reader.write_typed(addr, &x);
                    let _ = reader.write_typed(addr + 4, &y);
                    let _ = reader.write_typed(addr + 8, &z);
                    let _ = reader.write_typed(addr + 12, &1u8);
                    synced += 1;
                }
            }
        }
    }

    tracing::info!("XYZ Sync: teleported {synced} clients to {fg_label}'s position ({x}, {y}, {z})");
    Ok(())
}
