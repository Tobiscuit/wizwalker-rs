use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};
use crate::memory::reader::MemoryReaderExt;
use crate::memory::objects::combat_participant::DynamicCombatParticipant;
use crate::types::XYZ;

// Re-export duel-related enums for convenience.
pub use super::enums::{DuelExecutionOrder, DuelPhase, SigilInitiativeSwitchMode};

/// A duel (combat encounter) in Wizard101.
///
/// All offsets are verified 1:1 against Python `wizwalker/memory/memory_objects/duel.py`.
#[derive(Clone)]
pub struct DynamicDuel {
    pub inner: DynamicMemoryObject,
}

impl DynamicDuel {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    // ── Identity ────────────────────────────────────────────────────

    /// Unique identifier for this duel instance.
    /// Python offset: 72, Primitive.uint64
    pub fn duel_id_full(&self) -> Result<u64> {
        self.inner.read_value_from_offset(72)
    }

    pub fn write_duel_id_full(&self, duel_id_full: u64) -> Result<()> {
        self.inner.write_value_to_offset(72, &duel_id_full)
    }

    // ── Participants ────────────────────────────────────────────────

    /// Read the list of combat participants in this duel.
    /// Python offset: 80, read_shared_vector
    pub fn participant_list(&self) -> Result<Vec<DynamicCombatParticipant>> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();

        let start_address: u64 = reader.read_typed((base_address + 80) as usize)?;
        let end_address: u64 = reader.read_typed((base_address + 88) as usize)?;

        if start_address == 0 || end_address == 0 || end_address <= start_address {
            return Ok(Vec::new());
        }

        // Shared pointers are 16 bytes each (8 bytes address + 8 bytes refcount)
        let size = (end_address - start_address) as usize;
        let element_count = size / 16;

        if element_count > 100 {
            return Ok(Vec::new()); // Sanity check
        }

        let shared_data = reader.read_bytes(start_address as usize, size)?;
        let mut participants = Vec::with_capacity(element_count);

