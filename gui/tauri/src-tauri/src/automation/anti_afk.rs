//! Anti-AFK — prevents the game from disconnecting due to inactivity.
//!
//! Periodically sends a tiny input to keep the game session alive.
//! Ported from Deimos anti-AFK behavior (built into the main loop).

use wizwalker::client::Client;
use wizwalker::constants::Keycode;

/// Send a minimal anti-AFK input to the client.
///
/// This rotates the camera slightly to prevent the AFK detection.
/// Call this periodically (e.g., every 10 minutes) from the telemetry loop.
///
/// # Python equivalent (Deimos main loop behavior)
/// The Python version typically rotates the camera or sends a tiny movement.
pub fn send_anti_afk(client: &Client) {
    // Send a small camera rotation — this doesn't move the player
    // but keeps the game session alive.
    client.send_key(Keycode::RightArrow);
    std::thread::sleep(std::time::Duration::from_millis(50));
}
