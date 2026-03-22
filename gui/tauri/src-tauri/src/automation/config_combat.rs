//! Combat strategy configuration — YAML-based strategy loading.
//!
//! Faithfully ported from `deimos-reference/src/config_combat.rs`.
#![allow(dead_code, unused_imports, non_snake_case)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use crate::automation::combat_objects::UNIVERSAL_SCHOOL_ID;

/// Default combat strategy string.
///
/// Python: `default_config` — config_combat.py:16
pub const DEFAULT_CONFIG: &str = "any<trap & inc_damage>[potent] @ enemy | any<trap & inc_damage & aoe>[potent] | any<blade & out_damage>[sharp] @ self | any<blade & out_damage & aoe>[sharp] | any<global> | any<aura & out_damage> | any<shadow> | any<damage & aoe>[any<mod_damage>] | any<damage>[any<mod_damage>] @ enemy";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CombatConfig {
    pub strategy: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetType {
    Self_,
    Enemy,
    Ally,
    All,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpellType {
    Damage,
    Blade,
    Trap,
    Global,
    Aura,
    Shadow,
    Heal,
    Utility,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateSpell {
    pub spell_id: u32,
    pub name: String,
    pub spell_type: SpellType,
}

/// Handles turning a raw string of file content into a dict of client combat configs.
///
/// Python: `delegate_combat_configs(input_data, fallback_clients, line_separator)` — config_combat.py:46
pub fn delegate_combat_configs(
    input_data: &str,
    fallback_clients: usize,
    line_separator: &str,
) -> HashMap<usize, String> {
    let config_lines: Vec<&str> = input_data.split(line_separator).collect();
    let mut client_configs: HashMap<usize, String> = HashMap::new();

    // Match number in "### pX", where X is a number. This determines the client index.
    let re = Regex::new(r"###\s*p\s*(\d+)").unwrap();
    let mut client_to_match: Option<usize> = None;
    let mut local_configs: Vec<&str> = Vec::new();

    for line in config_lines.iter() {
        if let Some(caps) = re.captures(line) {
            if let Some(idx) = client_to_match {
                client_configs.insert(idx, local_configs.join(line_separator));
            }
            if let Ok(num) = caps[1].parse::<usize>() {
                client_to_match = Some(num - 1);
            }
            local_configs.clear();
            continue;
        }
        local_configs.push(line);
    }

    if let Some(idx) = client_to_match {
        client_configs.insert(idx, local_configs.join(line_separator));
    }

    // If no client was ever specified, just assign entire config to all clients
    if client_to_match.is_none() {
        let full_config = config_lines.join(line_separator);
        for i in 0..fallback_clients {
            client_configs.insert(i, full_config.clone());
        }
    }

    client_configs
}

pub struct StrCombatConfigProvider {
    pub filename: String,
    pub config: CombatConfig,
    pub cast_time: f32,
}

impl StrCombatConfigProvider {
    pub fn new(config_data: &str, cast_time: f32) -> Self {
        Self {
            filename: "Config".to_string(),
            config: Self::parse_config(config_data),
            cast_time,
        }
    }

    pub fn parse_config(config_data: &str) -> CombatConfig {
        CombatConfig {
            strategy: config_data.to_string(),
        }
    }

    pub async fn handle_no_cards_given(&self) -> Result<(), String> {
        Err("Full config fail! Config might be empty or contains only explicit rounds. Consider adding a pass or something else.".to_string())
    }
}
