//! Stat viewer — combat stat display and damage estimation.
//!
//! Faithfully ported from `deimos-reference/src/stat_viewer.py`.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use super::combat_objects::{school_to_str, school_list_index};
use super::combat_utils::{add_universal_stat, to_percent, to_separated_str_stats, dict_to_str, STAT_DISPLAY_BLACKLIST};

// ── Damage-per-pip constants ────────────────────────────────────────

/// Base damage per pip by school ID.
///
/// Python: `damage_per_pip` — stat_viewer.py:24
pub fn damage_per_pip(school_id: u32) -> u32 {
    match school_id {
        2343174    => 100,  // Fire
        72777      => 83,   // Ice
        83375795   => 125,  // Storm
        78318724   => 85,   // Death
        2330892    => 83,   // Life
        2448141    => 90,   // Myth
        1027491821 => 85,   // Balance
        _          => 100,
    }
}

/// Shadow damage per pip by school ID.
///
/// Python: `shadow_damage_per_pip` — stat_viewer.py:34
pub fn shadow_damage_per_pip(school_id: u32) -> u32 {
    match school_id {
        2343174    => 120,  // Fire
        72777      => 100,  // Ice
        83375795   => 130,  // Storm
        78318724   => 105,  // Death
        2330892    => 100,  // Life
        2448141    => 115,  // Myth
        1027491821 => 105,  // Balance
        _          => 100,
    }
}

// ── Stat display ────────────────────────────────────────────────────

/// Collected stats for a combat member, ready for display.
///
/// Python: `total_stats()` return value — stat_viewer.py:45
#[derive(Debug, Clone)]
pub struct MemberStats {
    pub name: String,
    pub member_type: String,
    pub school_name: String,
    pub power_pips: u32,
    pub normal_pips: u32,
    pub shadow_pips: u32,
    pub health: u32,
    pub max_health: u32,
    pub resistances: HashMap<String, f32>,
    pub boosts: HashMap<String, f32>,
    pub damages: HashMap<String, f32>,
    pub pierces: HashMap<String, f32>,
    pub crits: HashMap<String, f32>,
    pub blocks: HashMap<String, f32>,
    pub masteries: Vec<String>,
    pub estimated_damage: f32,
    pub target_name: String,
}

impl MemberStats {
    /// Format stats into display strings.
    ///
    /// Python: `total_stats` return format — stat_viewer.py:146
    pub fn to_display_lines(&self) -> Vec<String> {
        let health_pct = if self.max_health > 0 {
            (self.health as f32 / self.max_health as f32 * 100.0) as u32
        } else {
            0
        };

        let default_blacklist = STAT_DISPLAY_BLACKLIST;

        vec![
            format!(
                "Estimated Max Dmg Against {}: {}",
                self.target_name, self.estimated_damage as i32
            ),
            format!(
                "Name: {} - {} - {}",
                self.name, self.member_type, self.school_name
            ),
            format!("Power Pips: {} - Pips: {}", self.power_pips, self.normal_pips),
            format!("Shadow Pips: {}", self.shadow_pips),
            format!(
                "Health: {}/{} ({}%)",
                self.health, self.max_health, health_pct
            ),
            format!(
                "Boosts: {}",
                dict_to_str(&self.boosts, ": ", ", ", true, default_blacklist)
            ),
            format!(
                "Resists: {}",
                dict_to_str(&self.resistances, ": ", ", ", false, default_blacklist)
            ),
            format!(
                "Damages: {}",
                dict_to_str(&self.damages, ": ", ", ", false, default_blacklist)
            ),
            format!(
                "Pierces: {}",
                dict_to_str(&self.pierces, ": ", ", ", false, default_blacklist)
            ),
            format!(
                "Crits: {}",
                dict_to_str(&self.crits, ": ", ", ", false, default_blacklist)
            ),
            format!(
                "Blocks: {}",
                dict_to_str(&self.blocks, ": ", ", ", false, default_blacklist)
            ),
            format!("Masteries: {}", self.masteries.join(", ")),
        ]
    }
}

/// Convert total stats lines to a single GUI string.
///
/// Python: `to_gui_str(stats, separator)` — stat_viewer.py:178
pub fn to_gui_str(stats: &[String], separator: &str) -> String {
    stats.join(separator)
}