        for i in 0..element_count {
            let offset = i * 16;
            if offset + 8 <= shared_data.len() {
                let addr = u64::from_le_bytes(
                    shared_data[offset..offset + 8].try_into().unwrap_or([0; 8]),
                );
                if addr != 0 {
                    if let Ok(inner) = DynamicMemoryObject::new(reader.clone(), addr) {
                        participants.push(DynamicCombatParticipant::new(inner));
                    }
                }
            }
        }
        Ok(participants)
    }

    // ── Turn Tracking ───────────────────────────────────────────────

    /// Current turn number within the duel's internal state machine.
    /// Python offset: 120, Primitive.uint32
    pub fn dynamic_turn(&self) -> Result<u32> {
        self.inner.read_value_from_offset(120)
    }

    pub fn write_dynamic_turn(&self, dynamic_turn: u32) -> Result<()> {
        self.inner.write_value_to_offset(120, &dynamic_turn)
    }

    /// Sub-circle within the current turn.
    /// Python offset: 124, Primitive.uint32
    pub fn dynamic_turn_subcircles(&self) -> Result<u32> {
        self.inner.read_value_from_offset(124)
    }

    pub fn write_dynamic_turn_subcircles(&self, value: u32) -> Result<()> {
        self.inner.write_value_to_offset(124, &value)
    }

    /// Counter tracking turn progression.
    /// Python offset: 128, Primitive.int32
    pub fn dynamic_turn_counter(&self) -> Result<i32> {
        self.inner.read_value_from_offset(128)
    }

    pub fn write_dynamic_turn_counter(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(128, &value)
    }

    // ── Combat Resolver ─────────────────────────────────────────────

    /// Pointer to the combat resolver for this duel.
    /// Python offset: 136, Primitive.int64
    /// Returns None if the pointer is null.
    pub fn combat_resolver_addr(&self) -> Result<Option<u64>> {
        let addr: u64 = self.inner.read_value_from_offset(136)?;
        if addr == 0 {
            Ok(None)
        } else {
            Ok(Some(addr))
        }
    }

    // ── Timers ──────────────────────────────────────────────────────

    /// Planning phase countdown timer (seconds remaining).
    /// Python offset: 144, Primitive.float32
    pub fn planning_timer(&self) -> Result<f32> {
        self.inner.read_value_from_offset(144)
    }

    pub fn write_planning_timer(&self, planning_timer: f32) -> Result<()> {
        self.inner.write_value_to_offset(144, &planning_timer)
    }

    // ── Position ────────────────────────────────────────────────────

    /// World position of the duel sigil.
    /// Python offset: 148, read_xyz
    pub fn position(&self) -> Result<XYZ> {
        self.inner.read_xyz(148)
    }

    pub fn write_position(&self, position: &XYZ) -> Result<()> {
        self.inner.write_xyz(148, position)
    }

    /// Rotation of the duel sigil (radians).
    /// Python offset: 160, Primitive.float32
    pub fn yaw(&self) -> Result<f32> {
        self.inner.read_value_from_offset(160)
    }

    pub fn write_yaw(&self, yaw: f32) -> Result<()> {
        self.inner.write_value_to_offset(160, &yaw)
    }

    // ── PvP / Battleground / Raid Flags ─────────────────────────────

    /// Whether this is a PvP duel.
    /// Python offset: 176, Primitive.bool
    pub fn is_pvp(&self) -> Result<bool> {
        self.inner.read_value_from_offset(176)
    }

    pub fn write_pvp(&self, pvp: bool) -> Result<()> {
        self.inner.write_value_to_offset(176, &pvp)
    }

    /// Whether this is a battleground duel.
    /// Python offset: 177, Primitive.bool
    pub fn is_battleground(&self) -> Result<bool> {
        self.inner.read_value_from_offset(177)
    }

    pub fn write_battleground(&self, battleground: bool) -> Result<()> {
        self.inner.write_value_to_offset(177, &battleground)
    }

    /// Whether this is a raid encounter.
    /// Python offset: 178, Primitive.bool
    pub fn is_raid(&self) -> Result<bool> {
        self.inner.read_value_from_offset(178)
    }

    pub fn write_raid(&self, raid: bool) -> Result<()> {
        self.inner.write_value_to_offset(178, &raid)
    }

    /// Whether the planning timer is disabled.
    /// Python offset: 179, Primitive.bool
    pub fn disable_timer(&self) -> Result<bool> {
        self.inner.read_value_from_offset(179)
    }

    pub fn write_disable_timer(&self, disable_timer: bool) -> Result<()> {
        self.inner.write_value_to_offset(179, &disable_timer)
    }

    /// Whether tutorial mode is active.
    /// Python offset: 180, Primitive.bool
    pub fn tutorial_mode(&self) -> Result<bool> {
        self.inner.read_value_from_offset(180)
    }

    pub fn write_tutorial_mode(&self, tutorial_mode: bool) -> Result<()> {
        self.inner.write_value_to_offset(180, &tutorial_mode)
    }

    // ── Initiative ──────────────────────────────────────────────────

    /// Which team acts first (0 or 1).
    /// Python offset: 184, Primitive.int32
    pub fn first_team_to_act(&self) -> Result<i32> {
        self.inner.read_value_from_offset(184)
    }

    pub fn write_first_team_to_act(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(184, &value)
    }

    /// Original first team to act (before any swaps).
    /// Python offset: 188, Primitive.int32
    pub fn original_first_team_to_act(&self) -> Result<i32> {
        self.inner.read_value_from_offset(188)
    }

    pub fn write_original_first_team_to_act(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(188, &value)
    }

    // ── Round / Phase ───────────────────────────────────────────────

    /// Current round number.
    /// Python offset: 192, Primitive.int32
    pub fn round_num(&self) -> Result<i32> {
        self.inner.read_value_from_offset(192)
    }

    pub fn write_round_num(&self, round_num: i32) -> Result<()> {
        self.inner.write_value_to_offset(192, &round_num)
    }

    /// Current duel phase (planning, execution, etc.).
    /// Python offset: 196, read_enum(DuelPhase)
    pub fn duel_phase(&self) -> Result<DuelPhase> {
        let val: i32 = self.inner.read_value_from_offset(196)?;
        match val {
            0 => Ok(DuelPhase::Starting),
            1 => Ok(DuelPhase::PrePlanning),
            2 => Ok(DuelPhase::Planning),
            3 => Ok(DuelPhase::PreExecution),
            4 => Ok(DuelPhase::Execution),
            5 => Ok(DuelPhase::Resolution),
            6 => Ok(DuelPhase::Victory),
            7 => Ok(DuelPhase::Ended),
            _ => Ok(DuelPhase::Max),
        }
    }

    pub fn write_duel_phase(&self, phase: DuelPhase) -> Result<()> {
        self.inner.write_value_to_offset(196, &(phase as i32))
    }

    /// Execution phase timer.
    /// Python offset: 200, Primitive.float32
    pub fn execution_phase_timer(&self) -> Result<f32> {
        self.inner.read_value_from_offset(200)
    }

    pub fn write_execution_phase_timer(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(200, &value)
    }

    // ── Unimplemented Complex Types ─────────────────────────────────
    // These exist in the game's memory layout but use complex types
    // that Python also left unimplemented.

    // TODO: offset 208, type: CombatAction
    // pub fn execution_phase_combat_actions(&self) -> Result<...> { ... }

    // TODO: offset 224, type: CombatAction
    // pub fn sigil_actions(&self) -> Result<...> { ... }

    // TODO: offset 264, type: SharedPointer<DuelModifier>
    // pub fn duel_modifier(&self) -> Result<...> { ... }

    // TODO: offset 280, type: SharedPointer<ShadowPipRule>
    // pub fn shadow_pip_rule(&self) -> Result<...> { ... }

    // TODO: offset 296, type: GameObjectAnimStateTracker
    // pub fn game_object_anim_state_tracker(&self) -> Result<...> { ... }

    // ── Initiative Switch ───────────────────────────────────────────

    /// How initiative switches between rounds.
    /// Python offset: 384, read_enum(SigilInitiativeSwitchMode)
    pub fn initiative_switch_mode(&self) -> Result<SigilInitiativeSwitchMode> {
        let val: i32 = self.inner.read_value_from_offset(384)?;
        match val {
            0 => Ok(SigilInitiativeSwitchMode::None),
            1 => Ok(SigilInitiativeSwitchMode::Reroll),
            2 => Ok(SigilInitiativeSwitchMode::Switch),
            _ => Ok(SigilInitiativeSwitchMode::None),
        }
    }

    pub fn write_initiative_switch_mode(&self, mode: SigilInitiativeSwitchMode) -> Result<()> {
        self.inner.write_value_to_offset(384, &(mode as i32))
    }

    /// Number of rounds between initiative switches.
    /// Python offset: 388, Primitive.int32
    pub fn initiative_switch_rounds(&self) -> Result<i32> {
        self.inner.read_value_from_offset(388)
    }

    pub fn write_initiative_switch_rounds(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(388, &value)
    }

    // ── Alternate Turn System ───────────────────────────────────────

    /// Alternate turn counter.
    /// Python offset: 456, Primitive.int32
    pub fn alt_turn_counter(&self) -> Result<i32> {
        self.inner.read_value_from_offset(456)
    }

    pub fn write_alt_turn_counter(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(456, &value)
    }

    // TODO: offset 464, type: SharedPointer<CombatRule>
    // pub fn combat_rules(&self) -> Result<...> { ... }

    // TODO: offset 480, type: SharedPointer<AlternateTurnsCombatRule>
    // pub fn alternate_turn_combat_rule(&self) -> Result<...> { ... }

    // TODO: offset 496, type: SharedPointer<GameEffectInfo>
    // pub fn game_effect_info(&self) -> Result<...> { ... }

    // TODO: offset 512, type: SharedPointer<GameEffectContainer>
    // pub fn stat_effects(&self) -> Result<...> { ... }

    // ── Execution Order ─────────────────────────────────────────────

    /// How combat actions are executed.
    /// Python offset: 528, read_enum(DuelExecutionOrder)
    pub fn execution_order(&self) -> Result<DuelExecutionOrder> {
        let val: i32 = self.inner.read_value_from_offset(528)?;
        match val {
            0 => Ok(DuelExecutionOrder::Sequential),
            1 => Ok(DuelExecutionOrder::Alternating),
            _ => Ok(DuelExecutionOrder::Sequential),
        }
    }

    pub fn write_execution_order(&self, order: DuelExecutionOrder) -> Result<()> {
        self.inner.write_value_to_offset(528, &(order as i32))
    }

    // ── Combat Modifiers ────────────────────────────────────────────

    /// Whether henchmen are prohibited.
    /// Python offset: 532, Primitive.bool
    pub fn no_henchmen(&self) -> Result<bool> {
        self.inner.read_value_from_offset(532)
    }

    pub fn write_no_henchmen(&self, value: bool) -> Result<()> {
        self.inner.write_value_to_offset(532, &value)
    }

    /// Distance at which non-combatants are hidden.
    /// Python offset: 536, Primitive.float32
    pub fn hide_noncombatant_distance(&self) -> Result<f32> {
        self.inner.read_value_from_offset(536)
    }

    pub fn write_hide_noncombatant_distance(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(536, &value)
    }

    /// Whether spell truncation is enabled.
    /// Python offset: 540, Primitive.bool
    pub fn spell_truncation(&self) -> Result<bool> {
        self.inner.read_value_from_offset(540)
    }

    pub fn write_spell_truncation(&self, value: bool) -> Result<()> {
        self.inner.write_value_to_offset(540, &value)
    }

    // ── Shadow Pip Mechanics ────────────────────────────────────────

    /// Shadow pip threshold factor.
    /// Python offset: 548, Primitive.float32
    pub fn shadow_threshold_factor(&self) -> Result<f32> {
        self.inner.read_value_from_offset(548)
    }

    pub fn write_shadow_threshold_factor(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(548, &value)
    }

    /// Shadow pip rating factor.
    /// Python offset: 552, Primitive.float32
    pub fn shadow_pip_rating_factor(&self) -> Result<f32> {
        self.inner.read_value_from_offset(552)
    }

    pub fn write_shadow_pip_rating_factor(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(552, &value)
    }

    /// Default shadow pip rating.
    /// Python offset: 556, Primitive.float32
    pub fn default_shadow_pip_rating(&self) -> Result<f32> {
        self.inner.read_value_from_offset(556)
    }

    pub fn write_default_shadow_pip_rating(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(556, &value)
    }

    /// Shadow pip threshold for team 0.
    /// Python offset: 560, Primitive.float32
    pub fn shadow_pip_threshold_team0(&self) -> Result<f32> {
        self.inner.read_value_from_offset(560)
    }

    pub fn write_shadow_pip_threshold_team0(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(560, &value)
    }

    /// Shadow pip threshold for team 1.
    /// Python offset: 564, Primitive.float32
    pub fn shadow_pip_threshold_team1(&self) -> Result<f32> {
        self.inner.read_value_from_offset(564)
    }

    pub fn write_shadow_pip_threshold_team1(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(564, &value)
    }

    // ── Duel Flag ───────────────────────────────────────────────────

    /// Whether this object is a duel (always true when valid).
    /// Python offset: 568, Primitive.bool
    pub fn is_duel(&self) -> Result<bool> {
        self.inner.read_value_from_offset(568)
    }

    pub fn write_duel(&self, value: bool) -> Result<()> {
        self.inner.write_value_to_offset(568, &value)
    }

    // TODO: offset 572, type: float32 (commented out in Python too)
    // pub fn max_archmastery(&self) -> Result<f32> {
    //     self.inner.read_value_from_offset(572)
    // }

    // ── Damage / Resist Scaling ─────────────────────────────────────

    /// Scalar damage multiplier for this duel.
    /// Python offset: 600, Primitive.float32
    pub fn scalar_damage(&self) -> Result<f32> {
        self.inner.read_value_from_offset(600)
    }

    pub fn write_scalar_damage(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(600, &value)
    }

    /// Scalar resist multiplier.
    /// Python offset: 604, Primitive.float32
    pub fn scalar_resist(&self) -> Result<f32> {
        self.inner.read_value_from_offset(604)
    }

    pub fn write_scalar_resist(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(604, &value)
    }

    /// Scalar pierce multiplier.
    /// Python offset: 608, Primitive.float32
    pub fn scalar_pierce(&self) -> Result<f32> {
        self.inner.read_value_from_offset(608)
    }

    pub fn write_scalar_pierce(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(608, &value)
    }

    /// Damage limit cap.
    /// Python offset: 612, Primitive.float32
    pub fn damage_limit(&self) -> Result<f32> {
        self.inner.read_value_from_offset(612)
    }

    pub fn write_damage_limit(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(612, &value)
    }

    /// Damage curve parameter k0.
    /// Python offset: 616, Primitive.float32
    pub fn damage_k0(&self) -> Result<f32> {
        self.inner.read_value_from_offset(616)
    }

    pub fn write_damage_k0(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(616, &value)
    }

    /// Damage curve parameter n0.
    /// Python offset: 620, Primitive.float32
    pub fn damage_n0(&self) -> Result<f32> {
        self.inner.read_value_from_offset(620)
    }

    pub fn write_damage_n0(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(620, &value)
    }

    /// Resist limit cap.
    /// Python offset: 624, Primitive.float32
    pub fn resist_limit(&self) -> Result<f32> {
        self.inner.read_value_from_offset(624)
    }

    pub fn write_resist_limit(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(624, &value)
    }

    /// Resist curve parameter k0.
    /// Python offset: 628, Primitive.float32
    pub fn resist_k0(&self) -> Result<f32> {
        self.inner.read_value_from_offset(628)
    }

    pub fn write_resist_k0(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(628, &value)
    }

    /// Resist curve parameter n0.
    /// Python offset: 632, Primitive.float32
    pub fn resist_n0(&self) -> Result<f32> {
        self.inner.read_value_from_offset(632)
    }

    pub fn write_resist_n0(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(632, &value)
    }

    // ── Party / Timer Settings ──────────────────────────────────────

    /// Whether this is a full party group.
    /// Python offset: 636, Primitive.bool
    pub fn full_party_group(&self) -> Result<bool> {
        self.inner.read_value_from_offset(636)
    }

    pub fn write_full_party_group(&self, value: bool) -> Result<()> {
        self.inner.write_value_to_offset(636, &value)
    }

    /// Whether this is a player-timed duel.
    /// Python offset: 637, Primitive.bool
    pub fn is_player_timed_duel(&self) -> Result<bool> {
        self.inner.read_value_from_offset(637)
    }

    pub fn write_player_timed_duel(&self, value: bool) -> Result<()> {
        self.inner.write_value_to_offset(637, &value)
    }

    /// Match timer (seconds).
    /// Python offset: 656, Primitive.float32
    pub fn match_timer(&self) -> Result<f32> {
        self.inner.read_value_from_offset(656)
    }

    pub fn write_match_timer(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(656, &value)
    }

    /// Bonus time per round (seconds).
    /// Python offset: 660, Primitive.int32
    pub fn bonus_time(&self) -> Result<i32> {
        self.inner.read_value_from_offset(660)
    }

    pub fn write_bonus_time(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(660, &value)
    }

    /// Time penalty for passing (seconds).
    /// Python offset: 664, Primitive.int32
    pub fn pass_penalty(&self) -> Result<i32> {
        self.inner.read_value_from_offset(664)
    }

    pub fn write_pass_penalty(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(664, &value)
    }

    /// Yellow warning time threshold (seconds).
    /// Python offset: 668, Primitive.int32
    pub fn yellow_time(&self) -> Result<i32> {
        self.inner.read_value_from_offset(668)
    }

    pub fn write_yellow_time(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(668, &value)
    }

    /// Red warning time threshold (seconds).
    /// Python offset: 672, Primitive.int32
    pub fn red_time(&self) -> Result<i32> {
        self.inner.read_value_from_offset(672)
    }

    pub fn write_red_time(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(672, &value)
    }

    /// Minimum turn time (seconds).
    /// Python offset: 676, Primitive.int32
    pub fn min_turn_time(&self) -> Result<i32> {
        self.inner.read_value_from_offset(676)
    }

    pub fn write_min_turn_time(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(676, &value)
    }
}

impl MemoryObject for DynamicDuel {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }
    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}
