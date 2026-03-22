//! Pet Trainer — auto feeding & dance game automation.
//!
//! Faithfully ported from Deimos `src/auto_pet.py` (365 lines).
#![allow(dead_code, unused_imports)]

use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

use wizwalker::client::Client;
use wizwalker::constants::Keycode;
use wizwalker::types::XYZ;

use super::utils::{
    click_window_by_path, is_visible_by_path, text_from_path, get_popup_title,
};
use super::paths::*;

/// Navigate from the Wizard City commons to the Pet Pavilion.
pub async fn navigate_to_pavilion_from_commons(client: &Client) {
    let pavilion_xyz = XYZ {
        x: 8426.3779296875,
        y: -2165.6982421875,
        z: -27.913818359375,
    };
    let _ = client.teleport(&pavilion_xyz);
    sleep(Duration::from_secs(5)).await;
    client.wait_for_loading_screen().await;
    sleep(Duration::from_secs(2)).await;
}

/// Navigate within the pet pavilion to the dance game sigil.
pub async fn navigate_to_dance_game(client: &Client) {
    let _ = client.teleport(&XYZ { x: -1738.7811279296875, y: -387.345458984375, z: 0.0 });
    sleep(Duration::from_millis(500)).await;
    let _ = client.teleport(&XYZ { x: -4090.10888671875, y: -1186.3660888671875, z: 0.0 });
    sleep(Duration::from_millis(500)).await;
    let _ = client.teleport(&XYZ { x: -4449.36669921875, y: -992.9967651367188, z: 0.0 });
    sleep(Duration::from_millis(500)).await;
}

/// Play the dance game automatically.
pub async fn dancedance(client: &Client) {
    for _ in 0..50 {
        if is_visible_by_path(client, DANCE_GAME_ACTION_TEXTBOX_PATH) { break; }
        sleep(Duration::from_millis(100)).await;
    }

    for _ in 0..5 {
        for _ in 0..100 {
            let text = text_from_path(client, DANCE_GAME_ACTION_TEXTBOX_PATH).unwrap_or_default();
            if text.contains("Go!") { break; }
            sleep(Duration::from_millis(125)).await;
        }
        for _ in 0..100 {
            let text = text_from_path(client, DANCE_GAME_ACTION_TEXTBOX_PATH).unwrap_or_default();
            if !text.contains("Go!") { break; }
            sleep(Duration::from_millis(125)).await;
        }
        sleep(Duration::from_millis(1500)).await;

        match client.hook_handler.read_current_dance_game_moves() {
            Ok(moves) => {
                debug!("Dance moves: {moves}");
                for ch in moves.chars() {
                    let keycode = match ch {
                        'W' => Keycode::W,
                        'A' => Keycode::A,
                        'S' => Keycode::S,
                        'D' => Keycode::D,
                        _ => continue,
                    };
                    client.send_key(keycode);
                    sleep(Duration::from_millis(100)).await;
                }
            }
            Err(e) => debug!("Failed to read dance moves: {e}"),
        }
    }
    sleep(Duration::from_secs(3)).await;
}

