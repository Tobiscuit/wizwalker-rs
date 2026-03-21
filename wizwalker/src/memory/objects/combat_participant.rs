//! Combat participant memory object — represents a wizard or mob in combat.
//!
//! # Python equivalent
//! `wizwalker/memory/memory_objects/combat_participant.py` — `CombatParticipant` / `DynamicCombatParticipant`.
//!
//! All offsets are taken directly from the Python source.
//! Pips are read via a `PipCount` sub-object at offset 152, not inline.

use crate::errors::{Result, WizWalkerError};
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};
use crate::memory::objects::game_stats::DynamicGameStats;
use crate::memory::objects::spell_effect::DynamicSpellEffect;

/// A concrete combat participant — wraps a `DynamicMemoryObject` pointing at
/// the combat participant data in the game's memory.
///
/// All offsets match the Python WizWalker source exactly.
#[derive(Clone)]
pub struct DynamicCombatParticipant {
    pub inner: DynamicMemoryObject,
}

impl DynamicCombatParticipant {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    // ── Identity ────────────────────────────────────────────────────

    /// Owner ID (global ID of the wizard who controls this participant).
    /// Python offset: 112, Primitive.uint64
    pub fn owner_id_full(&self) -> Result<u64> {
        self.inner.read_value_from_offset(112)
    }

    pub fn write_owner_id_full(&self, val: u64) -> Result<()> {
        self.inner.write_value_to_offset(112, &val)
    }

    /// Template ID.
    /// Python offset: 120, Primitive.uint64
    pub fn template_id_full(&self) -> Result<u64> {
        self.inner.read_value_from_offset(120)
    }

    pub fn write_template_id_full(&self, val: u64) -> Result<()> {
        self.inner.write_value_to_offset(120, &val)
    }

    /// Is this participant a player (wizard)?
    /// Python offset: 128, Primitive.bool
    pub fn is_player(&self) -> Result<bool> {
        self.inner.read_value_from_offset(128)
    }

    pub fn write_is_player(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(128, &val)
    }

    /// Zone ID.
    /// Python offset: 136, Primitive.uint64
    pub fn zone_id_full(&self) -> Result<u64> {
        self.inner.read_value_from_offset(136)
    }

    pub fn write_zone_id_full(&self, val: u64) -> Result<()> {
        self.inner.write_value_to_offset(136, &val)
    }

    /// Team ID.
    /// Python offset: 144, Primitive.int32
    pub fn team_id(&self) -> Result<i32> {
        self.inner.read_value_from_offset(144)
    }

    pub fn write_team_id(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(144, &val)
    }

    /// Primary magic school ID (template ID).
    /// Python offset: 148, Primitive.int32
    pub fn primary_magic_school_id(&self) -> Result<i32> {
        self.inner.read_value_from_offset(148)
    }

    pub fn write_primary_magic_school_id(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(148, &val)
    }

    // ── Pips (via PipCount pointer at offset 152) ───────────────────

    /// Read the PipCount pointer. Pips are stored in a sub-object.
    /// Python offset: 152, Primitive.int64 (pointer to PipCount)
    fn pip_count_address(&self) -> Result<u64> {
        let addr: u64 = self.inner.read_value_from_offset(152)?;
        if addr == 0 {
            return Err(WizWalkerError::Other("PipCount pointer is null".into()));
        }
        Ok(addr)
    }

    /// Generic (normal) pips.
    /// PipCount.generic_pips is at PipCount + some offset.
    /// Python: pipcount.generic_pips() — PipCount offset 72 for generic_pips
    pub fn num_pips(&self) -> Result<i32> {
        let pip_addr = self.pip_count_address()?;
        let pip_obj = DynamicMemoryObject::new(self.inner.reader(), pip_addr)?;
        // PipCount.generic_pips offset = 72 (from pip_count.py)
        pip_obj.read_value_from_offset(72)
    }

    /// Power pips.
    /// PipCount.power_pips offset = 76
    pub fn num_power_pips(&self) -> Result<i32> {
        let pip_addr = self.pip_count_address()?;
        let pip_obj = DynamicMemoryObject::new(self.inner.reader(), pip_addr)?;
        pip_obj.read_value_from_offset(76)
    }

