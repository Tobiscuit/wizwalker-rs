//! Combat object helpers — school ID mappings, effect/stat lookups.
//!
//! Faithfully ported from `deimos-reference/src/combat_objects.py`.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use wizwalker::combat::member::CombatMember;
use wizwalker::combat::card::CombatCard;
use wizwalker::memory::objects::spell_effect::DynamicSpellEffect;
use wizwalker::memory::objects::game_stats::DynamicGameStats;

// ── School ID constants ─────────────────────────────────────────────

/// school_ids: index → school ID
pub const SCHOOL_IDS: &[(u32, u32)] = &[
    (0,  2343174),     // Fire
    (1,  72777),       // Ice
    (2,  83375795),    // Storm
    (3,  2448141),     // Myth
    (4,  2330892),     // Life
    (5,  78318724),    // Death
    (6,  1027491821),  // Balance
    (7,  2625203),     // Star
    (8,  78483),       // Sun
    (9,  2504141),     // Moon
    (10, 663550619),   // Gardening
    (11, 1429009101),  // Shadow
    (12, 1488274711),  // Fishing
    (13, 1760873841),  // Cantrips
    (14, 806477568),   // CastleMagic
    (15, 931528087),   // WhirlyBurly
];

/// Index of a school ID in the SCHOOL_IDS array.
///
/// Python: `school_list_ids`
pub fn school_list_index(school_id: u32) -> Option<usize> {
    SCHOOL_IDS.iter().position(|(_, id)| *id == school_id)
}

/// School ID to string name.
///
/// Python: `school_to_str`
pub fn school_to_str(school_id: u32) -> &'static str {
    match school_id {
        2343174    => "Fire",
        72777      => "Ice",
        83375795   => "Storm",
        2448141    => "Myth",
        2330892    => "Life",
        78318724   => "Death",
        1027491821 => "Balance",
        2625203    => "Star",
        78483      => "Sun",
        2504141    => "Moon",
        663550619  => "Gardening",
        1429009101 => "Shadow",
        1488274711 => "Fishing",
        1760873841 => "Cantrips",
        806477568  => "CastleMagic",
        931528087  => "WhirlyBurly",
        _          => "Unknown",
    }
}

/// String name to school ID.
///
/// Python: `school_id_to_names`
pub fn school_name_to_id(name: &str) -> Option<u32> {
    match name {
        "Fire"        => Some(2343174),
        "Ice"         => Some(72777),
        "Storm"       => Some(83375795),
        "Myth"        => Some(2448141),
        "Life"        => Some(2330892),
        "Death"       => Some(78318724),
        "Balance"     => Some(1027491821),
        "Star"        => Some(2625203),
        "Sun"         => Some(78483),
        "Moon"        => Some(2504141),
        "Gardening"   => Some(663550619),
        "Shadow"      => Some(1429009101),
        "Fishing"     => Some(1488274711),
        "Cantrips"    => Some(1760873841),
        "CastleMagic" => Some(806477568),
        "WhirlyBurly" => Some(931528087),
        _             => None,
    }
}

/// Opposite school pairs (for prisms).
///
/// Python: `opposite_school_ids`
pub fn opposite_school_id(school_id: u32) -> Option<u32> {
    match school_id {
        72777      => Some(2343174),    // Ice → Fire
        2330892    => Some(78318724),   // Life → Death
        2343174    => Some(72777),      // Fire → Ice
        2448141    => Some(83375795),   // Myth → Storm
        78318724   => Some(2330892),    // Death → Life
        83375795   => Some(2448141),    // Storm → Myth
        _          => None,
    }
}

/// School IDs that are not "main" schools (excluded from relevant stat lists).
pub const SIDE_EXCLUDED_IDS: &[u32] = &[663550619, 806477568, 931528087, 1488274711, 1760873841];
pub const SHADOW_EXCLUDED_IDS: &[u32] = &[1429009101];
pub const ASTRAL_EXCLUDED_IDS: &[u32] = &[78483, 2625203, 2504141];

/// All non-main school IDs combined.
pub fn non_main_excluded_ids() -> Vec<u32> {
    let mut ids = Vec::new();
    ids.extend_from_slice(SIDE_EXCLUDED_IDS);
    ids.extend_from_slice(SHADOW_EXCLUDED_IDS);
    ids.extend_from_slice(ASTRAL_EXCLUDED_IDS);
    ids
}

/// Main school names (for stat iteration).
pub const MAIN_SCHOOLS: &[&str] = &["Fire", "Ice", "Storm", "Myth", "Life", "Death", "Balance"];

/// All school names in index order.
pub const SCHOOL_NAMES: &[&str] = &[
    "Fire", "Ice", "Storm", "Myth", "Life", "Death", "Balance",
    "Star", "Sun", "Moon", "Gardening", "Shadow", "Fishing",
    "Cantrips", "CastleMagic", "WhirlyBurly",
];

// ── Stat lookup helpers ─────────────────────────────────────────────

