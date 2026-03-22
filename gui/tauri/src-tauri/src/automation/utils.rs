//! Utility functions — faithfully ported from Deimos `src/utils.py`.
//!
//! Core functions used by all automation loops:
//! - `get_window_from_path` — navigate UI tree by name path
//! - `is_visible_by_path` — check if a window path is visible
//! - `click_window_by_path` — click a window found by path
//! - `wait_for_zone_change` — wait until the zone name changes
//! - `is_free` — check if the player is free (not loading/fighting/dialogue)

use std::time::Duration;
use tokio::time::sleep;
use wizwalker::client::Client;
use wizwalker::constants::Keycode;
use wizwalker::types::XYZ;
use wizwalker::memory::objects::window::DynamicWindow;
use wizwalker::memory::memory_object::MemoryObject;
use wizwalker::memory::reader::MemoryReaderExt;
use tracing::{debug, warn};
use regex::Regex;
use lazy_static::lazy_static;

use super::paths;

lazy_static! {
    static ref FRIEND_LIST_ENTRY: Regex = Regex::new(r"\[(?P<icon_index>\d+),(?P<icon_list>\d+),'(?P<name>[^']+)'\]").unwrap();
}

// ── Window Path Navigation ──────────────────────────────────────────

/// Navigate the game's UI window tree by following a name path.
pub fn get_window_from_path(root: &DynamicWindow, path: &[&str]) -> Option<DynamicWindow> {
    if path.is_empty() {
        return Some(root.clone());
    }

    let children = root.children().ok()?;
    let target_name = path[0];

    for child in children {
        let child_name = child.name().unwrap_or_default();
        let matches = target_name.is_empty() || child_name == target_name;

        if matches {
            if let Some(found) = get_window_from_path(&child, &path[1..]) {
                return Some(found);
            }
        }
    }

    None
}

/// Check if a window at the given path is visible.
pub fn is_visible_by_path(client: &Client, path: &[&str]) -> bool {
    let root = match &client.root_window {
        Some(rw) => &rw.window,
        None => return false,
    };

    match get_window_from_path(root, path) {
        Some(window) => window.is_visible().unwrap_or(false),
        None => false,
    }
}

/// Click a window found by navigating the UI tree path.
pub async fn click_window_by_path(client: &Client, path: &[&str]) -> bool {
    let root = match &client.root_window {
        Some(rw) => &rw.window,
        None => return false,
    };

    match get_window_from_path(root, path) {
        Some(window) => {
            let mouse = &client.mouse_handler;
            match mouse.click_window(&window, false).await {
                Ok(()) => true,
                Err(e) => {
                    warn!("click_window_by_path failed: {e}");
                    false
                }
            }
        }
        None => false,
    }
}

/// Read text from a window found by path.
pub fn text_from_path(client: &Client, path: &[&str]) -> Option<String> {
    let root = match &client.root_window {
        Some(rw) => &rw.window,
        None => return None,
    };

    let window = get_window_from_path(root, path)?;
    window.maybe_text().ok()
}

// ── Zone Change Detection ───────────────────────────────────────────

