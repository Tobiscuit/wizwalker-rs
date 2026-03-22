//! Client management commands — scan, list, connect, disconnect.
//!
//! All commands are **synchronous** (non-async) because:
//! - Win32 operations (EnumWindows, ReadProcessMemory) are inherently blocking
//! - ClientHandler methods are all sync (Option A architecture)
//! - Per Tauri v2 docs: sync commands use `State<'_, Mutex<T>>` with `.lock().unwrap()`

use std::sync::Mutex;

use tauri::State;

use wizwalker::memory::reader::MemoryReaderExt;

use crate::state::{ClientInfo, CommandError, CommandResult, WizState};

/// Helper: build ClientInfo from a locked Client.
fn build_info(label: &str, client: &wizwalker::client::Client) -> ClientInfo {
    ClientInfo {
        label: label.to_string(),
        pid: client.process_id,
        title: client.title(),
        hooked: client.hook_handler.has_any_hooks(),
        zone: client.zone_name().unwrap_or_else(|| "Unknown".into()),
        is_foreground: client.is_foreground(),
        is_running: client.is_running(),
    }
}

/// Scan for new Wizard101 game instances and add them to the handler.
///
/// ClientHandler::get_new_clients() is fully synchronous (EnumWindows + OpenProcess).
/// No async bridging needed.
#[tauri::command]
pub fn scan_clients(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<Vec<ClientInfo>> {
    let mut wiz = state.lock().unwrap();

    // Direct sync call — no block_on, no block_in_place needed.
    let new_clients = wiz.handler.get_new_clients().unwrap_or_default();

    for client_arc in new_clients {
        let idx = wiz.clients.len();
        let label = WizState::client_label(idx);
        wiz.clients.insert(label, client_arc);
    }

    // Remove dead clients (also sync).
    wiz.handler.remove_dead_clients();

    // Build client info list.
    let mut infos = Vec::new();
    for (label, client_arc) in &wiz.clients {
        if let Ok(client) = client_arc.try_lock() {
            infos.push(build_info(label, &client));
        }
    }
    infos.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(infos)
}

/// Get the list of currently connected clients without scanning.
#[tauri::command]
pub fn get_clients(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<Vec<ClientInfo>> {
    let wiz = state.lock().unwrap();

    let mut infos = Vec::new();
    for (label, client_arc) in &wiz.clients {
        if let Ok(client) = client_arc.try_lock() {
            infos.push(build_info(label, &client));
        }
    }
    infos.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(infos)
}

/// Open a client (attach memory reader) by its label.
#[tauri::command]
pub fn open_client(
    state: State<'_, Mutex<WizState>>,
    label: String,
) -> CommandResult<ClientInfo> {
    let wiz = state.lock().unwrap();

    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::ClientNotFound(format!("No client with label '{label}'"))
    })?;

    let mut client = client_arc.blocking_lock();
    client.open().map_err(|e| {
        CommandError::MemoryError(format!("Failed to open client {label}: {e}"))
    })?;

    Ok(build_info(&label, &client))
}

/// Activate all memory hooks for a client.
#[tauri::command]
pub fn activate_hooks(
    state: State<'_, Mutex<WizState>>,
    label: String,
) -> CommandResult<()> {
    let wiz = state.lock().unwrap();

    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::ClientNotFound(format!("No client with label '{label}'"))
    })?;

    let mut client = client_arc.blocking_lock();
    client.activate_hooks().map_err(|e| {
        CommandError::HookError(format!("Failed to activate hooks for {label}: {e}"))
    })?;

    tracing::info!("Hooks activated for client {label}");
    Ok(())
}

/// Close a client (unhook, release memory reader).
#[tauri::command]
pub fn close_client(
    state: State<'_, Mutex<WizState>>,
    label: String,
) -> CommandResult<()> {
    let mut wiz = state.lock().unwrap();

    if let Some(client_arc) = wiz.clients.remove(&label) {
        let mut client = client_arc.blocking_lock();

        // Python Deimos.py:489-517 tool_finish() cleanup:
        // 1. Restore original speed multiplier to normal (0 = 1x)
        if let Ok(client_base) = client.hook_handler.read_current_client_base() {
            if let Some(reader) = client.process_reader() {
                let client_obj_ptr: u64 = reader.read_typed(client_base + 0x21318).unwrap_or(0);
                if client_obj_ptr != 0 {
                    let normal: i16 = 0;
                    let _ = reader.write_typed::<i16>(client_obj_ptr as usize + 192, &normal);
                }
            }
        }

        // 2. Reset window title to "Wizard101"
        client.set_title("Wizard101");

        // 3. Close (unhook + release)
        client.close();
        tracing::info!("Client {label} closed with cleanup (speed restored, title reset)");
    }

    Ok(())
}
