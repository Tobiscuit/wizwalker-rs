//! Combat effect simulator — Simulates spell effects (damage, healing, charms).
//!
//! Faithfully ported from `deimos-reference/src/effect_simulation.py`.
//! NOTE: This module is unfinished in the original Python and has known bugs.
#![allow(dead_code, unused_imports, non_snake_case, unused_mut, unused_variables)]

use wizwalker::memory::objects::enums::{SpellEffects, MagicSchool, HangingDisposition};
use serde_json::{Value, json};
use crate::automation::combat_cache::{Cache, cache_get, cache_get_multi, cache_modify, cache_remove, filter_caches};
use crate::automation::combat_math::curve_stat;
use crate::automation::combat_objects::{MagicSchoolIndex, opposite_school_id, UNIVERSAL_SCHOOL_ID};
use std::collections::{HashSet, HashMap};

pub const MAIN_SCHOOLS: &[&str] = &["balance", "death", "life", "myth", "storm", "fire", "ice"];
pub const HANGING_EFFECT_PREFIXES: &[&str] = &["hanging", "public_hanging", "aura", "shadow_spell", "death_activated", "delay_cast"];

lazy_static::lazy_static! {
    pub static ref HANGING_EFFECT_PATHS: Vec<String> = HANGING_EFFECT_PREFIXES.iter().map(|p| format!("get_participant.{}_effects", p)).collect();
    
    pub static ref CHARM_EFFECT_TYPES: HashSet<SpellEffects> = HashSet::from([
        SpellEffects::ModifyOutgoingDamage,
        SpellEffects::ModifyOutgoingDamageFlat,
        SpellEffects::ModifyOutgoingHeal,
        SpellEffects::ModifyOutgoingHealFlat,
        SpellEffects::ModifyOutgoingDamageType,
        SpellEffects::ModifyOutgoingArmorPiercing,
        SpellEffects::ModifyAccuracy,
        SpellEffects::Dispel,
    ]);

    pub static ref WARD_EFFECT_TYPES: HashSet<SpellEffects> = HashSet::from([
        SpellEffects::ModifyIncomingDamage,
        SpellEffects::ModifyIncomingDamageFlat,
        SpellEffects::MaximumIncomingDamage,
        SpellEffects::ModifyIncomingHeal,
        SpellEffects::ModifyIncomingHealFlat,
        SpellEffects::ModifyIncomingDamageType,
        SpellEffects::ModifyIncomingArmorPiercing,
        SpellEffects::AbsorbDamage,
        SpellEffects::AbsorbHeal,
        SpellEffects::BounceNext,
        SpellEffects::BouncePrevious,
        SpellEffects::BounceBack,
        SpellEffects::BounceAll,
    ]);

    pub static ref DOT_EFFECT_TYPES: HashSet<SpellEffects> = HashSet::from([
        SpellEffects::DamageOverTime,
        SpellEffects::DeferredDamage,
    ]);

    pub static ref HOT_EFFECT_TYPES: HashSet<SpellEffects> = HashSet::from([
        SpellEffects::HealOverTime,
    ]);
}

pub fn clamp<T: PartialOrd>(num: T, min_val: T, max_val: T) -> T {
    if num < min_val { min_val }
    else if num > max_val { max_val }
    else { num }
}

/// Python: `collapse_effect` — effect_simulation.py:104
pub fn collapse_effect(subeffects: &[Value], type_name: &str, caster: &Value, target: &Value) -> Option<Value> {
    let mut effect: Option<Value> = None;

    match type_name {
        "RandomSpellEffect" | "RandomPerTargetSpellEffect" => {
            // Unless we want to create new permutations for every random spell effect, we should just choose a random effect.
            // BUG: Rust implementation just takes the first one since we don't have a random generator passed in, 
            // but the Python used random.choice(subeffects)
            effect = subeffects.first().cloned();
        }
        "VariableSpellEffect" => {
            // Handles per pip spells, like Tempest.
            let pip_values: Vec<i64> = subeffects.iter()
                .filter_map(|subeffect| subeffect["pip_num"].as_i64())
                .collect();
            if !pip_values.is_empty() {
                let min_pips = *pip_values.iter().min().unwrap();
                let max_pips = *pip_values.iter().max().unwrap();
                
                // BUG: (from Python original) 'pips' is used but not defined in the original Python function scope.
                let pips = 0; 
                let clamped_pips = clamp(pips, min_pips, max_pips);
                
                if let Some(idx) = pip_values.iter().position(|&p| p == clamped_pips) {
                    effect = subeffects.get(idx).cloned();
                }
            }
        }
        "HangingConversionSpellEffect" => {
            // TODO: Roshambo shit here
        }
        "ConditionalSpellEffect" => {
            // TODO: Conditional shit
        }
        "EffectListSpellEffect" => {
            // TODO: Figure out what this actually means
        }
        _ => {}
    }

    effect
}

