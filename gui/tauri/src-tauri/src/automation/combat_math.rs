//! Combat math — damage calculation engine with stat curving, crit, hanging effects.
//!
//! Faithfully ported from `deimos-reference/src/combat_math.py`.
#![allow(dead_code, unused_imports)]

use super::combat_objects::{school_list_index, UNIVERSAL_SCHOOL_ID};
use super::combat_utils::add_universal_stat;

// ── Effect attributes cache ─────────────────────────────────────────

/// Non-async cache of spell effect attributes used in damage calculations.
///
/// Python: `EffectAttributes` dataclass — combat_math.py:12
#[derive(Debug, Clone)]
pub struct EffectAttributes {
    pub effect_param: f32,
    pub effect_type: u32,
    pub damage_type: u32,
    pub spell_template_id: u32,
    pub enchantment_spell_template_id: u32,
}

// ── Stat curving ────────────────────────────────────────────────────

/// Curve a stat in the same way the game does with resist/damage past an intersection.
///
/// Python: `curve_stat(stat, l, k0, n0)` — combat_math.py:39
pub fn curve_stat(stat: f32, l: f32, k0: f32, n0: f32) -> f32 {
    if stat > (k0 + n0) / 100.0 {
        let limit = l * 100.0;

        let k = if k0 != 0.0 {
            (limit / (limit - k0)).ln() / k0
        } else {
            1.0 / limit
        };

        let n = (1.0 - (k0 + n0) / limit).ln() + k * (k0 + n0);

        l - l * std::f32::consts::E.powf(-k * (stat * 100.0) + n)
    } else {
        stat
    }
}

/// Calculate a stacking ID for a spell effect.
///
/// If two stacking IDs match, the spell effects do not stack.
/// Aegis/Indemnity don't stack with un-enchanted versions.
///
/// Python: `spell_effect_stacking_id(spell_template_id, enchantment_id)` — combat_math.py:80
pub fn spell_effect_stacking_id(
    spell_template_id: u32,
    enchantment_spell_template_id: u32,
) -> (u32, u32) {
    const AEGIS: u32 = 85300353;
    const INDEMNITY: u32 = 655113637;

    let effective_enchant = if enchantment_spell_template_id == AEGIS
        || enchantment_spell_template_id == INDEMNITY
    {
        0 // For stacking purposes, these are the same as unenchanted
    } else {
        enchantment_spell_template_id
    };

    (spell_template_id, effective_enchant)
}

// ── SpellEffects enum values (from Python wizwalker) ────────────────

/// Known spell effect type IDs used in damage calculation.
///
/// These match the `SpellEffects` enum values from wizwalker.
pub mod spell_effects {
    pub const MODIFY_OUTGOING_DAMAGE: u32 = 1;
    pub const MODIFY_OUTGOING_DAMAGE_FLAT: u32 = 2;
    pub const MODIFY_OUTGOING_ARMOR_PIERCING: u32 = 3;
    pub const MODIFY_OUTGOING_DAMAGE_TYPE: u32 = 4;
    pub const MODIFY_INCOMING_DAMAGE: u32 = 5;
    pub const MODIFY_INCOMING_DAMAGE_FLAT: u32 = 6;
    pub const MODIFY_INCOMING_ARMOR_PIERCING: u32 = 7;
    pub const MODIFY_INCOMING_DAMAGE_TYPE: u32 = 8;
    pub const ABSORB_DAMAGE: u32 = 9;
    pub const INTERCEPT: u32 = 10;
}

// ── Main damage calculation ─────────────────────────────────────────