    /// Shadow pips.
    /// PipCount.shadow_pips offset = 80
    pub fn num_shadow_pips(&self) -> Result<i32> {
        let pip_addr = self.pip_count_address()?;
        let pip_obj = DynamicMemoryObject::new(self.inner.reader(), pip_addr)?;
        pip_obj.read_value_from_offset(80)
    }

    /// Pips suspended.
    /// Python offset: 184, Primitive.bool
    pub fn pips_suspended(&self) -> Result<bool> {
        self.inner.read_value_from_offset(184)
    }

    pub fn write_pips_suspended(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(184, &val)
    }

    // ── Combat State ────────────────────────────────────────────────

    /// Stun counter. 0 = not stunned.
    /// Python offset: 188, Primitive.int32
    pub fn stunned(&self) -> Result<i32> {
        self.inner.read_value_from_offset(188)
    }

    pub fn write_stunned(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(188, &val)
    }

    /// Stunned display flag.
    /// Python offset: 192, Primitive.bool
    pub fn stunned_display(&self) -> Result<bool> {
        self.inner.read_value_from_offset(192)
    }

    pub fn write_stunned_display(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(192, &val)
    }

    /// Confused counter.
    /// Python offset: 196, Primitive.int32
    pub fn confused(&self) -> Result<i32> {
        self.inner.read_value_from_offset(196)
    }

    pub fn write_confused(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(196, &val)
    }

    /// Confusion trigger.
    /// Python offset: 200, Primitive.int32
    pub fn confusion_trigger(&self) -> Result<i32> {
        self.inner.read_value_from_offset(200)
    }

    pub fn write_confusion_trigger(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(200, &val)
    }

    /// Confusion display.
    /// Python offset: 204, Primitive.bool
    pub fn confusion_display(&self) -> Result<bool> {
        self.inner.read_value_from_offset(204)
    }

    pub fn write_confusion_display(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(204, &val)
    }

    /// Confused target.
    /// Python offset: 205, Primitive.bool
    pub fn confused_target(&self) -> Result<bool> {
        self.inner.read_value_from_offset(205)
    }

    pub fn write_confused_target(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(205, &val)
    }

    /// Untargetable flag.
    /// Python offset: 206, Primitive.bool
    pub fn untargetable(&self) -> Result<bool> {
        self.inner.read_value_from_offset(206)
    }

    pub fn write_untargetable(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(206, &val)
    }

    /// Untargetable rounds counter.
    /// Python offset: 208, Primitive.int32
    pub fn untargetable_rounds(&self) -> Result<i32> {
        self.inner.read_value_from_offset(208)
    }

    pub fn write_untargetable_rounds(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(208, &val)
    }

    /// Restricted target flag.
    /// Python offset: 212, Primitive.bool
    pub fn restricted_target(&self) -> Result<bool> {
        self.inner.read_value_from_offset(212)
    }

    pub fn write_restricted_target(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(212, &val)
    }

    /// Exit combat flag.
    /// Python offset: 213, Primitive.bool
    pub fn exit_combat(&self) -> Result<bool> {
        self.inner.read_value_from_offset(213)
    }

    pub fn write_exit_combat(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(213, &val)
    }

    /// Mind controlled counter.
    /// Python offset: 216, Primitive.int32
    pub fn mindcontrolled(&self) -> Result<i32> {
        self.inner.read_value_from_offset(216)
    }

    pub fn write_mindcontrolled(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(216, &val)
    }

    /// Mind controlled display.
    /// Python offset: 220, Primitive.bool
    pub fn mindcontrolled_display(&self) -> Result<bool> {
        self.inner.read_value_from_offset(220)
    }

    pub fn write_mindcontrolled_display(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(220, &val)
    }

    /// Original team (before mind control).
    /// Python offset: 224, Primitive.int32
    pub fn original_team(&self) -> Result<i32> {
        self.inner.read_value_from_offset(224)
    }

    pub fn write_original_team(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(224, &val)
    }

    /// Clue.
    /// Python offset: 228, Primitive.int32
    pub fn clue(&self) -> Result<i32> {
        self.inner.read_value_from_offset(228)
    }

    pub fn write_clue(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(228, &val)
    }

    /// Rounds dead.
    /// Python offset: 232, Primitive.int32
    pub fn rounds_dead(&self) -> Result<i32> {
        self.inner.read_value_from_offset(232)
    }

    pub fn write_rounds_dead(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(232, &val)
    }