/// Python: `sanitize_effect_list` — effect_simulation.py:135
pub fn sanitize_effect_list(effects: Vec<Value>) -> Vec<Value> {
    let mut result_effects = Vec::new();
    for effect in effects {
        if let Some(effect_type) = effect["effect_type"].as_i64() {
            if effect_type == SpellEffects::InvalidSpellEffect as i64 {
                if effect["maybe_effect_list"].is_null() {
                    continue;
                }
            }
        }
        result_effects.push(effect);
    }
    result_effects
}

/// Python: `remove_used_effects` — effect_simulation.py:151
pub fn remove_used_effects(cache: &mut Value, effect_list_index: usize, used_effect_indexes: &[usize]) {
    let mut index_offset = 0;
    for &i in used_effect_indexes {
        let path = format!("{}.{}", HANGING_EFFECT_PATHS[effect_list_index], i - index_offset);
        cache_remove(cache, &path);
        index_offset += 1;
    }
}

/// Python: `sim_outgoing_dmg_effects` — effect_simulation.py:159
pub fn sim_outgoing_dmg_effects(cache: &Value, mut damage_type: u32, mut damage: f64, mut pierce: f64) -> (Value, u32, f64, f64) {
    let mut result_cache = cache.clone();
    let hanging_paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s: &String| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &hanging_paths);

    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        let mut used_indexes = Vec::new();
        let mut used_ids = HashSet::new();
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_template_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let enchantment_spell_template_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_template_id, enchantment_spell_template_id);

                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != damage_type) || used_ids.contains(&ids) {
                    continue;
                }

                damage = clamp(damage, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);

                if effect_type == SpellEffects::ModifyOutgoingDamage as i64 {
                    damage *= (param / 100.0) + 1.0;
                } else if effect_type == SpellEffects::ModifyOutgoingDamageFlat as i64 {
                    damage += param;
                } else if effect_type == SpellEffects::ModifyOutgoingArmorPiercing as i64 {
                    pierce += param / 100.0;
                    pierce = (pierce * 100.0).round() / 100.0;
                } else if effect_type == SpellEffects::ModifyOutgoingDamageType as i64 {
                    damage_type = param as u32;
                } else {
                    continue;
                }

                used_indexes.push(m_i);
                used_ids.insert(ids);
            }
        }

        if i > 1 {
            continue;
        }

        remove_used_effects(&mut result_cache, i, &used_indexes);
    }

    (result_cache, damage_type, damage, pierce)
}

/// Python: `sim_outgoing_heal_effects` — effect_simulation.py:211
pub fn sim_outgoing_heal_effects(cache: &Value, mut heal_type: u32, mut heal: f64) -> (Value, f64) {
    let mut result_cache = cache.clone();
    let hanging_paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s: &String| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &hanging_paths);

    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        let mut used_indexes = Vec::new();
        let mut used_ids = HashSet::new();
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_template_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let enchantment_spell_template_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_template_id, enchantment_spell_template_id);

                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != heal_type) || used_ids.contains(&ids) {
                    continue;
                }

                heal = clamp(heal, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);

                if effect_type == SpellEffects::ModifyOutgoingHeal as i64 {
                    heal *= (param / 100.0) + 1.0;
                } else if effect_type == SpellEffects::ModifyOutgoingHealFlat as i64 {
                    heal += param;
                } else {
                    continue;
                }

                used_indexes.push(m_i);
                used_ids.insert(ids);
            }
        }

        if i > 1 {
            continue;
        }

        remove_used_effects(&mut result_cache, i, &used_indexes);
    }
    (result_cache, heal)
}

/// Python: `sim_incoming_dmg_effects` — effect_simulation.py:252
pub fn sim_incoming_dmg_effects(cache: &Value, mut damage_type: u32, mut damage: f64, mut pierce: f64) -> (Value, u32, f64, f64) {
    let mut result_cache = cache.clone();
    let hanging_paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s: &String| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &hanging_paths);

    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        let mut used_indexes = Vec::new();
        let mut used_ids = HashSet::new();
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_template_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let enchantment_spell_template_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_template_id, enchantment_spell_template_id);

                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != damage_type) || used_ids.contains(&ids) {
                    continue;
                }

                damage = clamp(damage, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);
                let max_pierce = pierce;

                if effect_type == SpellEffects::ModifyIncomingDamage as i64 {
                    let mut p = param;
                    if p < 0.0 {
                        p += pierce;
                        pierce += param;
                        pierce = (pierce * 100.0).round() / 100.0;
                        pierce = clamp(pierce, 0.0, max_pierce);
                        p = clamp(p, param, 0.0);
                    }
                    damage *= (p / 100.0) + 1.0;
                } else if effect_type == SpellEffects::ModifyIncomingDamageFlat as i64 {
                    damage += param;
                } else if effect_type == SpellEffects::AbsorbDamage as i64 {
                    if damage < param {
                        let path = format!("{}.{}.effect_param", HANGING_EFFECT_PATHS[i], m_i);
                        cache_modify(&mut result_cache, json!(param - damage), &path);
                        damage = 0.0;
                        continue;
                    }
                    damage += param;
                } else if effect_type == SpellEffects::ModifyIncomingArmorPiercing as i64 {
                    pierce += param / 100.0;
                    pierce = (pierce * 100.0).round() / 100.0;
                } else if effect_type == SpellEffects::ModifyIncomingDamageType as i64 {
                    damage_type = param as u32;
                } else {
                    continue;
                }

                used_indexes.push(m_i);
                used_ids.insert(ids);
            }
        }

        if i > 1 {
            continue;
        }

        remove_used_effects(&mut result_cache, i, &used_indexes);
    }

    (result_cache, damage_type, damage, pierce)
}

