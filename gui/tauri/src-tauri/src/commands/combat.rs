//! Combat status and stat viewer commands (synchronous).

use std::sync::Mutex;

use tauri::State;

use wizwalker::memory::objects::game_stats::GameStats;

use crate::state::{CommandError, CommandResult, PlayerStats, WizState};

/// Serializable combat status for the frontend.
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombatStatus {
    pub in_combat: bool,
    pub round_number: u32,
    pub cards_count: u32,
}

/// Serializable card info for the frontend.
#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardInfo {
    pub name: String,
    pub display_name: String,
    pub accuracy: u8,
    pub is_castable: bool,
    pub is_enchanted: bool,
    pub is_treasure_card: bool,
}

/// Get the combat status for the active client.
#[tauri::command]
pub fn get_combat_status(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<CombatStatus> {
    let wiz = state.lock().unwrap();
    let label = WizState::client_label(wiz.active_client_idx);

    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::NoClients("No active client connected".into())
    })?;

    let client = client_arc.blocking_lock();

    Ok(CombatStatus {
        in_combat: client.in_battle(),
        round_number: 0,
        cards_count: 0,
    })
}

/// Get the player stats from game memory.
#[tauri::command]
pub fn get_stats(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<PlayerStats> {
    let wiz = state.lock().unwrap();
    let label = WizState::client_label(wiz.active_client_idx);

    let client_arc = wiz.clients.get(&label).ok_or_else(|| {
        CommandError::NoClients("No active client connected".into())
    })?;

    let client = client_arc.blocking_lock();

    if let Some(game_stats) = client.stats() {
        return Ok(PlayerStats {
            max_health: game_stats.max_hitpoints().unwrap_or(0),
            max_mana: game_stats.max_mana().unwrap_or(0),
            power_pip_chance: 0.0,
            accuracy: 0.0,
            resist: 0.0,
            damage: 0.0,
            critical: 0,
            pierce: 0.0,
        });
    }

    Ok(PlayerStats::default())
}

/// Get the cards currently available in combat.
#[tauri::command]
pub fn get_cards(
    state: State<'_, Mutex<WizState>>,
) -> CommandResult<Vec<CardInfo>> {
    let _wiz = state.lock().unwrap();
    tracing::info!("get_cards called (stub)");
    Ok(vec![])
}