/// Handle pet level-up window after winning dance game.
pub async fn won_game_leveled_up(client: &Client, ignore_pet_level_up: bool) {
    if is_visible_by_path(client, WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH) {
        click_window_by_path(client, WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH).await;
        sleep(Duration::from_secs(1)).await;
    }
    if is_visible_by_path(client, WON_PET_LEVELED_UP_WINDOW_PATH) {
        if !ignore_pet_level_up {
            info!("Pet leveled up! Close the window to continue.");
            while is_visible_by_path(client, WON_PET_LEVELED_UP_WINDOW_PATH) {
                sleep(Duration::from_secs(1)).await;
            }
        } else {
            while is_visible_by_path(client, WON_PET_LEVELED_UP_WINDOW_PATH) {
                if is_visible_by_path(client, EXIT_WON_PET_LEVELED_UP_PATH) {
                    click_window_by_path(client, EXIT_WON_PET_LEVELED_UP_PATH).await;
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }
}

/// Main pet feeding loop.
pub async fn nomnom(client: &mut Client, ignore_pet_level_up: bool, only_play_dance_game: bool) {
    let mut finished_feeding = false;
    let mut dance_hook_activated = false;

    while !finished_feeding {
        loop {
            let title = get_popup_title(client).unwrap_or_default();
            if title == "Dance Game" { break; }
            sleep(Duration::from_millis(125)).await;
        }

        while !is_visible_by_path(client, PET_FEED_WINDOW_VISIBLE_PATH) {
            let title = get_popup_title(client).unwrap_or_default();
            if title == "Dance Game" {
                client.send_key(Keycode::X);
                sleep(Duration::from_millis(125)).await;
            }
            sleep(Duration::from_millis(125)).await;
        }

        let energy_cost = get_pet_energy_cost(client).unwrap_or(999);
        let total_energy = get_pet_total_energy(client).unwrap_or(0);

        if total_energy >= energy_cost {
            let can_skip = is_visible_by_path(client, SKIP_PET_GAME_BUTTON_PATH);

            if (only_play_dance_game || !can_skip) && !dance_hook_activated {
                debug!("Activating dance game hook");
                let _ = client.hook_handler_mut().activate_dance_game_moves_hook();
                dance_hook_activated = true;
                sleep(Duration::from_secs(5)).await;
            }

            if can_skip && !only_play_dance_game {
                finished_feeding = skip_pet_game(client, ignore_pet_level_up).await;
            } else {
                finished_feeding = play_pet_game(client, ignore_pet_level_up).await;
            }
        } else {
            info!("Client is out of energy");
            finished_feeding = true;
        }
    }

    while is_visible_by_path(client, PET_FEED_WINDOW_VISIBLE_PATH) {
        if is_visible_by_path(client, PET_FEED_WINDOW_CANCEL_BUTTON_PATH) {
            click_window_by_path(client, PET_FEED_WINDOW_CANCEL_BUTTON_PATH).await;
            sleep(Duration::from_millis(200)).await;
        }
    }

    if dance_hook_activated {
        let _ = client.hook_handler_mut().deactivate_dance_game_moves_hook();
    }

    loop {
        let title = get_popup_title(client).unwrap_or_default();
        if title != "Dance Game" { break; }
        sleep(Duration::from_millis(125)).await;
    }
}

async fn skip_pet_game(client: &Client, ignore_pet_level_up: bool) -> bool {
    while is_visible_by_path(client, PET_FEED_WINDOW_VISIBLE_PATH) {
        if is_visible_by_path(client, SKIP_PET_GAME_BUTTON_PATH) {
            click_window_by_path(client, SKIP_PET_GAME_BUTTON_PATH).await;
            sleep(Duration::from_millis(200)).await;
        }
    }
    for _ in 0..100 {
        if is_visible_by_path(client, SKIPPED_PET_GAME_REWARDS_WINDOW_PATH) { break; }
        sleep(Duration::from_millis(100)).await;
    }
    if is_visible_by_path(client, SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH) {
        click_window_by_path(client, SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH).await;
        sleep(Duration::from_millis(1500)).await;
    }
    if is_visible_by_path(client, SKIPPED_FIRST_PET_SNACK_PATH) {
        click_window_by_path(client, SKIPPED_FIRST_PET_SNACK_PATH).await;
        sleep(Duration::from_millis(600)).await;
        if is_visible_by_path(client, SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH) {
            click_window_by_path(client, SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH).await;
            sleep(Duration::from_secs(1)).await;
            if is_visible_by_path(client, SKIPPED_PET_LEVELED_UP_WINDOW_PATH) {
                handle_skipped_pet_level_up(client, ignore_pet_level_up).await;
            }
            for _ in 0..100 {
                if is_visible_by_path(client, SKIPPED_FINISH_PET_BUTTON) { break; }
                sleep(Duration::from_millis(100)).await;
            }
            while is_visible_by_path(client, SKIPPED_FINISH_PET_BUTTON) {
                click_window_by_path(client, SKIPPED_FINISH_PET_BUTTON).await;
                sleep(Duration::from_millis(200)).await;
            }
            while is_visible_by_path(client, SKIPPED_PET_GAME_REWARDS_WINDOW_PATH) {
                sleep(Duration::from_millis(100)).await;
            }
        }
        sleep(Duration::from_millis(500)).await;
        false
    } else {
        info!("Client is out of snacks");
        true
    }
}

async fn play_pet_game(client: &Client, ignore_pet_level_up: bool) -> bool {
    while is_visible_by_path(client, PET_FEED_WINDOW_VISIBLE_PATH) {
        client.send_key(Keycode::X);
        if is_visible_by_path(client, WIZARD_CITY_DANCE_GAME_PATH) {
            click_window_by_path(client, WIZARD_CITY_DANCE_GAME_PATH).await;
            sleep(Duration::from_millis(200)).await;
        }
        if is_visible_by_path(client, PLAY_DANCE_GAME_BUTTON_PATH) {
            click_window_by_path(client, PLAY_DANCE_GAME_BUTTON_PATH).await;
            sleep(Duration::from_millis(100)).await;
        }
    }
    dancedance(client).await;
    if is_visible_by_path(client, WON_PET_LEVELED_UP_WINDOW_PATH) {
        won_game_leveled_up(client, ignore_pet_level_up).await;
    }
    if is_visible_by_path(client, WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH) {
        click_window_by_path(client, WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH).await;
        sleep(Duration::from_millis(1500)).await;
    }
    if is_visible_by_path(client, WON_FIRST_PET_SNACK_PATH) {
        click_window_by_path(client, WON_FIRST_PET_SNACK_PATH).await;
        sleep(Duration::from_millis(600)).await;
        if is_visible_by_path(client, WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH) {
            won_game_leveled_up(client, ignore_pet_level_up).await;
            for _ in 0..100 {
                if is_visible_by_path(client, WON_FINISH_PET_BUTTON) { break; }
                sleep(Duration::from_millis(100)).await;
            }
            while is_visible_by_path(client, WON_FINISH_PET_BUTTON) {
                click_window_by_path(client, WON_FINISH_PET_BUTTON).await;
                sleep(Duration::from_millis(200)).await;
            }
            while is_visible_by_path(client, WON_PET_GAME_REWARDS_WINDOW_PATH) {
                sleep(Duration::from_millis(100)).await;
            }
        }
        sleep(Duration::from_millis(500)).await;
        false
    } else {
        info!("Client is out of snacks");
        true
    }
}

async fn handle_skipped_pet_level_up(client: &Client, ignore_pet_level_up: bool) {
    if !ignore_pet_level_up {
        info!("Pet leveled up! Close the window to continue.");
        while is_visible_by_path(client, SKIPPED_PET_LEVELED_UP_WINDOW_PATH) {
            sleep(Duration::from_secs(1)).await;
        }
    } else {
        while is_visible_by_path(client, SKIPPED_PET_LEVELED_UP_WINDOW_PATH) {
            if is_visible_by_path(client, EXIT_SKIPPED_PET_LEVELED_UP_PATH) {
                click_window_by_path(client, EXIT_SKIPPED_PET_LEVELED_UP_PATH).await;
                sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

/// Full auto-pet flow: navigate to pavilion, feed, return.
pub async fn auto_pet(client: &mut Client, ignore_pet_level_up: bool, only_play_dance_game: bool) {
    let zone_name = client.zone_name().unwrap_or_default();
    let started_at_pavilion = zone_name == "WizardCity/WC_Streets/Interiors/WC_PET_Park";

    if !started_at_pavilion {
        let original_mana = client.stats_current_mana().unwrap_or(0);
        loop {
            client.send_key(Keycode::PageDown);
            sleep(Duration::from_millis(750)).await;
            let current = client.stats_current_mana().unwrap_or(0);
            if current < original_mana { break; }
        }
        sleep(Duration::from_millis(500)).await;
        navigate_to_pavilion_from_commons(client).await;
        navigate_to_dance_game(client).await;
    } else {
        let _ = client.teleport(&XYZ { x: -4450.57958984375, y: -994.8973388671875, z: -8.041412353515625 });
    }

    nomnom(client, ignore_pet_level_up, only_play_dance_game).await;
    sleep(Duration::from_secs(1)).await;

    if !started_at_pavilion {
        client.send_key(Keycode::PageUp);
        sleep(Duration::from_secs(5)).await;
        client.wait_for_loading_screen().await;
    }
}

fn get_pet_energy_cost(client: &Client) -> Option<i32> {
    let text = text_from_path(client, PET_FEED_WINDOW_ENERGY_COST_TEXTBOX_PATH)?;
    text.get(8..)?.trim().parse().ok()
}

fn get_pet_total_energy(client: &Client) -> Option<i32> {
    let text = text_from_path(client, PET_FEED_WINDOW_YOUR_ENERGY_TEXTBOX_PATH)?;
    let after_colon = text.get(8..)?;
    after_colon.split('/').next()?.trim().parse().ok()
}