    /// Aura turn length.
    /// Python offset: 236, Primitive.int32
    pub fn aura_turn_length(&self) -> Result<i32> {
        self.inner.read_value_from_offset(236)
    }

    pub fn write_aura_turn_length(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(236, &val)
    }

    /// Polymorph turn length.
    /// Python offset: 240, Primitive.int32
    pub fn polymorph_turn_length(&self) -> Result<i32> {
        self.inner.read_value_from_offset(240)
    }

    pub fn write_polymorph_turn_length(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(240, &val)
    }

    // ── Health ──────────────────────────────────────────────────────

    /// Current player health.
    /// Python offset: 244, Primitive.int32
    pub fn player_health(&self) -> Result<i32> {
        self.inner.read_value_from_offset(244)
    }

    pub fn write_player_health(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(244, &val)
    }

    /// Max player health.
    /// Python offset: 248, Primitive.int32
    pub fn max_player_health(&self) -> Result<i32> {
        self.inner.read_value_from_offset(248)
    }

    pub fn write_max_player_health(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(248, &val)
    }

    /// Hide current HP flag.
    /// Python offset: 252, Primitive.bool
    pub fn hide_current_hp(&self) -> Result<bool> {
        self.inner.read_value_from_offset(252)
    }

    pub fn write_hide_current_hp(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(252, &val)
    }

    /// Max hand size.
    /// Python offset: 256, Primitive.int32
    pub fn max_hand_size(&self) -> Result<i32> {
        self.inner.read_value_from_offset(256)
    }

    pub fn write_max_hand_size(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(256, &val)
    }

    // ── Game Stats ──────────────────────────────────────────────────

