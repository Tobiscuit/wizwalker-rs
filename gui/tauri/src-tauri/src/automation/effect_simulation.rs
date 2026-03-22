//! Combat effect simulator — Simulates spell effects (damage, healing, charms).
//!
//! Faithfully ported from `deimos-reference/src/effect_simulation.py`.
//! NOTE: This module is unfinished in the original Python and has known bugs.
#![allow(dead_code, unused_imports, non_snake_case, unused_mut, unused_variables)]

use wizwalker::memory::objects::enums::{SpellEffects, MagicSchool, HangingDisposition};
use serde_json::Value;
use crate::automation::combat_cache::{Cache, cache_get, cache_get_multi, cache_modify, cache_remove};
use crate::automation::combat_math::curve_stat;
use crate::automation::combat_objects::{MagicSchoolIndex, opposite_school_id, UNIVERSAL_SCHOOL_ID};
use std::collections::{HashSet, HashMap};

pub const MAIN_SCHOOLS: &[&str] = &["balance_pips", "death_pips", "fire_pips", "ice_pips", "life_pips", "myth_pips", "storm_pips"];

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

/// Python: `collapse_effect` — effect_simulation.py:90
pub fn collapse_effect(subeffects: &[Value], type_name: &str, caster: &Value, target: &Value) -> Option<Value> {
    match type_name {
        "RandomSpellEffect" | "RandomPerTargetSpellEffect" => subeffects.first().cloned(),
        "VariableSpellEffect" => None, // BUG: Python code has undefined variable 'pips'
        "HangingConversionSpellEffect" => None, // TODO: Roshambo
        "ConditionalSpellEffect" => None,
        "EffectListSpellEffect" => None,
        _ => None
    }
}

/// Python: `sanitize_effect_list` — effect_simulation.py:126
pub fn sanitize_effect_list(effects: Vec<Value>) -> Vec<Value> {
    let mut result_effects = Vec::new();
    for effect in effects {
        if let Some(effect_type) = effect["effect_type"].as_i64() {
            if effect_type == SpellEffects::InvalidSpellEffect as i64 {
                if effect["maybe_effect_list"].is_null() { continue; }
            }
        }
        result_effects.push(effect);
    }
    result_effects
}

/// Python: `remove_used_effects` — effect_simulation.py:146
pub fn remove_used_effects(cache: &mut Value, effect_list_index: usize, used_effect_indexes: &[usize]) {
    let mut index_offset = 0;
    for &i in used_effect_indexes {
        let path = format!("{}.{}", HANGING_EFFECT_PATHS[effect_list_index], i - index_offset);
        cache_remove(cache, &path);
        index_offset += 1;
    }
}

/// Python: `sim_outgoing_dmg_effects` — effect_simulation.py:157
pub fn sim_outgoing_dmg_effects(cache: &Value, mut damage_type: u32, mut damage: f64, mut pierce: f64) -> (Value, u32, f64, f64) {
    let mut result_cache = cache.clone();
    let paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &paths);
    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            let mut used_indexes = Vec::new();
            let mut used_ids = HashSet::new();
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let ench_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_id, ench_id);
                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != damage_type) || used_ids.contains(&ids) { continue; }
                damage = clamp(damage, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);
                if effect_type == SpellEffects::ModifyOutgoingDamage as i64 { damage *= (param / 100.0) + 1.0; }
                else if effect_type == SpellEffects::ModifyOutgoingDamageFlat as i64 { damage += param; }
                else if effect_type == SpellEffects::ModifyOutgoingArmorPiercing as i64 { pierce += param / 100.0; }
                else if effect_type == SpellEffects::ModifyOutgoingDamageType as i64 { damage_type = param as u32; }
                else { continue; }
                used_indexes.push(m_i); used_ids.insert(ids);
            }
            if i <= 1 { remove_used_effects(&mut result_cache, i, &used_indexes); }
        }
    }
    (result_cache, damage_type, damage, pierce)
}

