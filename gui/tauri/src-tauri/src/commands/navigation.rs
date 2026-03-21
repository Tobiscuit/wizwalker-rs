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

    // Read the teleport helper export address from the hook handler.
    if let Ok(addr) = client.hook_handler.read_teleport_helper() {
        if let Some(reader) = client.reader() {
            let x = reader.read_typed::<f32>(addr).unwrap_or(0.0);
            let y = reader.read_typed::<f32>(addr + 4).unwrap_or(0.0);
            let z = reader.read_typed::<f32>(addr + 8).unwrap_or(0.0);
            return Ok(Position { x, y, z });
        }
    }

    Ok(Position::default())
}

/// Teleport the active client to the specified XYZ coordinates.
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
    // Set should_update flag (offset 12)
    reader.write_typed(addr + 12, &1u8).map_err(|e| {
        CommandError::MemoryError(format!("Failed to set update flag: {e}"))
    })?;

    tracing::info!("Teleported {label} to ({x}, {y}, {z})");
    Ok(())
}

/// Teleport all connected clients to the foreground client's position.
#[tauri::command]
pub fn xyz_sync(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<()> {
    let _wiz = state.lock().unwrap();
    tracing::info!("XYZ Sync triggered (stub)");
    Ok(())
}
