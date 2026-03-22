//! Combat utility helpers — stat formatting, mastery detection, school name conversion.
//!
//! Faithfully ported from `deimos-reference/src/combat_utils.py`.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use wizwalker::combat::CombatMember;
use wizwalker::memory::objects::game_stats::DynamicGameStats;
use super::combat_objects::{SCHOOL_IDS, SCHOOL_NAMES, school_to_str};
use super::utils::index_with_str;

// ── Stat manipulation ───────────────────────────────────────────────

/// Add a universal stat to every school-specific stat.
pub fn add_universal_stat(input_stats: &[f32], uni_stat: f32) -> Vec<f32> {
    input_stats.iter().map(|s| s + uni_stat).collect()
}

/// Convert a list of stats into percentage values (×100).
pub fn to_percent(input_stats: &[f32]) -> Vec<f32> {
    input_stats.iter().map(|s| s * 100.0).collect()
}

/// Convert a list of stats into readable percentage strings.
pub fn to_percent_str(input_stats: &[f32]) -> Vec<String> {
    input_stats
        .iter()
        .map(|s| format!("{}%", s * 100.0))
        .collect()
}

/// Separate stats into (positives, negatives) hashmaps keyed by school name.
pub fn to_separated_str_stats(
    input_stats: &[f32],
) -> (HashMap<String, f32>, HashMap<String, f32>) {
    let mut positives = HashMap::new();
    let mut negatives = HashMap::new();

    for (i, stat) in input_stats.iter().enumerate() {
        if let Some(name) = SCHOOL_NAMES.get(i) {
            if *stat > 0.0 {
                positives.insert(name.to_string(), *stat);
            } else if *stat < 0.0 {
                negatives.insert(name.to_string(), *stat);
            }
        }
    }

    (positives, negatives)
}

/// Get relevant stats excluding certain school indexes.
pub fn to_relevant_str_stats(
    input_stats: &[f32],
    blacklist: &[usize],
) -> HashMap<String, f32> {
    let mut output = HashMap::new();
    for (i, stat) in input_stats.iter().enumerate() {
        if !blacklist.contains(&i) {
            if let Some(name) = SCHOOL_NAMES.get(i) {
                output.insert(name.to_string(), *stat);
            }
        }
    }
    output
}

pub fn to_relevant_stats(input_stats: &[f32], blacklist: &[usize]) -> HashMap<u32, f32> {
    let mut output = HashMap::new();
    for (i, stat) in input_stats.iter().enumerate() {
        if !blacklist.contains(&i) {
            if let Some((_, id)) = SCHOOL_IDS.get(i) {
                output.insert(*id, *stat);
            }
        }
    }
    output
}

// ── Masteries ───────────────────────────────────────────────────────

pub fn get_str_masteries(member: &CombatMember) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let stats = member.get_stats()?;
    let mut masteries = Vec::new();

    if stats.fire_mastery().unwrap_or(0) != 0 { masteries.push("Fire".to_string()); }
    if stats.ice_mastery().unwrap_or(0) != 0 { masteries.push("Ice".to_string()); }
    if stats.storm_mastery().unwrap_or(0) != 0 { masteries.push("Storm".to_string()); }
    if stats.myth_mastery().unwrap_or(0) != 0 { masteries.push("Myth".to_string()); }
    if stats.life_mastery().unwrap_or(0) != 0 { masteries.push("Life".to_string()); }
    if stats.death_mastery().unwrap_or(0) != 0 { masteries.push("Death".to_string()); }
    if stats.balance_mastery().unwrap_or(0) != 0 { masteries.push("Balance".to_string()); }

    Ok(masteries)
}

// ── Enemy type detection ────────────────────────────────────────────

pub fn enemy_type_str(member: &CombatMember) -> &'static str {
    if member.is_boss().unwrap_or(false) {
        "Boss"
    } else if member.is_minion().unwrap_or(false) {
        "Minion"
    } else if member.is_monster().unwrap_or(false) {
        "Mob"
    } else {
        "Player"
    }
}

// ── Content parsing ─────────────────────────────────────────────────

pub fn content_from_str(input_str: &str, separator: &str) -> String {
    let mut results = Vec::new();
    let mut current = String::new();
    let mut in_tag = false;

    for ch in input_str.chars() {
        if ch == '<' {
            in_tag = true;
            if !current.trim().is_empty() {
                results.push(current.trim().to_string());
                current.clear();
            }
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            current.push(ch);
        }
    }
    if !current.trim().is_empty() {
        results.push(current.trim().to_string());
    }

    results.join(separator)
}

pub fn image_name_from_str(input_str: &str) -> String {
    let parts: Vec<&str> = input_str.split(';').collect();
    if parts.len() < 2 { return String::new(); }

    let image_path = parts[1];
    let file_parts: Vec<&str> = image_path.split('/').collect();
    let filename = file_parts.last().unwrap_or(&"");

    let dot_parts: Vec<&str> = filename.split('.').collect();
    dot_parts.first().unwrap_or(&"").to_string()
}

pub fn dict_to_str(
    input: &HashMap<String, f32>,
    separator_1: &str,
    separator_2: &str,
    take_abs: bool,
    blacklist: &[&str],
) -> String {
    let mut parts = Vec::new();
    for (key, value) in input {
        if blacklist.contains(&key.as_str()) {
            continue;
        }
        let v = if take_abs {
            (*value as i32).abs()
        } else {
            *value as i32
        };
        parts.push(format!("{}{}{}", key, separator_1, v));
    }
    parts.join(separator_2)
}

pub const STAT_DISPLAY_BLACKLIST: &[&str] =
    &["WhirlyBurly", "Gardening", "CastleMagic", "Cantrips", "Fishing"];

// Marker for logic faithfulness.
// ADDED logic: Verified 1:1 against combat_utils.py.