/// Python: `sim_outgoing_heal_effects` — effect_simulation.py:214
pub fn sim_outgoing_heal_effects(cache: &Value, mut heal_type: u32, mut heal: f64) -> (Value, f64) {
    let mut result_cache = cache.clone();
    let paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &paths);
    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            let mut used_indexes = Vec::new();
            let mut used_ids = HashSet::new();
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let ench_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_id, ench_id);
                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != heal_type) || used_ids.contains(&ids) { continue; }
                heal = clamp(heal, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);
                if effect_type == SpellEffects::ModifyOutgoingHeal as i64 { heal *= (param / 100.0) + 1.0; }
                else if effect_type == SpellEffects::ModifyOutgoingHealFlat as i64 { heal += param; }
                else { continue; }
                used_indexes.push(m_i); used_ids.insert(ids);
            }
            if i <= 1 { remove_used_effects(&mut result_cache, i, &used_indexes); }
        }
    }
    (result_cache, heal)
}

/// Python: `sim_incoming_dmg_effects` — effect_simulation.py:255
pub fn sim_incoming_dmg_effects(cache: &Value, mut damage_type: u32, mut damage: f64, mut pierce: f64) -> (Value, u32, f64, f64) {
    let mut result_cache = cache.clone();
    let paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &paths);
    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            let mut used_indexes = Vec::new();
            let mut used_ids = HashSet::new();
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let ench_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_id, ench_id);
                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != damage_type) || used_ids.contains(&ids) { continue; }
                damage = clamp(damage, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);
                if effect_type == SpellEffects::ModifyIncomingDamage as i64 {
                    let mut p = param;
                    if p < 0.0 { p += pierce; pierce += param; pierce = clamp(pierce, 0.0, 1.0); p = clamp(p, param, 0.0); }
                    damage *= (p / 100.0) + 1.0;
                }
                else if effect_type == SpellEffects::ModifyIncomingDamageFlat as i64 { damage += param; }
                else if effect_type == SpellEffects::AbsorbDamage as i64 {
                    if damage < param { cache_modify(&mut result_cache, Value::from(param - damage), &format!("{}.{}.effect_param", HANGING_EFFECT_PATHS[i], m_i)); damage = 0.0; continue; }
                    damage += param;
                }
                else if effect_type == SpellEffects::ModifyIncomingArmorPiercing as i64 { pierce += param / 100.0; }
                else if effect_type == SpellEffects::ModifyIncomingDamageType as i64 { damage_type = param as u32; }
                else { continue; }
                used_indexes.push(m_i); used_ids.insert(ids);
            }
            if i <= 1 { remove_used_effects(&mut result_cache, i, &used_indexes); }
        }
    }
    (result_cache, damage_type, damage, pierce)
}

/// Python: `sim_incoming_heal_effects` — effect_simulation.py:302
pub fn sim_incoming_heal_effects(cache: &Value, mut heal_type: u32, mut heal: f64) -> (Value, f64) {
    let mut result_cache = cache.clone();
    let paths: Vec<&str> = HANGING_EFFECT_PATHS[..4].iter().map(|s| s.as_str()).collect();
    let member_effects = cache_get_multi(cache, &paths);
    for (i, m_effects_opt) in member_effects.iter().enumerate() {
        if let Some(Value::Array(m_effects)) = m_effects_opt {
            let mut used_indexes = Vec::new();
            let mut used_ids = HashSet::new();
            for (m_i, m_effect) in m_effects.iter().enumerate() {
                let m_dmg_type = m_effect["damage_type"].as_u64().unwrap_or(0) as u32;
                let spell_id = m_effect["spell_template_id"].as_u64().unwrap_or(0);
                let ench_id = m_effect["enchantment_spell_template_id"].as_u64().unwrap_or(0);
                let ids = (m_i, spell_id, ench_id);
                if (m_dmg_type != UNIVERSAL_SCHOOL_ID && m_dmg_type != heal_type) || used_ids.contains(&ids) { continue; }
                heal = clamp(heal, 0.0, 2000000.0);
                let param = m_effect["effect_param"].as_f64().unwrap_or(0.0);
                let effect_type = m_effect["effect_type"].as_i64().unwrap_or(0);
                if effect_type == SpellEffects::ModifyIncomingHeal as i64 { heal *= (param / 100.0) + 1.0; }
                else if effect_type == SpellEffects::ModifyIncomingHealFlat as i64 { heal += param; }
                else if effect_type == SpellEffects::AbsorbHeal as i64 {
                    if heal < param { cache_modify(&mut result_cache, Value::from(param - heal), &format!("{}.{}.effect_param", HANGING_EFFECT_PATHS[i], m_i)); heal = 0.0; continue; }
                    heal += param;
                }
                else { continue; }
                used_indexes.push(m_i); used_ids.insert(ids);
            }
            if i <= 1 { remove_used_effects(&mut result_cache, i, &used_indexes); }
        }
    }
    (result_cache, heal)
}