    /// Saved game stats pointer.
    /// Python offset: 296, Primitive.int64
    pub fn saved_game_stats(&self) -> Result<Option<DynamicGameStats>> {
        let addr: u64 = self.inner.read_value_from_offset(296)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicGameStats { inner }))
    }

    /// Saved primary magic school ID.
    /// Python offset: 312, Primitive.int32
    pub fn saved_primary_magic_school_id(&self) -> Result<i32> {
        self.inner.read_value_from_offset(312)
    }

    pub fn write_saved_primary_magic_school_id(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(312, &val)
    }

    /// Current game stats pointer.
    /// Python offset: 320, Primitive.int64
    pub fn game_stats(&self) -> Result<Option<DynamicGameStats>> {
        let addr: u64 = self.inner.read_value_from_offset(320)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicGameStats { inner }))
    }

    // ── Position & Geometry ─────────────────────────────────────────

    /// Rotation.
    /// Python offset: 340, Primitive.float32
    pub fn rotation(&self) -> Result<f32> {
        self.inner.read_value_from_offset(340)
    }

    pub fn write_rotation(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(340, &val)
    }

    /// Radius.
    /// Python offset: 344, Primitive.float32
    pub fn radius(&self) -> Result<f32> {
        self.inner.read_value_from_offset(344)
    }

    pub fn write_radius(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(344, &val)
    }

    /// Subcircle.
    /// Python offset: 348, Primitive.int32
    pub fn subcircle(&self) -> Result<i32> {
        self.inner.read_value_from_offset(348)
    }

    pub fn write_subcircle(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(348, &val)
    }

    /// PVP flag.
    /// Python offset: 352, Primitive.bool
    pub fn pvp(&self) -> Result<bool> {
        self.inner.read_value_from_offset(352)
    }

    pub fn write_pvp(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(352, &val)
    }

    /// Raid flag.
    /// Python offset: 353, Primitive.bool
    pub fn raid(&self) -> Result<bool> {
        self.inner.read_value_from_offset(353)
    }

    pub fn write_raid(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(353, &val)
    }

    /// Shadow pact target.
    /// Python offset: 392, Primitive.int32
    pub fn shadow_pact_target(&self) -> Result<i32> {
        self.inner.read_value_from_offset(392)
    }

    pub fn write_shadow_pact_target(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(392, &val)
    }

    /// Accuracy bonus.
    /// Python offset: 400, Primitive.float32
    pub fn accuracy_bonus(&self) -> Result<f32> {
        self.inner.read_value_from_offset(400)
    }

    pub fn write_accuracy_bonus(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(400, &val)
    }

    /// Minion subcircle.
    /// Python offset: 404, Primitive.int32
    pub fn minion_sub_circle(&self) -> Result<i32> {
        self.inner.read_value_from_offset(404)
    }

    pub fn write_minion_sub_circle(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(404, &val)
    }

    /// Is minion.
    /// Python offset: 408, Primitive.bool
    pub fn is_minion(&self) -> Result<bool> {
        self.inner.read_value_from_offset(408)
    }

    pub fn write_is_minion(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(408, &val)
    }

    /// Is monster.
    /// Python offset: 412, Primitive.uint32
    pub fn is_monster(&self) -> Result<u32> {
        self.inner.read_value_from_offset(412)
    }

    pub fn write_is_monster(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(412, &val)
    }

    /// Is accompany NPC.
    /// Python offset: 416, Primitive.bool
    pub fn is_accompany_npc(&self) -> Result<bool> {
        self.inner.read_value_from_offset(416)
    }

    pub fn write_is_accompany_npc(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(416, &val)
    }

    // ── Hanging Effects (linked lists of spell effects) ─────────────

    /// Hanging effects (wards, traps, etc).
    /// Python offset: 424, via read_linked_list
    pub fn hanging_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_linked_list(424)?;
        addrs
            .into_iter()
            .map(|addr| {
                DynamicSpellEffect::from_addr(self.inner.reader(), addr as u64)
            })
            .collect()
    }

    /// Public hanging effects.
    /// Python offset: 440, via read_linked_list
    pub fn public_hanging_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_linked_list(440)?;
        addrs
            .into_iter()
            .map(|addr| {
                DynamicSpellEffect::from_addr(self.inner.reader(), addr as u64)
            })
            .collect()
    }

    /// Aura effects.
    /// Python offset: 456, via read_linked_list
    pub fn aura_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_linked_list(456)?;
        addrs
            .into_iter()
            .map(|addr| {
                DynamicSpellEffect::from_addr(self.inner.reader(), addr as u64)
            })
            .collect()
    }

    /// Shadow spell effects.
    /// Python offset: 488, via read_linked_list
    pub fn shadow_spell_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_linked_list(488)?;
        addrs
            .into_iter()
            .map(|addr| {
                DynamicSpellEffect::from_addr(self.inner.reader(), addr as u64)
            })
            .collect()
    }

    /// Death activated effects.
    /// Python offset: 520, via read_shared_linked_list
    pub fn death_activated_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_shared_linked_list(520)?;
        addrs
            .into_iter()
            .map(|addr| {
                DynamicSpellEffect::from_addr(self.inner.reader(), addr as u64)
            })
            .collect()
    }

    /// Delay cast effects.
    /// Python offset: 536, via read_linked_list
    pub fn delay_cast_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_linked_list(536)?;
        addrs
            .into_iter()
            .map(|addr| {
                DynamicSpellEffect::from_addr(self.inner.reader(), addr as u64)
            })
            .collect()
    }

    // ── Polymorph & Spells ──────────────────────────────────────────

    /// Polymorph spell template ID.
    /// Python offset: 584, Primitive.uint32
    pub fn polymorph_spell_template_id(&self) -> Result<u32> {
        self.inner.read_value_from_offset(584)
    }

    pub fn write_polymorph_spell_template_id(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(584, &val)
    }

    /// Shadow spells disabled.
    /// Python offset: 680, Primitive.bool
    pub fn shadow_spells_disabled(&self) -> Result<bool> {
        self.inner.read_value_from_offset(680)
    }

    pub fn write_shadow_spells_disabled(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(680, &val)
    }

    /// Ignore spells PVP only flag.
    /// Python offset: 681, Primitive.bool
    pub fn ignore_spells_pvp_only_flag(&self) -> Result<bool> {
        self.inner.read_value_from_offset(681)
    }

    pub fn write_ignore_spells_pvp_only_flag(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(681, &val)
    }

    /// Ignore spells PVE only flag.
    /// Python offset: 682, Primitive.bool
    pub fn ignore_spells_pve_only_flag(&self) -> Result<bool> {
        self.inner.read_value_from_offset(682)
    }

    pub fn write_ignore_spells_pve_only_flag(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(682, &val)
    }

    /// Boss mob (fallback — prefer `fetch_npc_behavior_template` when possible).
    /// Python offset: 683, Primitive.bool
    pub fn boss_mob(&self) -> Result<bool> {
        self.inner.read_value_from_offset(683)
    }

    /// Hide PVP enemy chat.
    /// Python offset: 684, Primitive.bool
    pub fn hide_pvp_enemy_chat(&self) -> Result<bool> {
        self.inner.read_value_from_offset(684)
    }

    pub fn write_hide_pvp_enemy_chat(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(684, &val)
    }

    // ── Combat Triggers & Misc ──────────────────────────────────────

    /// Combat trigger IDs.
    /// Python offset: 704, Primitive.int32
    pub fn combat_trigger_ids(&self) -> Result<i32> {
        self.inner.read_value_from_offset(704)
    }

    pub fn write_combat_trigger_ids(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(704, &val)
    }

    /// Pet combat trigger.
    /// Python offset: 720, Primitive.int32
    pub fn pet_combat_trigger(&self) -> Result<i32> {
        self.inner.read_value_from_offset(720)
    }

    pub fn write_pet_combat_trigger(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(720, &val)
    }

    /// Pet combat trigger target.
    /// Python offset: 724, Primitive.int32
    pub fn pet_combat_trigger_target(&self) -> Result<i32> {
        self.inner.read_value_from_offset(724)
    }

    pub fn write_pet_combat_trigger_target(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(724, &val)
    }

    /// Auto pass.
    /// Python offset: 728, Primitive.bool
    pub fn auto_pass(&self) -> Result<bool> {
        self.inner.read_value_from_offset(728)
    }

    pub fn write_auto_pass(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(728, &val)
    }

    /// Vanish flag.
    /// Python offset: 729, Primitive.bool
    pub fn vanish(&self) -> Result<bool> {
        self.inner.read_value_from_offset(729)
    }

    pub fn write_vanish(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(729, &val)
    }

    /// My team turn.
    /// Python offset: 730, Primitive.bool
    pub fn my_team_turn(&self) -> Result<bool> {
        self.inner.read_value_from_offset(730)
    }

    pub fn write_my_team_turn(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(730, &val)
    }

    // ── Backlash & Shadow ───────────────────────────────────────────

    /// Current backlash.
    /// Python offset: 732, Primitive.int32
    pub fn backlash(&self) -> Result<i32> {
        self.inner.read_value_from_offset(732)
    }

    pub fn write_backlash(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(732, &val)
    }

    /// Past backlash.
    /// Python offset: 736, Primitive.int32
    pub fn past_backlash(&self) -> Result<i32> {
        self.inner.read_value_from_offset(736)
    }

    pub fn write_past_backlash(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(736, &val)
    }

    /// Shadow creature level.
    /// Python offset: 740, Primitive.int32
    pub fn shadow_creature_level(&self) -> Result<i32> {
        self.inner.read_value_from_offset(740)
    }

    pub fn write_shadow_creature_level(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(740, &val)
    }

    /// Past shadow creature level.
    /// Python offset: 744, Primitive.int32
    pub fn past_shadow_creature_level(&self) -> Result<i32> {
        self.inner.read_value_from_offset(744)
    }

    pub fn write_past_shadow_creature_level(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(744, &val)
    }

    /// Shadow creature level count.
    /// Python offset: 752, Primitive.int32
    pub fn shadow_creature_level_count(&self) -> Result<i32> {
        self.inner.read_value_from_offset(752)
    }

    pub fn write_shadow_creature_level_count(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(752, &val)
    }

    /// Intercept effect pointer.
    /// Python offset: 776, Primitive.int64
    pub fn intercept_effect(&self) -> Result<Option<DynamicSpellEffect>> {
        let addr: u64 = self.inner.read_value_from_offset(776)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(DynamicSpellEffect::from_addr(self.inner.reader(), addr)?))
    }

    /// Rounds since shadow pip.
    /// Python offset: 808, Primitive.int32
    pub fn rounds_since_shadow_pip(&self) -> Result<i32> {
        self.inner.read_value_from_offset(808)
    }

    pub fn write_rounds_since_shadow_pip(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(808, &val)
    }

    /// Polymorph effect pointer.
    /// Python offset: 832, Primitive.int64
    pub fn polymorph_effect(&self) -> Result<Option<DynamicSpellEffect>> {
        let addr: u64 = self.inner.read_value_from_offset(832)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(DynamicSpellEffect::from_addr(self.inner.reader(), addr)?))
    }

    // ── Archmastery & Rate ──────────────────────────────────────────

    /// Shadow pip rate threshold.
    /// Python offset: 848, Primitive.float32
    pub fn shadow_pip_rate_threshold(&self) -> Result<f32> {
        self.inner.read_value_from_offset(848)
    }

    pub fn write_shadow_pip_rate_threshold(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(848, &val)
    }

    /// Base spell damage.
    /// Python offset: 852, Primitive.int32
    pub fn base_spell_damage(&self) -> Result<i32> {
        self.inner.read_value_from_offset(852)
    }

    pub fn write_base_spell_damage(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(852, &val)
    }

    /// Stat damage.
    /// Python offset: 856, Primitive.float32
    pub fn stat_damage(&self) -> Result<f32> {
        self.inner.read_value_from_offset(856)
    }

    pub fn write_stat_damage(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(856, &val)
    }

    /// Stat resist.
    /// Python offset: 860, Primitive.float32
    pub fn stat_resist(&self) -> Result<f32> {
        self.inner.read_value_from_offset(860)
    }

    pub fn write_stat_resist(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(860, &val)
    }

    /// Stat pierce.
    /// Python offset: 864, Primitive.float32
    pub fn stat_pierce(&self) -> Result<f32> {
        self.inner.read_value_from_offset(864)
    }

    pub fn write_stat_pierce(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(864, &val)
    }

    /// Mob level.
    /// Python offset: 868, Primitive.int32
    pub fn mob_level(&self) -> Result<i32> {
        self.inner.read_value_from_offset(868)
    }

    pub fn write_mob_level(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(868, &val)
    }

    /// Player time updated flag.
    /// Python offset: 872, Primitive.bool
    pub fn player_time_updated(&self) -> Result<bool> {
        self.inner.read_value_from_offset(872)
    }

    pub fn write_player_time_updated(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(872, &val)
    }

    /// Player time eliminated flag.
    /// Python offset: 873, Primitive.bool
    pub fn player_time_eliminated(&self) -> Result<bool> {
        self.inner.read_value_from_offset(873)
    }

    pub fn write_player_time_eliminated(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(873, &val)
    }

    /// Player time warning.
    /// Python offset: 874, Primitive.bool
    pub fn player_time_warning(&self) -> Result<bool> {
        self.inner.read_value_from_offset(874)
    }

    pub fn write_player_time_warning(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(874, &val)
    }

    /// Deck fullness.
    /// Python offset: 876, Primitive.float32
    pub fn deck_fullness(&self) -> Result<f32> {
        self.inner.read_value_from_offset(876)
    }

    pub fn write_deck_fullness(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(876, &val)
    }

    /// Archmastery points.
    /// Python offset: 880, Primitive.float32
    pub fn archmastery_points(&self) -> Result<f32> {
        self.inner.read_value_from_offset(880)
    }

    pub fn write_archmastery_points(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(880, &val)
    }

    /// Max archmastery points.
    /// Python offset: 884, Primitive.float32
    pub fn max_archmastery_points(&self) -> Result<f32> {
        self.inner.read_value_from_offset(884)
    }

    pub fn write_max_archmastery_points(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(884, &val)
    }

    /// Archmastery school.
    /// Python offset: 888, Primitive.uint32
    pub fn archmastery_school(&self) -> Result<u32> {
        self.inner.read_value_from_offset(888)
    }

    pub fn write_archmastery_school(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(888, &val)
    }

    /// Archmastery flags.
    /// Python offset: 892, Primitive.uint32
    pub fn archmastery_flags(&self) -> Result<u32> {
        self.inner.read_value_from_offset(892)
    }

    pub fn write_archmastery_flags(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(892, &val)
    }
}

/// Legacy trait kept for backward compatibility with older code paths.
pub trait CombatParticipant: MemoryObject {
    fn owner_id_full(&self) -> u64 {
        self.read_value_from_offset(112).unwrap_or(0)
    }

    fn write_owner_id_full(&self, val: u64) {
        let _ = self.write_value_to_offset(112, &val);
    }

    fn health(&self) -> i32 {
        self.read_value_from_offset(244).unwrap_or(0)
    }

    fn write_health(&self, health: i32) {
        let _ = self.write_value_to_offset(244, &health);
    }

    fn max_health(&self) -> i32 {
        self.read_value_from_offset(248).unwrap_or(0)
    }

    fn write_max_health(&self, max_health: i32) {
        let _ = self.write_value_to_offset(248, &max_health);
    }
}
