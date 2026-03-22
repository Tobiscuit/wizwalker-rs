//! Utility functions — faithfully ported from Deimos `src/utils.py`.
//!
//! Core functions used by all automation loops:
//! - `get_window_from_path` — navigate UI tree by name path
//! - `is_visible_by_path` — check if a window path is visible
//! - `click_window_by_path` — click a window found by path
//! - `wait_for_zone_change` — wait until the zone name changes
//! - `is_free` — check if the player is free (not loading/fighting/dialogue)

use wizwalker::client::Client;
use wizwalker::memory::objects::window::DynamicWindow;

use super::paths;

// ── Window Path Navigation ──────────────────────────────────────────

/// Navigate the game's UI window tree by following a name path.
///
/// Each element in `path` is the name of a child window to follow.
/// An empty string `""` matches any child (wildcard).
///
/// Returns `None` if the path cannot be resolved (window doesn't exist).
///
/// # Python equivalent
/// ```python
/// async def get_window_from_path(root_window, name_path):
///     async def _recurse_follow_path(window, path):
///         if len(path) == 0:
///             return window
///         for child in await window.children():
///             if await child.name() == path[0]:
///                 found_window = await _recurse_follow_path(child, path[1:])
///                 if not found_window is False:
///                     return found_window
///         return False
///     return await _recurse_follow_path(root_window, name_path)
/// ```
pub fn get_window_from_path(root: &DynamicWindow, path: &[&str]) -> Option<DynamicWindow> {
    if path.is_empty() {
        return Some(root.clone());
    }

    let children = root.children().ok()?;
    let target_name = path[0];

    for child in children {
        let child_name = child.name().unwrap_or_default();

        // Empty string in path means "match any child"
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
///
/// Returns `false` if the window doesn't exist or isn't visible.
///
/// # Python equivalent
/// ```python
/// async def is_visible_by_path(client, path):
///     root = client.root_window
///     windows = await get_window_from_path(root, path)
///     if windows == False:
///         return False
///     elif await windows.is_visible():
///         return True
///     else:
///         return False
/// ```
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
///
/// Uses the mouse handler to click the center of the found window.
///
/// # Python equivalent
/// ```python
/// async def click_window_by_path(client, path, hooks=False):
///     root = client.root_window
///     windows = await get_window_from_path(root, path)
///     if windows:
///         async with client.mouse_handler:
///             await client.mouse_handler.click_window(windows)
/// ```
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
                    tracing::warn!("click_window_by_path failed: {e}");
                    false
                }
            }
        }
        None => false,
    }
}

/// Read text from a window found by path.
///
/// # Python equivalent
/// ```python
/// async def text_from_path(client, path):
///     window = await get_window_from_path(client.root_window, path)
///     return await window.maybe_text()
/// ```
pub fn text_from_path(client: &Client, path: &[&str]) -> Option<String> {
    let root = match &client.root_window {
        Some(rw) => &rw.window,
        None => return None,
    };

    let window = get_window_from_path(root, path)?;
    window.maybe_text().ok()
}

// ── Zone Change Detection ───────────────────────────────────────────

/// Wait for the zone to change from the current zone.
///
/// Polls `client.zone_name()` every 100ms until it differs from `current_zone`.
/// If `current_zone` is `None`, reads the current zone first.
///
/// # Python equivalent
/// ```python
/// async def wait_for_zone_change(client, current_zone=None, to_zone=None, loading_only=False):
///     if current_zone is None:
///         current_zone = await client.zone_name()
///     while current_zone == await client.zone_name():
///         await asyncio.sleep(0.1)
///     while await client.is_loading():
///         await asyncio.sleep(0.1)
/// ```
pub fn wait_for_zone_change(client: &Client, current_zone: Option<&str>, timeout_secs: f64) -> bool {
    let start_zone = match current_zone {
        Some(z) => z.to_string(),
        None => client.zone_name().unwrap_or_default(),
    };

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs_f64(timeout_secs);

    // Wait until zone changes
    loop {
        if start.elapsed() > timeout {
            tracing::warn!("wait_for_zone_change timed out after {timeout_secs}s");
            return false;
        }

        let current = client.zone_name().unwrap_or_default();
        if current != start_zone {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Wait until no longer loading
    while client.is_loading() {
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    true
}

/// Wait for a loading screen to start, then wait for it to finish.
///
/// # Python equivalent
/// ```python
/// async def wait_for_loading_screen(client):
///     while not await client.is_loading():
///         await asyncio.sleep(0.1)
///     while await client.is_loading():
///         await asyncio.sleep(0.1)
/// ```
pub fn wait_for_loading_screen(client: &Client, timeout_secs: f64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs_f64(timeout_secs);

    // Wait for loading to start
    while !client.is_loading() {
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Wait for loading to end
    while client.is_loading() {
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    true
}

// ── Player State Checks ─────────────────────────────────────────────

/// Returns `true` if the player is free (not in combat, loading, or dialogue).
///
/// # Python equivalent
/// ```python
/// async def is_free(client):
///     return not any([await client.is_loading(), await client.in_battle(),
///                     await is_visible_by_path(client, advance_dialog_path)])
/// ```
pub fn is_free(client: &Client) -> bool {
    !client.is_loading()
        && !client.in_battle()
        && !is_visible_by_path(client, paths::ADVANCE_DIALOG)
}

/// Get the quest objective text from the HUD.
///
/// # Python equivalent
/// ```python
/// async def get_quest_name(client):
///     quest_name_window = await get_window_from_path(root, quest_name_path)
///     quest_objective = await quest_name_window.maybe_text()
///     quest_objective = quest_objective.replace('<center>', '').replace('</center>', '')
///     return quest_objective
/// ```
pub fn get_quest_name(client: &Client) -> Option<String> {
    let text = text_from_path(client, paths::QUEST_NAME)?;
    Some(text.replace("<center>", "").replace("</center>", ""))
}

/// Get the NPC popup title text.
pub fn get_popup_title(client: &Client) -> Option<String> {
    let text = text_from_path(client, paths::POPUP_TITLE)?;
    Some(text.replace("<center>", "").replace("</center>", ""))
}

/// Whether the player is in NPC range (the NPC popup window is visible).
pub fn is_in_npc_range(client: &Client) -> bool {
    is_visible_by_path(client, paths::NPC_RANGE)
}

/// Try to close NPC and shop menus by clicking each provided path.
///
/// Python: `async def exit_menus(client, paths_to_try)`
pub async fn exit_menus(client: &Client, menu_paths: &[&[&str]]) {
    for path in menu_paths {
        if is_visible_by_path(client, path) {
            click_window_by_path(client, path).await;
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }
}

/// Read the NPC Range popup message text.
pub fn read_popup_message(client: &Client) -> String {
    text_from_path(client, paths::POPUP_MSGTEXT).unwrap_or_default()
}