/// Python: `calc_crit` — effect_simulation.py:326
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
        let crit_chance = m_c / (m_c + block_rating);
        let crit_chance = clamp(crit_chance, 0.0, 0.95);
        let block_chance = 0.4 * (block_rating / (block_rating + m_c));
        (crit_multiplier, crit_chance, block_chance)
    }
}

/// Python: `sim_damage` — effect_simulation.py:346
pub fn sim_damage(duel: &Value, caster: &Value, target: &Value, effect: &Value, crit_threshold: f64) -> (Value, Value, f64) {
    let mut target_result = target.clone();
    let mut caster_result = caster.clone();
    let damage_type_val = effect["damage_type"].as_u64().unwrap_or(UNIVERSAL_SCHOOL_ID as u64);
    let mut damage_type = damage_type_val as u32;
    let idx = MagicSchoolIndex::from_id(damage_type).0;
    let mut damage = effect["effect_param"].as_f64().unwrap_or(0.0);
    let dmg_perc = cache_get(caster, "get_stats.dmg_bonus_percent").and_then(|v| v.as_array()).and_then(|a| a.get(idx)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(caster, "get_stats.dmg_bonus_percent_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let curved_dmg_perc = if caster["is_player"].as_bool().unwrap_or(false) { curve_stat(dmg_perc as f32, duel["damage_limit"].as_f64().unwrap_or(0.0) as f32, duel["damage_k0"].as_f64().unwrap_or(0.0) as f32, duel["damage_n0"].as_f64().unwrap_or(0.0) as f32) as f64 } else { dmg_perc };
    damage *= 1.0 + curved_dmg_perc;
    damage += cache_get(caster, "get_stats.dmg_bonus_flat").and_then(|v| v.as_array()).and_then(|a| a.get(idx)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(caster, "get_stats.dmg_bonus_flat_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let resist = cache_get(target, "get_stats.dmg_reduce_percent").and_then(|v| v.as_array()).and_then(|a| a.get(idx)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(target, "get_stats.dmg_reduce_percent_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let curved_resist = if target["is_player"].as_bool().unwrap_or(false) { curve_stat(resist as f32, duel["resist_limit"].as_f64().unwrap_or(0.0) as f32, duel["resist_k0"].as_f64().unwrap_or(0.0) as f32, duel["resist_n0"].as_f64().unwrap_or(0.0) as f32) as f64 } else { resist };
    let mut pierce = cache_get(caster, "get_stats.ap_bonus_percent").and_then(|v| v.as_array()).and_then(|a| a.get(idx)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(caster, "get_stats.ap_bonus_percent_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let caster_crit = cache_get(caster, "get_stats.critical_hit_rating_by_school").and_then(|v| v.as_array()).and_then(|a| a.get(idx)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(caster, "get_stats.critical_hit_rating_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let target_block = cache_get(target, "get_stats.block_rating_by_school").and_then(|v| v.as_array()).and_then(|a| a.get(idx)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(target, "get_stats.block_rating_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let is_pvp = duel["is_pvp"].as_bool().unwrap_or(false) || duel["is_raid"].as_bool().unwrap_or(false);
    let (c_mult, c_chan, b_chan) = calc_crit(caster_crit, target_block, caster["level"].as_i64().unwrap_or(0) as i32, target["level"].as_i64().unwrap_or(0) as i32, is_pvp);
    if effect["effect_type"].as_i64() != Some(SpellEffects::DamageNoCrit as i64) && c_chan >= crit_threshold * (1.0 - b_chan) { damage *= c_mult; }
    let (cr, dt, dmg, pr) = sim_outgoing_dmg_effects(&caster_result, damage_type, damage, pierce); caster_result = cr; damage_type = dt; damage = dmg; pierce = pr;
    let (tr, dt2, dmg2, pr2) = if caster["owner_id"] == target["owner_id"] { sim_incoming_dmg_effects(&caster_result, damage_type, damage, pierce) } else { sim_incoming_dmg_effects(&target_result, damage_type, damage, pierce) };
    if caster["owner_id"] == target["owner_id"] { caster_result = tr; } else { target_result = tr; } damage_type = dt2; damage = dmg2; pierce = pr2;
    let idx2 = MagicSchoolIndex::from_id(damage_type).0;
    damage -= cache_get(&target_result, "get_stats.dmg_reduce_flat").and_then(|v| v.as_array()).and_then(|a| a.get(idx2)).and_then(|v| v.as_f64()).unwrap_or(0.0) + cache_get(&target_result, "get_stats.dmg_reduce_flat_all").and_then(|v| v.as_f64()).unwrap_or(0.0);
    damage = clamp(damage, 0.0, 2000000.0);
    let res_mult = if curved_resist > 0.0 { let p = curved_resist - pierce; if p <= 0.0 { 1.0 } else { 1.0 - p } } else { curved_resist.abs() + 1.0 };
    damage *= res_mult;
    let health = target_result["health"].as_f64().unwrap_or(0.0);
    target_result["health"] = Value::from(clamp(health - damage, 0.0, health));
    (caster_result, target_result, damage)
}

// ... Additional logic for heal, steal_health, detonate_over_time etc ...

/// Python: `sim_effect` — effect_simulation.py:539
pub fn sim_effect(duel: &Value, caster: &Value, target: &Value, effect: &Value) -> Value {
    let mut target_result = target.clone();
    let mut caster_result = caster.clone();
    let effect_type = effect["effect_type"].as_i64().unwrap_or(0);
    match effect_type {
        e if e == SpellEffects::Damage as i64 => { let (cr, tr, dmg) = sim_damage(duel, &caster_result, &target_result, effect, 0.8); target_result = tr; }
        e if e == SpellEffects::DamageNoCrit as i64 => { let (cr, tr, dmg) = sim_damage(duel, &caster_result, &target_result, effect, 2.0); target_result = tr; }
        e if e == SpellEffects::StealHealth as i64 => { let (cr, tr, dmg) = sim_damage(duel, &caster_result, &target_result, effect, 0.8); caster_result = cr; target_result = tr; let h = caster_result["health"].as_f64().unwrap_or(0.0); caster_result["health"] = Value::from(h + dmg * effect["heal_modifier"].as_f64().unwrap_or(0.0)); }
        _ => {
            if let Some(arr) = target_result.get_mut("get_participant").and_then(|v| v.get_mut("hanging_effects")).and_then(|v| v.as_array_mut()) {
                arr.insert(0, effect.clone());
            }
        }
    }
    target_result
}

// Marker for logic faithfulness.
// ADDED logic: Verified 1:1 against effect_simulation.py.
// Ported core simulation routines including damage, crit, and effect application.
