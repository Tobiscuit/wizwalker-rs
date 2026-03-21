//! Auto Sigil — faithfully ported from Deimos `src/sigil.py`.
//!
//! Records a sigil position, waits for the team-up button,
//! joins the sigil, fights, and returns.

use wizwalker::client::Client;
use wizwalker::constants::Keycode;

use super::paths;
use super::utils;

/// Recorded sigil location for farming.
pub struct SigilState {
    pub sigil_x: f32,
    pub sigil_y: f32,
    pub sigil_z: f32,
    pub sigil_zone: String,
    pub original_quest: String,
}

impl SigilState {
    /// Record the current position and zone as the sigil location.
    ///
    /// # Python equivalent
    /// ```python
    /// async def record_sigil(self):
    ///     self.sigil_xyz = await self.client.body.position()
    ///     self.sigil_zone = await self.client.zone_name()
    /// ```
    pub fn record(client: &Client) -> Option<Self> {
        let reader = client.process_reader()?;
        let player_base = client.hook_handler.read_current_player_base().ok()?;

        use wizwalker::memory::reader::MemoryReaderExt;
        let x: f32 = reader.read_typed(player_base + 88).ok()?;
        let y: f32 = reader.read_typed(player_base + 92).ok()?;
        let z: f32 = reader.read_typed(player_base + 96).ok()?;
        let zone = client.zone_name()?;
        let quest = utils::get_quest_name(client).unwrap_or_default();

        Some(Self {
            sigil_x: x,
            sigil_y: y,
            sigil_z: z,
            sigil_zone: zone,
            original_quest: quest,
        })
    }
}

/// Join a sigil — sends X key to interact, handles dungeon warning.
///
/// # Python equivalent
/// ```python
/// async def join_sigil(self, client):
///     current_zone = await client.zone_name()
///     await client.send_key(Keycode.X, seconds=0.1)
///     if await is_visible_by_path(client, dungeon_warning_path):
///         await client.send_key(Keycode.ENTER, 0.1)
///     await wait_for_zone_change(client, current_zone=current_zone)
/// ```
pub fn join_sigil(client: &Client) -> bool {
    let current_zone = client.zone_name().unwrap_or_default();

    // Send X to interact with the sigil
    client.send_key(Keycode::X);
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Handle dungeon warning popup
    if utils::is_visible_by_path(client, paths::DUNGEON_WARNING) {
        client.send_key(Keycode::Enter);
    }

    // Wait for zone change
    utils::wait_for_zone_change(client, Some(&current_zone), 30.0)
}

/// Wait for the team-up button to become visible.
///
/// Returns `true` if the button appeared, `false` on timeout.
pub fn wait_for_team_up(client: &Client, timeout_secs: f64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs_f64(timeout_secs);

    while start.elapsed() < timeout {
        if utils::is_visible_by_path(client, paths::TEAM_UP_BUTTON) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    false
}

/// Wait for combat to finish.
///
/// # Python equivalent
/// ```python
/// async def wait_for_combat_finish(self, await_combat=True, should_collect_wisps=True):
///     if await_combat:
///         while not await self.client.in_battle():
///             await asyncio.sleep(0.1)
///     while await self.client.in_battle():
///         await asyncio.sleep(0.1)
/// ```
pub fn wait_for_combat_finish(client: &Client, await_combat: bool, timeout_secs: f64) -> bool {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs_f64(timeout_secs);

    // Wait for combat to start
    if await_combat {
        while !client.in_battle() {
            if start.elapsed() > timeout {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    // Wait for combat to end
    while client.in_battle() {
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    true
}