/// Python: `sim_incoming_heal_effects` — effect_simulation.py:314
pub fn sim_incoming_heal_effects(cache: &Value, mut heal_type: u32, mut heal: f64) -> (Value, f64) {
    let mut result_cache = cache.clone();
    let hanging_paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s: &String| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &hanging_paths);

    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        let mut used_indexes = Vec::new();
        let mut used_ids = HashSet::new();
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_template_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let enchantment_spell_template_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_template_id, enchantment_spell_template_id);

                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != heal_type) || used_ids.contains(&ids) {
                    continue;
                }

                heal = clamp(heal, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);

                if effect_type == SpellEffects::ModifyIncomingHeal as i64 {
                    heal *= (param / 100.0) + 1.0;
                } else if effect_type == SpellEffects::ModifyIncomingHealFlat as i64 {
                    heal += param;
                } else if effect_type == SpellEffects::AbsorbHeal as i64 {
                    if heal < param {
                        let path = format!("{}.{}.effect_param", HANGING_EFFECT_PATHS[i], m_i);
                        cache_modify(&mut result_cache, json!(param - heal), &path);
                        heal = 0.0;
                        continue;
                    }
                    heal += param;
                } else {
                    continue;
                }

                used_indexes.push(m_i);
                used_ids.insert(ids);
            }
        }

        if i > 1 {
            continue;
        }

        remove_used_effects(&mut result_cache, i, &used_indexes);
    }

    (result_cache, heal)
}

/// Python: `calc_crit` — effect_simulation.py:361
pub fn calc_crit(crit_rating: f64, block_rating: f64, caster_level: i32, target_level: i32, is_pvp: bool) -> (f64, f64, f64) {
    if is_pvp {
        let m_b = 5.0 * block_rating;
        let crit_multiplier = 2.0 - (m_b / (crit_rating + m_b));
        let m_c = 12.0 * crit_rating;
        let crit_chance = (caster_level as f64 / 185.0) * (m_c / (m_c + block_rating));
        let block_chance = (target_level as f64 / 185.0) * (block_rating / (block_rating + m_c));
        (crit_multiplier, crit_chance, block_chance)
    } else {
        let m_b = 3.0 * block_rating;
        let crit_multiplier = 2.0 - (m_b / (crit_rating + m_b));
        let m_c = 3.0 * crit_rating;
        let mut crit_chance = m_c / (m_c + block_rating);
        crit_chance = clamp(crit_chance, 0.0, 0.95);
        let block_chance = 0.4 * (block_rating / (block_rating + m_c));
        (crit_multiplier, crit_chance, block_chance)
    }
}

