//! Camera read/write commands (synchronous).
//!
//! Camera controller access chain (from Python game_client.py):
//!   GameClient base (ClientHook) → selected_camera_controller at +0x22290 → deref → camera
//!
//! Camera controller offsets (from Python camera_controller.py):
//!  - position: 108 (XYZ, 3x f32)
//!  - pitch: 120 (f32)
//!  - roll: 124 (f32)
//!  - yaw: 128 (f32)

use std::sync::Mutex;

use tauri::State;

use wizwalker::memory::reader::MemoryReaderExt;

use crate::state::{CameraState, CommandError, CommandResult, Position, WizState};

/// Helper: resolve the selected_camera_controller base address.
///
/// Chain: GameClient base (ClientHook) → [+0x22290] → camera controller pointer
/// Python: game_client.py selected_camera_controller() at offset 0x22290
fn resolve_camera_controller(wiz: &WizState) -> Result<(usize, std::sync::Arc<dyn wizwalker::memory::MemoryReader>), CommandError> {
    let label = WizState::client_label(wiz.active_client_idx);
    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::NoClients("No active client connected".into())
    })?;

    let client = client_arc.blocking_lock();

    // Get GameClient base from ClientHook export
    let client_base = client.hook_handler.read_current_client_base().map_err(|e| {
        CommandError::HookError(format!("ClientHook not active: {e}"))
    })?;

    let reader = client.reader().ok_or_else(|| {
        CommandError::MemoryError("Client not opened".into())
    })?;

    // Dereference GameClient + 0x22290 → selected_camera_controller*
    let camera_ptr: u64 = reader.read_typed(client_base + 0x22290).map_err(|e| {
        CommandError::MemoryError(format!("Failed to read camera controller pointer: {e}"))
    })?;

    if camera_ptr == 0 {
        return Err(CommandError::MemoryError("selected_camera_controller is null".into()));
    }

    Ok((camera_ptr as usize, reader))
}

/// Get the current camera state.
///
/// Python: reads position (108), pitch (120), roll (124), yaw (128) from camera controller
#[tauri::command]
pub fn get_camera(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<CameraState> {
    let wiz = state.lock().unwrap();

    match resolve_camera_controller(&wiz) {
        Ok((cam_base, reader)) => {
            let position = Position {
                x: reader.read_typed::<f32>(cam_base + 108).unwrap_or(0.0),
                y: reader.read_typed::<f32>(cam_base + 112).unwrap_or(0.0),
                z: reader.read_typed::<f32>(cam_base + 116).unwrap_or(0.0),
            };
            let pitch = reader.read_typed::<f32>(cam_base + 120).unwrap_or(0.0);
            let roll = reader.read_typed::<f32>(cam_base + 124).unwrap_or(0.0);
            let yaw = reader.read_typed::<f32>(cam_base + 128).unwrap_or(0.0);

            Ok(CameraState {
                position,
                pitch,
                roll,
                yaw,
                fov: 0.0, // FOV is on a different object (gamebryo camera)
                distance: 0.0, // Distance is on elastic camera controller
            })
        }
        Err(_) => Ok(CameraState::default()),
    }
}

/// Set the camera position.
///
/// Python: camera_controller.write_position(xyz) at offsets 108, 112, 116
#[tauri::command]
pub fn set_camera_position(
    state: State<'_, Mutex<WizState>>,
    x: f32,
    y: f32,
    z: f32,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();
    let (cam_base, reader) = resolve_camera_controller(&wiz)?;

    reader.write_typed(cam_base + 108, &x).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write camera X: {e}"))
    })?;
    reader.write_typed(cam_base + 112, &y).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write camera Y: {e}"))
    })?;
    reader.write_typed(cam_base + 116, &z).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write camera Z: {e}"))
    })?;

    tracing::info!("Camera position set to ({x}, {y}, {z})");
    Ok(())
}

/// Set the camera field of view.
///
/// Note: FOV is on the gamebryo camera, which is a different object.
/// For now this is a stub — full FOV requires finding the NiCamera object.
#[tauri::command]
pub fn set_camera_fov(
    state: State<'_, Mutex<WizState>>,
    fov: f32,
) -> CommandResult<()> {
    let _wiz = state.lock().unwrap();
    // FOV is on the gamebryo NiCamera, not the camera controller
    tracing::info!("set_camera_fov({fov}) — requires NiCamera object resolution");
    Ok(())
}

/// Set the camera rotation (yaw, pitch, roll).
///
/// Python: camera_controller pitch=120, roll=124, yaw=128
#[tauri::command]
pub fn set_camera_rotation(
    state: State<'_, Mutex<WizState>>,
    yaw: f32,
    pitch: f32,
    roll: f32,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();
    let (cam_base, reader) = resolve_camera_controller(&wiz)?;

    reader.write_typed(cam_base + 120, &pitch).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write camera pitch: {e}"))
    })?;
    reader.write_typed(cam_base + 124, &roll).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write camera roll: {e}"))
    })?;
    reader.write_typed(cam_base + 128, &yaw).map_err(|e| {
        CommandError::MemoryError(format!("Failed to write camera yaw: {e}"))
    })?;

    tracing::info!("Camera rotation set to (yaw={yaw}, pitch={pitch}, roll={roll})");
    Ok(())
}
