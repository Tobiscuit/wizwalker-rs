//! Camera control commands (synchronous).

use std::sync::Mutex;

use tauri::State;

use crate::state::{CameraState, CommandResult, WizState};

/// Get the current camera state from game memory.
#[tauri::command]
pub fn get_camera(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<CameraState> {
    let _wiz = state.lock().unwrap();
    tracing::info!("get_camera called (stub)");
    Ok(CameraState::default())
}

/// Set the camera position.
#[tauri::command]
pub fn set_camera_position(
    state: State<'_, Mutex<WizState>>,
    x: f32,
    y: f32,
    z: f32,
) -> CommandResult<()> {
    let _wiz = state.lock().unwrap();
    tracing::info!("set_camera_position({x}, {y}, {z}) (stub)");
    Ok(())
}

/// Set the camera field of view.
#[tauri::command]
pub fn set_camera_fov(
    state: State<'_, Mutex<WizState>>,
    fov: f32,
) -> CommandResult<()> {
    let _wiz = state.lock().unwrap();
    tracing::info!("set_camera_fov({fov}) (stub)");
    Ok(())
}

/// Set the camera rotation.
#[tauri::command]
pub fn set_camera_rotation(
    state: State<'_, Mutex<WizState>>,
    yaw: f32,
    pitch: f32,
    roll: f32,
) -> CommandResult<()> {
    let _wiz = state.lock().unwrap();
    tracing::info!("set_camera_rotation(yaw={yaw}, pitch={pitch}, roll={roll}) (stub)");
    Ok(())
}