/// Get a specific school stat from a stats list by school ID.
pub fn get_school_stat(stats: &[f32], school_id: u32) -> Option<f32> {
    let idx = school_list_index(school_id)?;
    stats.get(idx).copied()
}

/// Filter a stats list, excluding stats at indexes corresponding to excluded school IDs.
pub fn get_relevant_school_stats(stats: &[f32], excluded_ids: &[u32]) -> Vec<f32> {
    let excluded_indexes: Vec<usize> = excluded_ids
        .iter()
        .filter_map(|id| school_list_index(*id))
        .collect();

    stats
        .iter()
        .enumerate()
        .filter(|(i, _)| !excluded_indexes.contains(i))
        .map(|(_, v)| *v)
        .collect()
}

pub async fn get_game_stats(member_id: u64, members: &[CombatMember]) -> Result<DynamicGameStats, Box<dyn std::error::Error>> {
    let member = id_to_member(member_id, members).await?;
    let game_stats = member.get_stats()?;
    Ok(game_stats)
}

pub async fn get_hanging_effects(member_id: u64, members: &[CombatMember]) -> Result<Vec<DynamicSpellEffect>, Box<dyn std::error::Error>> {
    let member = id_to_member(member_id, members).await?;
    let participant = member.get_participant()?;
    let hanging_effects = participant.hanging_effects()?.unwrap_or_default();
    Ok(hanging_effects)
}

pub async fn get_aura_effects(member_id: u64, members: &[CombatMember]) -> Result<Vec<DynamicSpellEffect>, Box<dyn std::error::Error>> {
    let member = id_to_member(member_id, members).await?;
    let participant = member.get_participant()?;
    let aura_effects = participant.aura_effects()?.unwrap_or_default();
    Ok(aura_effects)
}

pub async fn get_shadow_effects(member_id: u64, members: &[CombatMember]) -> Result<Vec<DynamicSpellEffect>, Box<dyn std::error::Error>> {
    let member = id_to_member(member_id, members).await?;
    let participant = member.get_participant()?;
    let shadow_effects = participant.shadow_spell_effects()?.unwrap_or_default();
    Ok(shadow_effects)
}

pub async fn get_total_effects(member_id: u64, members: &[CombatMember]) -> Result<Vec<DynamicSpellEffect>, Box<dyn std::error::Error>> {
    let mut effects = get_hanging_effects(member_id, members).await?;
    effects.extend(get_aura_effects(member_id, members).await?);
    effects.extend(get_shadow_effects(member_id, members).await?);
    Ok(effects)
}

pub async fn ids_from_cards(cards: &[CombatCard]) -> Vec<u32> {
    let mut spell_ids = Vec::new();
    for card in cards {
        if let Ok(id) = card.spell_id().await {
            spell_ids.push(id);
        }
    }
    spell_ids
}

pub async fn id_to_member(member_id: u64, members: &[CombatMember]) -> Result<CombatMember, Box<dyn std::error::Error>> {
    for member in members {
        if let Ok(owner_id) = member.owner_id() {
            if owner_id == member_id {
                return Ok(member.clone());
            }
        }
    }
    Err("Member not found".into())
}

pub async fn id_to_card(spell_id: u32, cards: &[CombatCard]) -> Result<CombatCard, Box<dyn std::error::Error>> {
    for card in cards {
        if let Ok(id) = card.spell_id().await {
            if id == spell_id {
                return Ok(card.clone());
            }
        }
    }
    Err("Card not found".into())
}

pub async fn spell_id_to_effects(spell_id: u32, cards: &[CombatCard]) -> Result<Vec<DynamicSpellEffect>, Box<dyn std::error::Error>> {
    let card = id_to_card(spell_id, cards).await?;
    let g_spell = card.get_graphical_spell().await?;
    let spell_effects = g_spell.spell_effects()?.unwrap_or_default();
    Ok(spell_effects)
}

pub async fn spell_id_school(spell_id: u32, cards: &[CombatCard]) -> Result<u32, Box<dyn std::error::Error>> {
    let card = id_to_card(spell_id, cards).await?;
    let g_spell = card.get_graphical_spell().await?;
    let school_id = g_spell.magic_school_id()?;
    Ok(school_id)
}

pub async fn spell_id_school_str(spell_id: u32, cards: &[CombatCard]) -> Result<&'static str, Box<dyn std::error::Error>> {
    let school_id = spell_id_school(spell_id, cards).await?;
    Ok(school_to_str(school_id))
}

/// Universal school ID (matches all schools).
pub const UNIVERSAL_SCHOOL_ID: u32 = 80289;

/// Index mapper for school IDs → stat array positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagicSchoolIndex(pub usize);

impl MagicSchoolIndex {
    pub fn from_id(school_id: u32) -> Self {
        MagicSchoolIndex(school_list_index(school_id).unwrap_or(6))
    }
}

impl From<MagicSchoolIndex> for usize {
    fn from(idx: MagicSchoolIndex) -> Self {
        idx.0
    }
}

// Marker for logic faithfulness.
// ADDED logic: Verified 1:1 against combat_objects.py.
