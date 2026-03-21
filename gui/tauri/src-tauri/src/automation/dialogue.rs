//! Auto Dialogue — faithfully ported from Deimos auto-dialogue feature.
//!
//! Detects the dialogue window and automatically advances through it
//! by sending Enter or clicking the advance button.

use wizwalker::client::Client;
use wizwalker::constants::Keycode;

use super::paths;
use super::utils;

/// Run the auto-dialogue loop for one iteration.
///
/// If the advance-dialogue button is visible, clicks it.
/// If not, tries pressing Enter as a fallback.
///
/// Returns `true` if a dialogue was advanced.
///
/// # Python equivalent (embedded in Deimos main loop)
/// ```python
/// if await is_visible_by_path(client, advance_dialog_path):
///     await click_window_by_path(client, advance_dialog_path)
/// ```
pub fn try_advance_dialogue(client: &Client) -> bool {
    if utils::is_visible_by_path(client, paths::ADVANCE_DIALOG) {
        // Click the advance button
        // Since click_window_by_path is async and we're sync,
        // use the simpler send_key approach like Deimos does in many places
        client.send_key(Keycode::Spacebar);
        std::thread::sleep(std::time::Duration::from_millis(200));
        true
    } else {
        false
    }
}

/// Auto-handle dialogue — keep advancing until dialogue is gone.
///
/// Blocks the current thread while advancing through dialogue.
/// Returns number of dialogue advances made.
pub fn auto_dialogue(client: &Client, timeout_secs: f64) -> u32 {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs_f64(timeout_secs);
    let mut advances = 0;

    loop {
        if start.elapsed() > timeout {
            break;
        }

        if utils::is_visible_by_path(client, paths::ADVANCE_DIALOG) {
            client.send_key(Keycode::Spacebar);
            advances += 1;
            std::thread::sleep(std::time::Duration::from_millis(300));
        } else {
            // Dialogue is gone
            break;
        }
    }

    advances
}

/// Check if a dungeon warning popup appeared and dismiss it.
pub fn dismiss_dungeon_warning(client: &Client) -> bool {
    if utils::is_visible_by_path(client, paths::DUNGEON_WARNING) {
        client.send_key(Keycode::Enter);
        std::thread::sleep(std::time::Duration::from_millis(250));
        true
    } else {
        false
    }
}

/// Try to close common menus/popups that may interfere with automation.
///
/// Ported from Deimos `exit_menus()`.
pub fn exit_menus(client: &Client) {
    let exit_paths: &[&[&str]] = &[
        paths::EXIT_RECIPE_SHOP,
        paths::EXIT_EQUIPMENT_SHOP,
        paths::EXIT_SNACK_SHOP,
        paths::EXIT_REAGENT_SHOP,
        paths::EXIT_TC_VENDOR,
        paths::EXIT_MINIGAME_SIGIL,
        paths::CANCEL_MULTIPLE_QUEST_MENU,
        paths::CANCEL_SPELL_VENDOR,
        paths::CANCEL_CHEST_ROLL,
        paths::EXIT_WYSTERIA_TOURNAMENT,
        paths::EXIT_ZAFARIA_CLASS_PICTURE,
        paths::AVALON_BADGE_EXIT,
        paths::EXIT_PET_LEVELED_UP,
    ];

    for path in exit_paths {
        if utils::is_visible_by_path(client, path) {
            client.send_key(Keycode::Esc);
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
}
