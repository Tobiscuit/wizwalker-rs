//! Camera read/write commands (synchronous).
//!
//! The camera controller is accessed via GameClient → selected_camera_controller
//! at a pattern-scanned offset. For now we use the hardcoded offset from Python
//! reference (0x22290 = 140944 from game client base).
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

/// Helper: get the camera controller base address for the active client.
///
/// Chain: GameClient base (from ClientHook) → offset 0x22290 → deref → camera controller
fn get_camera_base(wiz: &WizState) -> Result<(usize, std::sync::Arc<dyn wizwalker::memory::MemoryReader>), CommandError> {
    let label = WizState::client_label(wiz.active_client_idx);
    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::NoClients("No active client connected".into())
    })?;

    let client = client_arc.blocking_lock();

    // Get the GameClient base from the ClientHook export
    // Python: CurrentGameClient uses a pattern scan for its base, but the
    // client hook captures the client object address, not GameClient.
    // We need to get the selected_camera_controller from the GameClient.
    // The GameClient base is obtained via a separate pattern scan in Python.
    //
    // For now, try reading the render context which gives us the camera.
    let _render_base = client.hook_handler.read_current_render_context_base().map_err(|e| {
        CommandError::HookError(format!("RenderContext hook not active: {e}"))
    })?;

    let reader = client.reader().ok_or_else(|| {
        CommandError::MemoryError("Client not opened".into())
    })?;

    // The RenderContext doesn't directly give us the camera controller.
    // We need the GameClient → selected_camera_controller path.
    // Python offset: 0x22290 from GameClient base.
    // But our ClientHook gives us the client_object (CoreObject), not GameClient.
    //
    // Alternative approach: read the elastic_camera_controller from the
    // GameClient which is obtained via a separate pattern scan.
    // This requires a GameClient base address, which we don't have yet.
    //
    // Fallback: return the render context base and use it as a starting point.
    // TODO: Implement proper GameClient base resolution via pattern scan.
    Ok((_render_base, reader))
}

/// Get the current camera state.
#[tauri::command]
pub fn get_camera(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<CameraState> {
    let wiz = state.lock().unwrap();

    match get_camera_base(&wiz) {
        Ok((_base, _reader)) => {
            // TODO: Once GameClient base is resolved, read camera controller
            // For now return defaults — camera reading requires GameClient pattern scan
            Ok(CameraState::default())
        }
        Err(_) => Ok(CameraState::default()),
    }
}

/// Set the camera position.
#[tauri::command]
pub fn set_camera_position(
    state: State<'_, Mutex<WizState>>,
    x: f32,
    y: f32,
    z: f32,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();
    let (_base, _reader) = get_camera_base(&wiz)?;

    // TODO: Write to camera controller position (offset 108) once GameClient resolved
    tracing::info!("set_camera_position({x}, {y}, {z}) — needs GameClient base resolution");
    Ok(())
}

/// Set the camera field of view.
#[tauri::command]
pub fn set_camera_fov(
    state: State<'_, Mutex<WizState>>,
    fov: f32,
) -> CommandResult<()> {
    let _wiz = state.lock().unwrap();

    // TODO: Camera FOV is on the gamebryo_camera, offset from render context
    tracing::info!("set_camera_fov({fov}) — needs GameClient base resolution");
    Ok(())
}

/// Set the camera rotation (yaw, pitch, roll).
#[tauri::command]
pub fn set_camera_rotation(
    state: State<'_, Mutex<WizState>>,
    yaw: f32,
    pitch: f32,
    roll: f32,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();
    let (_base, _reader) = get_camera_base(&wiz)?;

    // TODO: Write pitch (120), roll (124), yaw (128) once GameClient resolved
    tracing::info!("set_camera_rotation(yaw={yaw}, pitch={pitch}, roll={roll}) — needs GameClient base");
    Ok(())
}