/// Python: `sim_damage` — effect_simulation.py:380
pub fn sim_damage(duel: &Value, caster: &Value, target: &Value, effect: &Value, crit_threshold: f64) -> (Value, Value, f64) {
    let mut target_result = target.clone();
    let mut caster_result = caster.clone();

    let damage_type_val = effect["damage_type"].as_u64().unwrap_or(UNIVERSAL_SCHOOL_ID as u64);
    let mut damage_type = damage_type_val as u32;
    let mut damage_type_index = MagicSchoolIndex::from_id(damage_type).0;

    let mut damage = effect["effect_param"].as_f64().unwrap_or(0.0);
    
    let dmg_percent_stat_base = cache_get(caster, "get_stats.dmg_bonus_percent")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let dmg_percent_stat_all = cache_get(caster, "get_stats.dmg_bonus_percent_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let mut dmg_percent_stat = (dmg_percent_stat_base + dmg_percent_stat_all * 100.0).round() / 100.0;
    
    if caster["is_player"].as_bool().unwrap_or(false) {
        let limit = duel["damage_limit"].as_f64().unwrap_or(0.0) as f32;
        let k0 = duel["d_k0"].as_f64().unwrap_or(0.0) as f32;
        let n0 = duel["d_n0"].as_f64().unwrap_or(0.0) as f32;
        dmg_percent_stat = curve_stat(dmg_percent_stat as f32, limit, k0, n0) as f64;
    }
    
    damage *= 1.0 + dmg_percent_stat;
    
    let dmg_flat_base = cache_get(caster, "get_stats.dmg_bonus_flat")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let dmg_flat_all = cache_get(caster, "get_stats.dmg_bonus_flat_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    damage += dmg_flat_base + dmg_flat_all;

    let resist_base = cache_get(target, "get_stats.dmg_reduce_percent")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let resist_all = cache_get(target, "get_stats.dmg_reduce_percent_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let mut resist = resist_base + resist_all;
    
    if target["is_player"].as_bool().unwrap_or(false) {
        let limit = duel["resist_limit"].as_f64().unwrap_or(0.0) as f32;
        let k0 = duel["r_k0"].as_f64().unwrap_or(0.0) as f32;
        let n0 = duel["r_n0"].as_f64().unwrap_or(0.0) as f32;
        resist = curve_stat(resist as f32, limit, k0, n0) as f64;
    }
    resist = (resist * 100.0).round() / 100.0;

    let pierce_base = cache_get(caster, "get_stats.ap_bonus_percent")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let pierce_all = cache_get(caster, "get_stats.ap_bonus_percent_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let mut pierce = (pierce_base + pierce_all * 100.0).round() / 100.0;

    let caster_level = caster["level"].as_i64().unwrap_or(0) as i32;
    let caster_crit_base = cache_get(caster, "get_stats.critical_hit_rating_by_school")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let caster_crit_all = cache_get(caster, "get_stats.critical_hit_rating_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let caster_crit = caster_crit_base + caster_crit_all;

    let target_level = target["level"].as_i64().unwrap_or(0) as i32;
    let target_block_base = cache_get(target, "get_stat.block_rating_by_school")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let target_block_all = cache_get(target, "get_stats.block_rating_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let target_block = target_block_base + target_block_all;

    let is_pvp = duel["pvp"].as_bool().unwrap_or(false) || duel["raid"].as_bool().unwrap_or(false);
    let (crit_damage_multiplier, crit_chance, block_chance) = calc_crit(caster_crit, target_block, caster_level, target_level, is_pvp);

    let effect_type = effect["effect_type"].as_i64().unwrap_or(0);
    if effect_type == SpellEffects::DamageNoCrit as i64 {
        // pass
    } else if crit_chance >= crit_threshold * (1.0 - block_chance) {
        damage *= crit_damage_multiplier;
    }

    let (c_res, d_type, dmg, prc) = sim_outgoing_dmg_effects(&caster_result, damage_type, damage, pierce);
    caster_result = c_res;
    damage_type = d_type;
    damage = dmg;
    pierce = prc;

    if caster["owner_id"] == target["owner_id"] {
        let (c_res, d_type, dmg, prc) = sim_incoming_dmg_effects(&caster_result, damage_type, damage, pierce);
        caster_result = c_res;
        damage_type = d_type;
        damage = dmg;
        pierce = prc;
    } else {
        let (t_res, d_type, dmg, prc) = sim_incoming_dmg_effects(&target_result, damage_type, damage, pierce);
        target_result = t_res;
        damage_type = d_type;
        damage = dmg;
        pierce = prc;
    }

    damage_type_index = MagicSchoolIndex::from_id(damage_type).0;

    let target_ref = if caster["owner_id"] == target["owner_id"] { &caster_result } else { &target_result };
    let flat_resist_base = cache_get(target_ref, "get_stats.dmg_reduce_flat")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let flat_resist_all = cache_get(target_ref, "get_stats.dmg_reduce_flat_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    damage -= flat_resist_base + flat_resist_all;
    damage = clamp(damage, 0.0, 2000000.0);

    let mut resist_mult = if resist > 0.0 {
        let r = resist - pierce;
        if r <= 0.0 { 1.0 } else { 1.0 - r }
    } else {
        resist.abs() + 1.0
    };

    damage *= resist_mult;

    let target_health = target_result["health"].as_f64().unwrap_or(0.0);
    target_result["health"] = json!(clamp(target_health - damage, 0.0, target_health));

    (caster_result, target_result, damage)
}

/// Python: `sim_incoming_damage` — effect_simulation.py:465
pub fn sim_incoming_damage(duel: &Value, target: &Value, damage_type: u32, mut damage: f64) -> (Value, f64) {
    let mut target_result = target.clone();
    let damage_type_index = MagicSchoolIndex::from_id(damage_type).0;

    let resist_base = cache_get(target, "get_stats.dmg_reduce_percent")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let resist_all = cache_get(target, "get_stats.dmg_reduce_percent_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let mut resist = resist_base + resist_all;
    if target["is_player"].as_bool().unwrap_or(false) {
        let limit = duel["resist_limit"].as_f64().unwrap_or(0.0) as f32;
        let k0 = duel["r_k0"].as_f64().unwrap_or(0.0) as f32;
        let n0 = duel["r_n0"].as_f64().unwrap_or(0.0) as f32;
        resist = curve_stat(resist as f32, limit, k0, n0) as f64;
    }
    resist = (resist * 100.0).round() / 100.0;

    let (t_res, _, dmg, pierce) = sim_incoming_dmg_effects(&target_result, damage_type, damage, 0.0);
    target_result = t_res;
    damage = dmg;

    let flat_resist_base = cache_get(&target_result, "get_stats.dmg_reduce_flat")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(damage_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let flat_resist_all = cache_get(&target_result, "get_stats.dmg_reduce_flat_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    damage -= flat_resist_base + flat_resist_all;
    damage = clamp(damage, 0.0, 2000000.0);

    let resist_mult = if resist > 0.0 {
        let r = resist - pierce;
        if r <= 0.0 { 1.0 } else { 1.0 - r }
    } else {
        resist.abs() + 1.0
    };

    damage *= resist_mult;

    let target_health = target_result["health"].as_f64().unwrap_or(0.0);
    target_result["health"] = json!(clamp(target_health - damage, 0.0, target_health));

    (target_result, damage)
}

/// Python: `get_multi_effects` — effect_simulation.py:503
pub fn get_multi_effects(effects: &[Value], valid_types: &HashSet<SpellEffects>, disposition: HangingDisposition) -> (Vec<Value>, Vec<usize>) {
    let mut matches = Vec::new();
    let mut match_indices = Vec::new();

    for (i, effect) in effects.iter().enumerate() {
        let effect_type_val = effect["effect_type"].as_i64().unwrap_or(-1);
        
        let mut is_valid = false;
        for vt in valid_types {
            if *vt as i64 == effect_type_val {
                is_valid = true;
                break;
            }
        }
        
        // BUG: (from Python original) Python used 'if effect["effect_type"] not in matches: continue' 
        // which makes it return empty lists initially.
        let mut found_in_matches = false;
        for m in &matches {
            if effect == m {
                found_in_matches = true;
                break;
            }
        }
        if !found_in_matches {
            continue;
        }

        let param = effect["effect_param"].as_f64().unwrap_or(0.0);
        match disposition {
            HangingDisposition::Beneficial => {
                if param < 0.0 { continue; }
            }
            HangingDisposition::Harmful => {
                if param >= 0.0 { continue; }
            }
            _ => {}
        }

        matches.push(effect.clone());
        match_indices.push(i);
    }

    (matches, match_indices)
}

/// Python: `sim_heal` — effect_simulation.py:530
pub fn sim_heal(duel: &Value, caster: &Value, target: &Value, effect: &Value, crit_threshold: f64) -> (Value, Value, f64) {
    let mut caster_result = caster.clone();
    let mut target_result = target.clone();

    let heal_type = effect["damage_type"].as_u64().unwrap_or(UNIVERSAL_SCHOOL_ID as u64) as u32;
    let heal_type_index = MagicSchoolIndex::from_id(heal_type).0;

    let mut heal = effect["effect_param"].as_f64().unwrap_or(0.0);
    let heal_percent_base = cache_get(caster, "get_participant.get_stats.heal_bonus_percent")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(heal_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let heal_percent_all = cache_get(caster, "get_participant.get_stats.heal_bonus_percent_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    heal *= 1.0 + (heal_percent_base + heal_percent_all);

    let caster_level = caster["level"].as_i64().unwrap_or(0) as i32;
    let caster_crit_base = cache_get(caster, "get_stats.critical_hit_rating_by_school")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(heal_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let caster_crit_all = cache_get(caster, "get_stats.critical_hit_rating_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let caster_crit = caster_crit_base + caster_crit_all;

    let target_level = target["level"].as_i64().unwrap_or(0) as i32;
    let target_block_base = cache_get(target, "get_stat.block_rating_by_school")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(heal_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let target_block_all = cache_get(target, "get_stats.block_rating_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let target_block = target_block_base + target_block_all;

    let is_pvp = duel["pvp"].as_bool().unwrap_or(false) || duel["raid"].as_bool().unwrap_or(false);
    let (crit_heal_multiplier, crit_chance, _) = calc_crit(caster_crit, target_block, caster_level, target_level, is_pvp);

    if crit_chance >= crit_threshold {
        heal *= crit_heal_multiplier;
    }

    // BUG: (from Python original) Python used 'damage' instead of 'heal' in 'sim_outgoing_heal_effects(caster_result, heal_type, damage)'
    let (c_res, h) = sim_outgoing_heal_effects(&caster_result, heal_type, heal);
    caster_result = c_res;
    heal = h;

    let heal_inc_percent_base = cache_get(caster, "get_participant.get_stats.heal_inc_bonus_percent")
        .and_then(|v: &Value| v.as_array())
        .and_then(|a: &Vec<Value>| a.get(heal_type_index))
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    let heal_inc_percent_all = cache_get(caster, "get_participant.get_stats.heal_inc_bonus_percent_all")
        .and_then(|v: &Value| v.as_f64())
        .unwrap_or(0.0);
    heal *= 1.0 + (heal_inc_percent_base + heal_inc_percent_all);

    if caster["owner_id"] == target["owner_id"] {
        let (c_res, h) = sim_incoming_heal_effects(&caster_result, heal_type, heal);
        caster_result = c_res;
        heal = h;
        let health = caster_result["health"].as_f64().unwrap_or(0.0);
        let max_health = caster_result["max_health"].as_f64().unwrap_or(0.0);
        caster_result["health"] = json!(clamp(health + heal, 0.0, max_health));
    } else {
        let (t_res, h) = sim_incoming_heal_effects(&target_result, heal_type, heal);
        target_result = t_res;
        heal = h;
        let health = target_result["health"].as_f64().unwrap_or(0.0);
        let max_health = target_result["max_health"].as_f64().unwrap_or(0.0);
        target_result["health"] = json!(clamp(health + heal, 0.0, max_health));
    }

    (caster_result, target_result, heal)
}

/// Python: `sim_effect` — effect_simulation.py:577
pub fn sim_effect(duel: &Value, caster: &Value, target: &Value, effect: &Value) -> Value {
    let mut target_result = target.clone();
    let mut caster_result = caster.clone();

    fn _transfer_hanging_effects(origin: &mut Vec<Value>, recipient: &mut Vec<Value>, amount: usize, valid_types: &HashSet<SpellEffects>, disposition: HangingDisposition) {
        let (_, mut effect_indices) = get_multi_effects(origin, valid_types, disposition);
        for i in 0..amount {
            if effect_indices.len() < amount - i {
                break;
            }
            let effect_index = effect_indices.remove(0);
            let effect = origin.remove(effect_index - i);
            recipient.insert(0, effect);
        }
    }

    fn _pop_hanging_effects(origin: &mut Vec<Value>, amount: usize, valid_types: &HashSet<SpellEffects>, disposition: HangingDisposition) -> Vec<Value> {
        let (_, mut effect_indices) = get_multi_effects(origin, valid_types, disposition);
        let mut effects = Vec::new();
        for i in 0..amount {
            // BUG: (from Python original) Python used 'if len(effect_indices) < amount < i: break'
            if effect_indices.len() < amount && amount < i {
                break;
            }
            if effect_indices.is_empty() { break; }
            let effect_index = effect_indices.remove(0);
            let effect = origin.remove(effect_index - i);
            effects.push(effect);
        }
        effects
    }

    let caster_pips_cache = cache_get(&caster_result, "get_participant.pip_count").cloned().unwrap_or(json!({}));
    let mut caster_total_spips = 0;
    for s in MAIN_SCHOOLS {
        caster_total_spips += caster_pips_cache[format!("{}_pips", s)].as_i64().unwrap_or(0);
    }
    let mut caster_effects: Vec<Vec<Value>> = Vec::new();
    for p in HANGING_EFFECT_PATHS.iter() {
        let raw = cache_get(&caster_result, p).cloned().unwrap_or(json!([]));
        let effects = sanitize_effect_list(raw.as_array().cloned().unwrap_or_default());
        caster_effects.push(effects);
    }

    let target_pips_cache = cache_get(&target_result, "get_participant.pip_count").cloned().unwrap_or(json!({}));
    let mut target_total_spips = 0;
    for s in MAIN_SCHOOLS {
        target_total_spips += target_pips_cache[format!("{}_pips", s)].as_i64().unwrap_or(0);
    }
    let mut target_effects: Vec<Vec<Value>> = Vec::new();
    for p in HANGING_EFFECT_PATHS.iter() {
        let raw = cache_get(&target_result, p).cloned().unwrap_or(json!([]));
        let effects = sanitize_effect_list(raw.as_array().cloned().unwrap_or_default());
        target_effects.push(effects);
    }

    let effect_type = effect["effect_type"].as_i64().unwrap_or(0);
    let disposition_val = effect["disposition"].as_i64().unwrap_or(0) as i32;
    let disposition = match disposition_val {
        1 => HangingDisposition::Beneficial,
        2 => HangingDisposition::Harmful,
        _ => HangingDisposition::Both,
    };
    let effect_param_f = effect["effect_param"].as_f64().unwrap_or(0.0);
    let effect_param = effect_param_f as usize;

    if effect_type == SpellEffects::Damage as i64 {
        let (cr, tr, _) = sim_damage(duel, &caster_result, &target_result, effect, 0.8);
        caster_result = cr; target_result = tr;
    } else if effect_type == SpellEffects::DamageNoCrit as i64 {
        let (cr, tr, _) = sim_damage(duel, &caster_result, &target_result, effect, 2.0);
        caster_result = cr; target_result = tr;
    } else if effect_type == SpellEffects::StealHealth as i64 {
        let (cr, tr, damage) = sim_damage(duel, &caster_result, &target_result, effect, 0.8);
        caster_result = cr; target_result = tr;
        let h = caster_result["health"].as_f64().unwrap_or(0.0);
        caster_result["health"] = json!(h + damage * effect["heal_modifier"].as_f64().unwrap_or(0.0));
    } else if effect_type == SpellEffects::DamagePerTotalPipPower as i64 {
        let t_power_pips = target_pips_cache["power_pips"].as_i64().unwrap_or(0);
        let t_generic_pips = target_pips_cache["generic_pips"].as_i64().unwrap_or(0);
        let target_total_pip_value = (target_total_spips * 2) + (t_power_pips * 2) + t_generic_pips;
        let mut mod_effect = effect.clone();
        mod_effect["effect_param"] = json!(effect_param_f * target_total_pip_value as f64);
        let (cr, tr, _) = sim_damage(duel, &caster_result, &target_result, &mod_effect, 0.8);
        caster_result = cr; target_result = tr;
    } else if effect_type == SpellEffects::Heal as i64 {
        let (cr, tr, _) = sim_heal(duel, &caster_result, &target_result, effect, 0.8);
        caster_result = cr; target_result = tr;
    } else if effect_type == SpellEffects::HealPercent as i64 {
        let mut mod_effect = effect.clone();
        let max_health = target_result["max_health"].as_f64().unwrap_or(0.0);
        mod_effect["effect_param"] = json!((effect_param_f / 100.0) * max_health);
        let (cr, tr, _) = sim_heal(duel, &caster_result, &target_result, &mod_effect, 0.8);
        caster_result = cr; target_result = tr;
    } else if effect_type == SpellEffects::SetHealPercent as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ReduceOverTime as i64 {
        let mut match_map = HashMap::new();
        match_map.insert("effect_type".to_string(), json!(SpellEffects::DamageOverTime as i32));
        let (target_dots, dot_indices) = filter_caches(&target_effects[0], &match_map, false, false);
        for (dot, i) in target_dots.into_iter().zip(dot_indices) {
            let mut mod_dot = dot.clone();
            if let Some(p) = mod_dot["effect_param"].as_f64() {
                mod_dot["effect_param"] = json!(p - effect_param_f);
            }
            target_effects[0][i] = mod_dot;
        }
    } else if effect_type == SpellEffects::DetonateOverTime as i64 {
        let dots = _pop_hanging_effects(&mut target_effects[0], effect_param, &DOT_EFFECT_TYPES, disposition);
        for dot in dots {
            let d_type = dot["damage_type"].as_u64().unwrap_or(0) as u32;
            let d_param = dot["effect_param"].as_f64().unwrap_or(0.0);
            let heal_mod = effect["heal_modifier"].as_f64().unwrap_or(0.0);
            let (tr, _) = sim_incoming_damage(duel, &target_result, d_type, d_param * heal_mod);
            target_result = tr;
        }
    } else if effect_type == SpellEffects::PushCharm as i64 {
        _transfer_hanging_effects(&mut caster_effects[0], &mut target_effects[0], effect_param, &CHARM_EFFECT_TYPES, disposition);
    } else if effect_type == SpellEffects::StealCharm as i64 {
        _transfer_hanging_effects(&mut target_effects[0], &mut caster_effects[0], effect_param, &CHARM_EFFECT_TYPES, disposition);
    } else if effect_type == SpellEffects::PushWard as i64 {
        _transfer_hanging_effects(&mut caster_effects[0], &mut target_effects[0], effect_param, &WARD_EFFECT_TYPES, disposition);
    } else if effect_type == SpellEffects::StealWard as i64 {
        _transfer_hanging_effects(&mut target_effects[0], &mut caster_effects[0], effect_param, &WARD_EFFECT_TYPES, disposition);
    } else if effect_type == SpellEffects::PushOverTime as i64 {
        _transfer_hanging_effects(&mut caster_effects[0], &mut target_effects[0], effect_param, &DOT_EFFECT_TYPES, disposition);
    } else if effect_type == SpellEffects::StealOverTime as i64 {
        _transfer_hanging_effects(&mut target_effects[0], &mut caster_effects[0], effect_param, &DOT_EFFECT_TYPES, disposition);
    } else if effect_type == SpellEffects::SwapAll as i64 {
        std::mem::swap(&mut caster_effects[0], &mut target_effects[0]);
    } else if effect_type == SpellEffects::SwapCharm as i64 {
        let mut c_charms = _pop_hanging_effects(&mut caster_effects[0], effect_param, &CHARM_EFFECT_TYPES, disposition);
        let mut t_charms = _pop_hanging_effects(&mut target_effects[0], effect_param, &CHARM_EFFECT_TYPES, disposition);
        t_charms.append(&mut caster_effects[0]);
        caster_effects[0] = t_charms;
        c_charms.append(&mut target_effects[0]);
        target_effects[0] = c_charms;
    } else if effect_type == SpellEffects::SwapWard as i64 {
        let mut c_wards = _pop_hanging_effects(&mut caster_effects[0], effect_param, &WARD_EFFECT_TYPES, disposition);
        let mut t_wards = _pop_hanging_effects(&mut target_effects[0], effect_param, &WARD_EFFECT_TYPES, disposition);
        t_wards.append(&mut caster_effects[0]);
        caster_effects[0] = t_wards;
        c_wards.append(&mut target_effects[0]);
        target_effects[0] = c_wards;
    } else if effect_type == SpellEffects::SwapOverTime as i64 {
        let mut c_dots = _pop_hanging_effects(&mut caster_effects[0], effect_param, &DOT_EFFECT_TYPES, disposition);
        let mut t_dots = _pop_hanging_effects(&mut target_effects[0], effect_param, &DOT_EFFECT_TYPES, disposition);
        t_dots.append(&mut caster_effects[0]);
        caster_effects[0] = t_dots;
        c_dots.append(&mut target_effects[0]);
        target_effects[0] = c_dots;
    } else if effect_type == SpellEffects::Clue as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::DelayCast as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyCardCloak as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyCardDamage as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyCardAccuracy as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyCardMutation as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyCardRank as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyCardArmorPiercing as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::SummonCreature as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::TeleportPlayer as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::Reshuffle as i64 {
        // pass (from Python)
    } else if effect_type == SpellEffects::ModifyPips as i64 {
        // TODO: Make a mod pips function (from Python)
    } else {
        target_effects[0].insert(0, effect.clone());
    }

    for (i, path) in HANGING_EFFECT_PATHS.iter().enumerate() {
        cache_modify(&mut target_result, json!(target_effects[i]), path);
        cache_modify(&mut caster_result, json!(caster_effects[i]), path);
    }

    target_result
}

pub const SPIP_ORDER: &[&str] = &[
    "balance_pips",
    "death_pips",
    "fire_pips",
    "ice_pips",
    "life_pips",
    "myth_pips",
    "storm_pips",
];

pub const PIP_ORDER: &[&str] = &[
    "generic_pips",
    "power_pips"
];

/// Python: `generate_pip_list` — effect_simulation.py:764
pub fn generate_pip_list(pip_cache: &Value) -> (Vec<String>, i64) {
    let mut pip_list = Vec::new();
    let mut all_pip_order = Vec::new();
    all_pip_order.extend_from_slice(PIP_ORDER);
    all_pip_order.extend_from_slice(SPIP_ORDER);

    for &pip_type in &all_pip_order {
        let pip_num = pip_cache[pip_type].as_i64().unwrap_or(0);
        if pip_num == 0 {
            continue;
        }

        for _ in 0..pip_num {
            pip_list.push(pip_type.to_string());
        }
    }

    (pip_list, pip_cache["shadow_pips"].as_i64().unwrap_or(0))
}

/// Python: `clamp_pips` — effect_simulation.py:782
pub fn clamp_pips(pip_cache: &Value, pip_type: &str) -> (Value, i64) {
    let mut pip_cache_result = pip_cache.clone();
    pip_cache_result[pip_type] = json!(0); 

    let (min_pip_list, shadow_pips) = generate_pip_list(&pip_cache_result);
    let clamped_shadow_pips = clamp(shadow_pips, 0, 2);
    pip_cache_result["shadow_pips"] = json!(clamped_shadow_pips); 

    let max_specific_pips = 7 - (min_pip_list.len() as i64);
    if max_specific_pips <= 0 {
        return (pip_cache_result, clamped_shadow_pips);
    }

    let original_val = pip_cache[pip_type].as_i64().unwrap_or(0);
    pip_cache_result[pip_type] = json!(clamp(original_val, 0, max_specific_pips));

    (pip_cache_result, clamped_shadow_pips)
}

/// Python: `sim_add_pips` — effect_simulation.py:806
pub fn sim_add_pips(pip_cache: &Value, pip_type: &str, param: i64) -> Value {
    let mut pip_cache_result = pip_cache.clone();
    let current_val = pip_cache_result[pip_type].as_i64().unwrap_or(0);
    pip_cache_result[pip_type] = json!(current_val + param);
    let (res, _) = clamp_pips(&pip_cache_result, pip_type);
    res
}

/// Python: `sim_remove_pips` — effect_simulation.py:814
pub fn sim_remove_pips(pip_cache: &Value, pip_type: &str, param: i64) -> Value {
    let mut pip_cache_result = pip_cache.clone();
    // TODO: THIS ENTIRE FUNCTION (from Python)
    pip_cache_result
}

/// Python: `sim_modify_pips` — effect_simulation.py:819
pub fn sim_modify_pips(pip_cache: &Value, pip_type: &str, param: i64, pip_school: u32) -> Value {
    let mut pip_cache_result = pip_cache.clone();

    if pip_type == "shadow_pips" {
        let current_val = pip_cache[pip_type].as_i64().unwrap_or(0);
        pip_cache_result[pip_type] = json!(clamp(current_val, 0, 2));
        return pip_cache_result;
    }

    // TODO: THIS ENTIRE FUNCTION (from Python)
    // def _sim_remove_pips(pip_cache: Cache, pip_type: str) (from Python)
    pip_cache_result
}