/// Full damage calculation pipeline.
///
/// Faithfully ported from Python `base_damage_calculation_from_id()` — combat_math.py:98.
///
/// This handles:
/// 1. Apply curved damage stat + flat damage
/// 2. Process outgoing hanging effects (blades/weakness)
/// 3. Process incoming hanging effects (traps/shields/prisms/absorbs)
/// 4. Apply critical multiplier if chance ≥ 85%
/// 5. Apply flat resist
/// 6. Apply curved resist accounting for pierce
///
/// # Arguments
/// - `damage` — base damage value of the spell
/// - `damage_type` — school ID of the spell
/// - `caster_damages` — per-school damage stats (list of 16)
/// - `caster_flat_damages` — per-school flat damage stats
/// - `caster_crits` — per-school crit ratings
/// - `caster_pierces` — per-school pierce stats
/// - `caster_level` — caster's level
/// - `target_resistances` — per-school resist stats
/// - `target_flat_resistances` — per-school flat resist stats
/// - `target_blocks` — per-school block ratings
/// - `caster_effects` — caster's hanging effects (blades, etc.)
/// - `target_effects` — target's hanging effects (traps, shields, etc.)
/// - `force_crit` — override crit check
pub fn base_damage_calculation(
    damage: f32,
    initial_damage_type: u32,
    caster_damages: &[f32],
    caster_flat_damages: &[f32],
    caster_crits: &[f32],
    caster_pierces: &[f32],
    caster_level: u32,
    target_resistances: &[f32],
    target_flat_resistances: &[f32],
    target_blocks: &[f32],
    caster_effects: &[EffectAttributes],
    target_effects: &[EffectAttributes],
    force_crit: Option<bool>,
    // Curve parameters (from duel object)
    damage_limit: f32,
    d_k0: f32,
    d_n0: f32,
    resist_limit: f32,
    r_k0: f32,
    r_n0: f32,
    caster_is_player: bool,
    target_is_player: bool,
) -> f32 {
    let initial_idx = school_list_index(initial_damage_type).unwrap_or(0);

    // Get caster stats for the spell's school
    let caster_damage = caster_damages.get(initial_idx).copied().unwrap_or(0.0);
    let caster_flat = caster_flat_damages.get(initial_idx).copied().unwrap_or(0.0);
    let caster_crit = caster_crits.get(initial_idx).copied().unwrap_or(0.0);
    let mut caster_pierce = caster_pierces.get(initial_idx).copied().unwrap_or(0.0);

    // Curve damage stat
    let curved_damage = if caster_is_player {
        curve_stat(caster_damage, damage_limit, d_k0, d_n0) + 1.0
    } else {
        caster_damage + 1.0
    };

    // Apply curved damage and flat damage
    let mut damage = damage * curved_damage + caster_flat;
    let mut damage_type = initial_damage_type;

    // ── Outgoing hanging effects (caster) ──
    let mut seen_caster_stacking: std::collections::HashSet<(u32, u32)> =
        std::collections::HashSet::new();

    for effect in caster_effects {
        let stacking_id = spell_effect_stacking_id(
            effect.spell_template_id,
            effect.enchantment_spell_template_id,
        );
        if seen_caster_stacking.contains(&stacking_id) {
            continue;
        }
        if effect.damage_type != damage_type && effect.damage_type != UNIVERSAL_SCHOOL_ID {
            continue;
        }

        seen_caster_stacking.insert(stacking_id);

        match effect.effect_type {
            spell_effects::MODIFY_OUTGOING_DAMAGE => {
                damage *= (effect.effect_param / 100.0) + 1.0;
            }
            spell_effects::MODIFY_OUTGOING_DAMAGE_FLAT => {
                damage += effect.effect_param;
            }
            spell_effects::MODIFY_OUTGOING_ARMOR_PIERCING => {
                caster_pierce += effect.effect_param;
            }
            spell_effects::MODIFY_OUTGOING_DAMAGE_TYPE => {
                damage_type = effect.effect_param as u32;
            }
            _ => {}
        }
    }

    // ── Incoming hanging effects (target) ──
    let mut seen_target_stacking: std::collections::HashSet<(u32, u32)> =
        std::collections::HashSet::new();

    for effect in target_effects {
        let stacking_id = spell_effect_stacking_id(
            effect.spell_template_id,
            effect.enchantment_spell_template_id,
        );
        if seen_target_stacking.contains(&stacking_id) {
            continue;
        }
        if effect.damage_type != damage_type && effect.damage_type != UNIVERSAL_SCHOOL_ID {
            continue;
        }

        seen_target_stacking.insert(stacking_id);

        match effect.effect_type {
            spell_effects::MODIFY_INCOMING_DAMAGE => {
                let mut ward_param = effect.effect_param;
                if ward_param < 0.0 {
                    ward_param += caster_pierce;
                    caster_pierce += effect.effect_param;
                    if ward_param > 0.0 {
                        ward_param = 0.0;
                    }
                    if caster_pierce < 0.0 {
                        caster_pierce = 0.0;
                    }
                }
                damage *= (ward_param / 100.0) + 1.0;
            }
            spell_effects::INTERCEPT => {
                damage *= (effect.effect_param / 100.0) + 1.0;
            }
            spell_effects::MODIFY_INCOMING_DAMAGE_FLAT => {
                damage += effect.effect_param;
            }
            spell_effects::ABSORB_DAMAGE => {
                damage += effect.effect_param;
            }
            spell_effects::MODIFY_INCOMING_ARMOR_PIERCING => {
                caster_pierce += effect.effect_param;
            }
            spell_effects::MODIFY_INCOMING_DAMAGE_TYPE => {
                damage_type = effect.effect_param as u32;
            }
            _ => {}
        }
    }

    // Final damage type stats
    let final_idx = school_list_index(damage_type).unwrap_or(0);
    let target_resist = target_resistances.get(final_idx).copied().unwrap_or(0.0);
    let target_flat_resist = target_flat_resistances.get(final_idx).copied().unwrap_or(0.0);
    let target_block = target_blocks.get(final_idx).copied().unwrap_or(0.0);

    // Curve resist
    let curved_resist = if target_is_player {
        curve_stat(target_resist, resist_limit, r_k0, r_n0)
    } else {
        target_resist
    };

    // ── Critical ──
    if caster_crit > 0.0 {
        let clamped_level = caster_level.min(100) as f32;
        let crit_multiplier = 2.0 - (target_block / ((caster_crit / 3.0) + target_block));
        let client_crit = 0.03 * clamped_level * caster_crit;
        let mob_block = 3.0 * caster_crit + target_block;
        let crit_chance = client_crit / mob_block;

        let should_crit = match force_crit {
            Some(true) => true,
            Some(false) => false,
            None => crit_chance >= 0.85,
        };

        if should_crit {
            damage *= crit_multiplier;
        }
    }

    // Apply flat resist
    damage -= target_flat_resist;
    damage = damage.abs();

    // Apply percent resist accounting for pierce
    let resist_mult = if curved_resist > 0.0 {
        let pierced = curved_resist - caster_pierce;
        if pierced <= 0.0 {
            1.0 // Pierce fully negates resist
        } else {
            1.0 - pierced
        }
    } else {
        // Negative resist = boost
        curved_resist.abs() + 1.0
    };

    damage *= resist_mult;

    damage
}
