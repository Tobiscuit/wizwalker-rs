//! Dance Game Hook — Ported from Deimos `src/dance_game_hook.py`.
//!
//! Handles activation and reading of the pet dance game memory hook.

use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error};

use wizwalker::client::Client;
use wizwalker::memory::hooks::HookType;

pub async fn attempt_activate_dance_hook(client: &Client) {
    if !client.dance_hook_status.load(std::sync::atomic::Ordering::Relaxed) {
        match client.hook_handler.activate_hook(HookType::DanceGameMoves).await {
            Ok(_) => {
                client.dance_hook_status.store(true, std::sync::atomic::Ordering::Relaxed);
                debug!("Dance hook activated for client {}", client.title());
            }
            Err(e) => {
                debug!("Failed to activate dance hook for client {}: {}", client.title(), e);
            }
        }
    }
    sleep(Duration::from_millis(100)).await;
}

pub async fn attempt_deactivate_dance_hook(client: &Client) {
    if client.dance_hook_status.load(std::sync::atomic::Ordering::Relaxed) {
        match client.hook_handler.deactivate_hook(HookType::DanceGameMoves).await {
            Ok(_) => {
                client.dance_hook_status.store(false, std::sync::atomic::Ordering::Relaxed);
                debug!("Dance hook deactivated for client {}", client.title());
            }
            Err(e) => {
                debug!("Failed to deactivate dance hook for client {}: {}", client.title(), e);
            }
        }
    }
    sleep(Duration::from_millis(100)).await;
}

pub async fn read_current_dance_game_moves(client: &Client) -> Result<String, String> {
    let moves_raw = client.hook_handler.read_hook_export(HookType::DanceGameMoves, "dance_game_moves")
        .await
        .map_err(|e| e.to_string())?;

    // In Python: moves.partition(b"\0")[0].decode().translate(_dance_moves_transtable)
    // _dance_moves_transtable = str.maketrans("abcd", "WDSA")
    let moves_str = String::from_utf8_lossy(&moves_raw)
        .split('\0')
        .next()
        .unwrap_or_default()
        .to_string();

    let translated: String = moves_str.chars()
        .map(|c| match c {
            'a' => 'W',
            'b' => 'D',
            'c' => 'S',
            'd' => 'A',
            _ => c,
        })
        .collect();

    Ok(translated)
}