pub async fn wait_for_zone_change(client: &Client, current_zone: Option<&str>, timeout_secs: f64) -> bool {
    let start_zone = match current_zone {
        Some(z) => z.to_string(),
        None => client.zone_name().unwrap_or_default(),
    };

    let start = std::time::Instant::now();
    let timeout = Duration::from_secs_f64(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            warn!("wait_for_zone_change timed out after {timeout_secs}s");
            return false;
        }

        let current = client.zone_name().unwrap_or_default();
        if current != start_zone {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    while client.is_loading() {
        if start.elapsed() > timeout {
            return false;
        }
        sleep(Duration::from_millis(100)).await;
    }

    true
}

// ── Player State Checks ─────────────────────────────────────────────

pub fn is_free(client: &Client) -> bool {
    !client.is_loading()
        && !client.in_battle()
        && !is_visible_by_path(client, paths::ADVANCE_DIALOG)
}

pub fn get_quest_name(client: &Client) -> Option<String> {
    let text = text_from_path(client, paths::QUEST_NAME)?;
    Some(text.replace("<center>", "").replace("</center>", ""))
}

pub async fn exit_menus(client: &Client, menu_paths: &[&[&str]]) {
    for path in menu_paths {
        if is_visible_by_path(client, path) {
            click_window_by_path(client, path).await;
            sleep(Duration::from_millis(200)).await;
        }
    }
}

// ── Potion & Navigation Helpers ─────────────────────────────────────

pub async fn refill_potions(client: &Client, mark: bool, recall: bool) {
    if client.stats_reference_level().unwrap_or(0) < 6 { return; }
    if mark {
        if client.zone_name().unwrap_or_default() != "WizardCity/WC_Hub" {
            let _ = client.send_key(Keycode::PageDown);
            sleep(Duration::from_secs(2)).await;
        }
    }
    // Simple navigation to Hilda Brewer
    let hilda = XYZ { x: -4398.7, y: 1016.2, z: 229.0 };
    let _ = client.teleport(&hilda);
    sleep(Duration::from_secs(2)).await;

    // Open shop
    while !is_visible_by_path(client, paths::POTION_SHOP_BASE) {
        let _ = client.send_key(Keycode::X);
        sleep(Duration::from_millis(500)).await;
    }

    let _ = click_window_by_path(client, paths::POTION_FILL_ALL).await;
    sleep(Duration::from_millis(250)).await;
    let _ = click_window_by_path(client, paths::POTION_BUY).await;
    sleep(Duration::from_millis(250)).await;

    while is_visible_by_path(client, paths::POTION_SHOP_BASE) {
        let _ = click_window_by_path(client, paths::POTION_EXIT).await;
        sleep(Duration::from_millis(125)).await;
    }

    if recall {
        let _ = client.send_key(Keycode::PageUp);
        sleep(Duration::from_secs(1)).await;
        let _ = client.send_key(Keycode::PageUp);
    }
}

pub async fn refill_potions_if_needed(client: &Client) {
    if client.stats_potion_charge().unwrap_or(0) < 1 && client.stats_reference_level().unwrap_or(0) >= 6 {
        refill_potions(client, true, true).await;
    }
}

pub async fn logout_and_in(client: &Client) {
    let _ = client.send_key(Keycode::Escape);
    sleep(Duration::from_millis(500)).await;
    let root = match &client.root_window {
        Some(rw) => &rw.window,
        None => return,
    };
    if let Some(quit_btn) = get_window_from_path(root, paths::QUIT_BUTTON) {
        let _ = client.mouse_handler.click_window(&quit_btn, false).await;
    }
    sleep(Duration::from_secs(1)).await;
    if is_visible_by_path(client, paths::DUNGEON_WARNING) {
        let _ = client.send_key(Keycode::Enter);
    }
    // Wait for play button
    for _ in 0..60 {
        if is_visible_by_path(client, paths::PLAY_BUTTON) {
            let _ = click_window_by_path(client, paths::PLAY_BUTTON).await;
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
    sleep(Duration::from_secs(4)).await;
}

// ── Friendship Teleport ─────────────────────────────────────────────

pub async fn teleport_to_friend_from_list(
    client: &Client,
    name: Option<&str>,
    icon_list: Option<i32>,
    icon_index: Option<i32>
) -> Result<(), String> {
    let root = client.root_window.as_ref().map(|rw| &rw.window).ok_or("No root window")?;
    let mut friends_window = get_window_from_path(root, &["NewFriendsListWindow"]);
    if friends_window.is_none() || !friends_window.as_ref().unwrap().is_visible().unwrap_or(false) {
        if let Some(btn) = get_window_from_path(root, &["btnFriends"]) {
            let _ = client.mouse_handler.click_window(&btn, false).await;
            sleep(Duration::from_millis(400)).await;
            friends_window = get_window_from_path(root, &["NewFriendsListWindow"]);
        }
    }

    let friends_window = friends_window.ok_or("Could not open friends list")?;
    let list_friends = get_window_from_path(&friends_window, &["listFriends"]).ok_or("No listFriends window")?;
    let list_text = list_friends.maybe_text().unwrap_or_default();

    let mut match_idx: Option<usize> = None;
    for (idx, entry) in FRIEND_LIST_ENTRY.captures_iter(&list_text).enumerate() {
        let f_icon = entry["icon_index"].parse::<i32>().unwrap_or(-1);
        let f_list = entry["icon_list"].parse::<i32>().unwrap_or(-1);
        let f_name = entry["name"].to_lowercase();

        if let Some(n) = name {
            if f_name == n.to_lowercase() {
                if icon_list.map_or(true, |l| l == f_list) && icon_index.map_or(true, |i| i == f_icon) {
                    match_idx = Some(idx);
                    break;
                }
            }
        } else if let (Some(l), Some(i)) = (icon_list, icon_index) {
            if f_list == l && f_icon == i {
                match_idx = Some(idx);
                break;
            }
        }
    }

    let idx = match_idx.ok_or("Friend not found in list")?;
    
    // Select friend
    let mouse = &client.mouse_handler;
    // BUG: (from Python original) assumes 10 friends per page.
    let page = (idx / 10) + 1;
    if let Some(btn_down) = get_window_from_path(&friends_window, &["btnArrowDown"]) {
        for _ in 1..page {
            let _ = mouse.click_window(&btn_down, false).await;
            sleep(Duration::from_millis(200)).await;
        }
    }

    // Click character and teleport
    if let Some(char_win) = get_window_from_path(root, &["wndCharacter"]) {
        if let Some(tp_btn) = get_window_from_path(&char_win, &["btnTeleport"]) {
             let _ = mouse.click_window(&tp_btn, false).await;
        }
    }

    Ok(())
}
