//! Combat utility helpers — stat formatting, mastery detection, school name conversion.
//!
//! Faithfully ported from `deimos-reference/src/combat_utils.py`.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use super::combat_objects::{SCHOOL_IDS, SCHOOL_NAMES, school_to_str};

// ── Stat manipulation ───────────────────────────────────────────────

/// Add a universal stat to every school-specific stat.
///
/// Python: `add_universal_stat(input_stats, uni_stat)` — combat_utils.py:31
pub fn add_universal_stat(input_stats: &[f32], uni_stat: f32) -> Vec<f32> {
    input_stats.iter().map(|s| s + uni_stat).collect()
}

/// Convert a list of stats into percentage values (×100).
///
/// Python: `to_percent(input_stats)` — combat_utils.py:50
pub fn to_percent(input_stats: &[f32]) -> Vec<f32> {
    input_stats.iter().map(|s| s * 100.0).collect()
}

/// Convert a list of stats into readable percentage strings.
///
/// Python: `to_percent_str(input_stats)` — combat_utils.py:41
pub fn to_percent_str(input_stats: &[f32]) -> Vec<String> {
    input_stats
        .iter()
        .map(|s| format!("{}%", s * 100.0))
        .collect()
}

/// Separate stats into (positives, negatives) hashmaps keyed by school name.
///
/// Python: `to_seperated_str_stats(input_stats)` — combat_utils.py:79
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
///
/// Python: `to_relevant_str_stats(input_stats, blacklist)` — combat_utils.py:69
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

// ── Enemy type detection ────────────────────────────────────────────

/// Determine enemy type string from combat member flags.
///
/// Python: `enemy_type_str(member)` — combat_utils.py:120
pub fn enemy_type_str(is_boss: bool, is_minion: bool, is_monster: bool) -> &'static str {
    if is_boss {
        "Boss"
    } else if is_minion {
        "Minion"
    } else if is_monster {
        "Mob"
    } else {
        "Player"
    }
}

// ── Content parsing ─────────────────────────────────────────────────

/// Extract text content from a window string (strips HTML-like tags).
///
/// Python: `content_from_str(input_str, seperator)` — combat_utils.py:134
pub fn content_from_str(input_str: &str, separator: &str) -> String {
    // Find all text between > and <
    let mut results = Vec::new();
    let mut in_content = false;
    let mut current = String::new();

    for ch in input_str.chars() {
        if ch == '>' {
            in_content = true;
            current.clear();
        } else if ch == '<' {
            if in_content && !current.is_empty() {
                results.push(current.clone());
            }
            in_content = false;
        } else if in_content {
            current.push(ch);
        }
    }

    results.join(separator)
}

/// Convert a stats hashmap to a readable string.
///
/// Python: `dict_to_str(input_dict, ...)` — stat_viewer.py:164
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

/// Default blacklist for stat display.
pub const STAT_DISPLAY_BLACKLIST: &[&str] =
    &["WhirlyBurly", "Gardening", "CastleMagic", "Cantrips", "Fishing"];
