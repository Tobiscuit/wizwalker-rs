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
        let mut hh = client.hook_handler.clone();
        match hh.activate_dance_game_moves_hook() {
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
        let mut hh = client.hook_handler.clone();
        match hh.deactivate_dance_game_moves_hook() {
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
    client.hook_handler.read_current_dance_game_moves().map_err(|e| e.to_string())
}
