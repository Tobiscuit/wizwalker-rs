#![allow(dead_code, unused_imports, non_snake_case, unused_variables)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

pub const DEFAULT_CONFIG: &str = "any<trap & inc_damage>[potent] @ enemy | any<trap & inc_damage & aoe>[potent] | any<blade & out_damage>[sharp] @ self | any<blade & out_damage & aoe>[sharp] | any<global> | any<aura & out_damage> | any<shadow> | any<damage & aoe>[any<mod_damage>] | any<damage>[any<mod_damage>] @ enemy";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CombatConfig { pub strategy: String }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TargetType { Self_, Enemy, Ally, All }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpellType { Damage, Blade, Trap, Global, Aura, Shadow, Heal, Utility }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateSpell { pub spell_id: u32, pub name: String, pub spell_type: SpellType }

pub struct StrCombatConfigProvider {
    pub filename: String,
    pub config: CombatConfig,
    pub cast_time: f32,
}

impl StrCombatConfigProvider {
    pub fn new(config_data: &str, cast_time: f32) -> Self {
        let mut slf = Self { filename: "Config".to_string(), config: CombatConfig::default(), cast_time };
        slf.config = slf.parse_config(config_data);
        slf
    }

    pub async fn handle_no_cards_given(&self) -> Result<(), String> {
        Err("Full config fail! Config might be empty or contains only explicit rounds. Consider adding a pass or something else.".to_string())
    }

    pub fn parse_config(&self, file_contents: &str) -> CombatConfig {
        let config = serde_yaml::from_str(file_contents).unwrap_or(CombatConfig { strategy: file_contents.to_string() });
        self._expand_config(config)
    }

    fn _expand_config(&self, config: CombatConfig) -> CombatConfig { config }
}

pub fn delegate_combat_configs(input_data: &str, fallback_clients: usize, ls: &str) -> HashMap<usize, String> {
    let (mut configs, mut local, mut client, pattern) = (HashMap::new(), Vec::new(), None, Regex::new(r"###\sp\s*(\d+)").unwrap());
    for line in input_data.split(ls) {
        if let Some(caps) = pattern.captures(line) {
            if let Some(idx) = client { configs.insert(idx, local.join(ls)); }
            client = Some(caps[1].parse::<usize>().unwrap() - 1);
            local.clear();
        } else { local.push(line); }
    }
    if let Some(idx) = client { configs.insert(idx, local.join(ls)); }
    else { (0..fallback_clients).for_each(|i| { configs.insert(i, input_data.to_string()); }); }
    configs
}
