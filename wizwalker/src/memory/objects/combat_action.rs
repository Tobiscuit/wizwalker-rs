use crate::memory::MemoryObject;

pub trait CombatAction: MemoryObject {
    fn spell_caster(&self) -> i32 {
        self.read_value_from_offset(72).unwrap_or(0)
    }

    fn write_spell_caster(&self, spell_caster: i32) {
        let _ = self.write_value_to_offset(72, &spell_caster);
    }

    // fn spell(&self) -> Option<DynamicSpell> {
    //     if let Ok(addr) = self.read_value_from_offset::<i64>(104) {
    //         if addr != 0 {
    //             return Some(DynamicSpell::new(addr));
    //         }
    //     }
    //     None
    // }

    fn spell_hits(&self) -> u8 {
        self.read_value_from_offset(120).unwrap_or(0)
    }

    fn write_spell_hits(&self, spell_hits: u8) {
        let _ = self.write_value_to_offset(120, &spell_hits);
    }

    fn interrupt(&self) -> bool {
        self.read_value_from_offset(121).unwrap_or(false)
    }

    fn write_interrupt(&self, interrupt: bool) {
        let _ = self.write_value_to_offset(121, &interrupt);
    }

    fn sigil_spell(&self) -> bool {
        self.read_value_from_offset(122).unwrap_or(false)
    }

    fn write_sigil_spell(&self, sigil_spell: bool) {
        let _ = self.write_value_to_offset(122, &sigil_spell);
    }

    fn show_cast(&self) -> bool {
        self.read_value_from_offset(123).unwrap_or(false)
    }

    fn write_show_cast(&self, show_cast: bool) {
        let _ = self.write_value_to_offset(123, &show_cast);
    }

    fn critical_hit_roll(&self) -> u8 {
        self.read_value_from_offset(124).unwrap_or(0)
    }

    fn write_critical_hit_roll(&self, critical_hit_roll: u8) {
        let _ = self.write_value_to_offset(124, &critical_hit_roll);
    }

    fn stun_resist_roll(&self) -> u8 {
        self.read_value_from_offset(125).unwrap_or(0)
    }

    fn write_stun_resist_roll(&self, stun_resist_roll: u8) {
        let _ = self.write_value_to_offset(125, &stun_resist_roll);
    }

    fn blocks_calculated(&self) -> bool {
        self.read_value_from_offset(160).unwrap_or(false)
    }

    fn write_blocks_calculated(&self, blocks_calculated: bool) {
        let _ = self.write_value_to_offset(160, &blocks_calculated);
    }

    // fn serialized_blocks(&self) -> String {
    //     self.read_string_from_offset(168).unwrap_or_default()
    // }

    // fn write_serialized_blocks(&self, serialized_blocks: String) {
    //     let _ = self.write_string_to_offset(168, &serialized_blocks);
    // }

    fn effect_chosen(&self) -> u32 {
        self.read_value_from_offset(220).unwrap_or(0)
    }

    fn write_effect_chosen(&self, effect_chosen: u32) {
        let _ = self.write_value_to_offset(220, &effect_chosen);
    }

    // fn string_key_message(&self) -> String {
    //     self.read_string_from_offset(224).unwrap_or_default()
    // }

    // fn write_string_key_message(&self, string_key_message: String) {
    //     let _ = self.write_string_to_offset(224, &string_key_message);
    // }

    // fn sound_file_name(&self) -> String {
    //     self.read_string_from_offset(256).unwrap_or_default()
    // }

    // fn write_sound_file_name(&self, sound_file_name: String) {
    //     let _ = self.write_string_to_offset(256, &sound_file_name);
    // }

    fn duration_modifier(&self) -> f32 {
        self.read_value_from_offset(288).unwrap_or(0.0)
    }

    fn write_duration_modifier(&self, duration_modifier: f32) {
        let _ = self.write_value_to_offset(288, &duration_modifier);
    }

    // fn serialized_targets_affected(&self) -> String {
    //     self.read_string_from_offset(296).unwrap_or_default()
    // }

    // fn write_serialized_targets_affected(&self, serialized_targets_affected: String) {
    //     let _ = self.write_string_to_offset(296, &serialized_targets_affected);
    // }

    fn target_subcircle_list(&self) -> i32 {
        self.read_value_from_offset(80).unwrap_or(0)
    }

    fn write_target_subcircle_list(&self, target_subcircle_list: i32) {
        let _ = self.write_value_to_offset(80, &target_subcircle_list);
    }

    fn pip_conversion_roll(&self) -> i32 {
        self.read_value_from_offset(128).unwrap_or(0)
    }

    fn write_pip_conversion_roll(&self, pip_conversion_roll: i32) {
        let _ = self.write_value_to_offset(128, &pip_conversion_roll);
    }

    fn random_spell_effect_per_target_rolls(&self) -> i32 {
        self.read_value_from_offset(136).unwrap_or(0)
    }

    fn write_random_spell_effect_per_target_rolls(&self, rolls: i32) {
        let _ = self.write_value_to_offset(136, &rolls);
    }

    fn handled_random_spell_per_target(&self) -> bool {
        self.read_value_from_offset(132).unwrap_or(false)
    }

    fn write_handled_random_spell_per_target(&self, handled: bool) {
        let _ = self.write_value_to_offset(132, &handled);
    }

    fn confused_target(&self) -> bool {
        self.read_value_from_offset(216).unwrap_or(false)
    }

    fn write_confused_target(&self, confused_target: bool) {
        let _ = self.write_value_to_offset(216, &confused_target);
    }

    fn force_spell(&self) -> bool {
        self.read_value_from_offset(344).unwrap_or(false)
    }

    fn write_force_spell(&self, force_spell: bool) {
        let _ = self.write_value_to_offset(344, &force_spell);
    }

    fn after_died(&self) -> bool {
        self.read_value_from_offset(217).unwrap_or(false)
    }

    fn write_after_died(&self, after_died: bool) {
        let _ = self.write_value_to_offset(217, &after_died);
    }

    fn delayed(&self) -> bool {
        self.read_value_from_offset(345).unwrap_or(false)
    }

    fn write_delayed(&self, delayed: bool) {
        let _ = self.write_value_to_offset(345, &delayed);
    }

    fn delayed_enchanted(&self) -> bool {
        self.read_value_from_offset(346).unwrap_or(false)
    }

    fn write_delayed_enchanted(&self, delayed_enchanted: bool) {
        let _ = self.write_value_to_offset(346, &delayed_enchanted);
    }

    fn pet_cast(&self) -> bool {
        self.read_value_from_offset(347).unwrap_or(false)
    }

    fn write_pet_cast(&self, pet_cast: bool) {
        let _ = self.write_value_to_offset(347, &pet_cast);
    }

    fn pet_casted(&self) -> bool {
        self.read_value_from_offset(348).unwrap_or(false)
    }

    fn write_pet_casted(&self, pet_casted: bool) {
        let _ = self.write_value_to_offset(348, &pet_casted);
    }

    fn pet_cast_target(&self) -> i32 {
        self.read_value_from_offset(352).unwrap_or(0)
    }

    fn write_pet_cast_target(&self, pet_cast_target: i32) {
        let _ = self.write_value_to_offset(352, &pet_cast_target);
    }
}
