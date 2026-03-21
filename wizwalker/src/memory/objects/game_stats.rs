use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};

pub trait GameStats: MemoryObject {
    fn max_hitpoints(&self) -> Result<i32> {
        Ok(self.base_hitpoints()? + self.bonus_hitpoints()?)
    }

    fn max_mana(&self) -> Result<i32> {
        Ok(self.base_mana()? + self.bonus_mana()?)
    }

    fn base_hitpoints(&self) -> Result<i32> {
        self.read_value_from_offset(80)
    }

    fn write_base_hitpoints(&self, base_hitpoints: i32) -> Result<()> {
        self.write_value_to_offset(80, &base_hitpoints)
    }

    fn base_mana(&self) -> Result<i32> {
        self.read_value_from_offset(84)
    }

    fn write_base_mana(&self, base_mana: i32) -> Result<()> {
        self.write_value_to_offset(84, &base_mana)
    }

    fn base_gold_pouch(&self) -> Result<i32> {
        self.read_value_from_offset(88)
    }

    fn write_base_gold_pouch(&self, base_gold_pouch: i32) -> Result<()> {
        self.write_value_to_offset(88, &base_gold_pouch)
    }

    fn base_event_currency1_pouch(&self) -> Result<i32> {
        self.read_value_from_offset(92)
    }

    fn write_base_event_currency1_pouch(&self, base_event_currency1_pouch: i32) -> Result<()> {
        self.write_value_to_offset(92, &base_event_currency1_pouch)
    }

    fn base_event_currency2_pouch(&self) -> Result<i32> {
        self.read_value_from_offset(96)
    }

    fn write_base_event_currency2_pouch(&self, base_event_currency2_pouch: i32) -> Result<()> {
        self.write_value_to_offset(96, &base_event_currency2_pouch)
    }

    fn base_pvp_currency_pouch(&self) -> Result<i32> {
        self.read_value_from_offset(100)
    }

    fn write_base_pvp_currency_pouch(&self, base_pvp_currency_pouch: i32) -> Result<()> {
        self.write_value_to_offset(100, &base_pvp_currency_pouch)
    }

    fn base_pvp_tourney_currency_pouch(&self) -> Result<i32> {
        self.read_value_from_offset(104)
    }

    fn write_base_pvp_tourney_currency_pouch(&self, base_pvp_tourney_currency_pouch: i32) -> Result<()> {
        self.write_value_to_offset(104, &base_pvp_tourney_currency_pouch)
    }

    fn energy_max(&self) -> Result<i32> {
        self.read_value_from_offset(108)
    }

    fn write_energy_max(&self, energy_max: i32) -> Result<()> {
        self.write_value_to_offset(108, &energy_max)
    }

    fn current_hitpoints(&self) -> Result<i32> {
        self.read_value_from_offset(112)
    }

    fn write_current_hitpoints(&self, current_hitpoints: i32) -> Result<()> {
        self.write_value_to_offset(112, &current_hitpoints)
    }

    fn current_gold(&self) -> Result<i32> {
        self.read_value_from_offset(116)
    }

    fn write_current_gold(&self, current_gold: i32) -> Result<()> {
        self.write_value_to_offset(116, &current_gold)
    }

    fn current_event_currency1(&self) -> Result<i32> {
        self.read_value_from_offset(120)
    }

    fn write_current_event_currency1(&self, current_event_currency1: i32) -> Result<()> {
        self.write_value_to_offset(120, &current_event_currency1)
    }

    fn current_event_currency2(&self) -> Result<i32> {
        self.read_value_from_offset(124)
    }

    fn write_current_event_currency2(&self, current_event_currency2: i32) -> Result<()> {
        self.write_value_to_offset(124, &current_event_currency2)
    }

    fn current_pvp_currency(&self) -> Result<i32> {
        self.read_value_from_offset(128)
    }

    fn write_current_pvp_currency(&self, current_pvp_currency: i32) -> Result<()> {
        self.write_value_to_offset(128, &current_pvp_currency)
    }

    fn current_pvp_tourney_currency(&self) -> Result<i32> {
        self.read_value_from_offset(132)
    }

    fn write_current_pvp_tourney_currency(&self, current_pvp_tourney_currency: i32) -> Result<()> {
        self.write_value_to_offset(132, &current_pvp_tourney_currency)
    }

    fn current_mana(&self) -> Result<i32> {
        self.read_value_from_offset(136)
    }

    fn write_current_mana(&self, current_mana: i32) -> Result<()> {
        self.write_value_to_offset(136, &current_mana)
    }

    fn current_arena_points(&self) -> Result<i32> {
        self.read_value_from_offset(140)
    }

    fn write_current_arena_points(&self, current_arena_points: i32) -> Result<()> {
        self.write_value_to_offset(140, &current_arena_points)
    }

    fn spell_charge_base(&self) -> Result<Vec<i32>> {
        self.read_dynamic_vector(144)
    }

    fn potion_max(&self) -> Result<f32> {
        self.read_value_from_offset(168)
    }

    fn write_potion_max(&self, potion_max: f32) -> Result<()> {
        self.write_value_to_offset(168, &potion_max)
    }

    fn potion_charge(&self) -> Result<f32> {
        self.read_value_from_offset(172)
    }

    fn write_potion_charge(&self, potion_charge: f32) -> Result<()> {
        self.write_value_to_offset(172, &potion_charge)
    }

    // TODO: offset 176, type: SharedPointer<Ladder>
    // fn arena_ladder(&self) -> Result<...> { ... }

    // TODO: offset 192, type: SharedPointer<Ladder>
    // fn derby_ladder(&self) -> Result<...> { ... }

    // TODO: offset 208, type: SharedPointer<Ladder>
    // fn bracket_ladder(&self) -> Result<...> { ... }

    fn bonus_hitpoints(&self) -> Result<i32> {
        self.read_value_from_offset(224)
    }

    fn write_bonus_hitpoints(&self, bonus_hitpoints: i32) -> Result<()> {
        self.write_value_to_offset(224, &bonus_hitpoints)
    }

    fn bonus_mana(&self) -> Result<i32> {
        self.read_value_from_offset(228)
    }

    fn write_bonus_mana(&self, bonus_mana: i32) -> Result<()> {
        self.write_value_to_offset(228, &bonus_mana)
    }

    fn bonus_energy(&self) -> Result<i32> {
        self.read_value_from_offset(244)
    }

    fn write_bonus_energy(&self, bonus_energy: i32) -> Result<()> {
        self.write_value_to_offset(244, &bonus_energy)
    }

    fn critical_hit_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(248)
    }

    fn write_critical_hit_percent_all(&self, critical_hit_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(248, &critical_hit_percent_all)
    }

    fn block_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(252)
    }

    fn write_block_percent_all(&self, block_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(252, &block_percent_all)
    }

    fn critical_hit_rating_all(&self) -> Result<f32> {
        self.read_value_from_offset(256)
    }

    fn write_critical_hit_rating_all(&self, critical_hit_rating_all: f32) -> Result<()> {
        self.write_value_to_offset(256, &critical_hit_rating_all)
    }

    fn block_rating_all(&self) -> Result<f32> {
        self.read_value_from_offset(260)
    }

    fn write_block_rating_all(&self, block_rating_all: f32) -> Result<()> {
        self.write_value_to_offset(260, &block_rating_all)
    }

    fn reference_level(&self) -> Result<i32> {
        self.read_value_from_offset(324)
    }

    fn write_reference_level(&self, reference_level: i32) -> Result<()> {
        self.write_value_to_offset(324, &reference_level)
    }

    fn highest_character_level_on_account(&self) -> Result<i32> {
        self.read_value_from_offset(336)
    }

    fn write_highest_character_level_on_account(&self, highest_character_level_on_account: i32) -> Result<()> {
        self.write_value_to_offset(336, &highest_character_level_on_account)
    }

    fn pet_act_chance(&self) -> Result<i32> {
        self.read_value_from_offset(344)
    }

    fn write_pet_act_chance(&self, pet_act_chance: i32) -> Result<()> {
        self.write_value_to_offset(344, &pet_act_chance)
    }

    fn dmg_bonus_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(352)
    }

    fn dmg_bonus_flat(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(376)
    }

    fn acc_bonus_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(400)
    }

    fn ap_bonus_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(424)
    }

    fn dmg_reduce_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(448)
    }

    fn dmg_reduce_flat(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(472)
    }

    fn acc_reduce_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(496)
    }

    fn heal_bonus_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(520)
    }

    fn heal_inc_bonus_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(544)
    }

    fn spell_charge_bonus(&self) -> Result<Vec<i32>> {
        self.read_dynamic_vector(592)
    }

    fn dmg_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(712)
    }

    fn write_dmg_bonus_percent_all(&self, dmg_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(712, &dmg_bonus_percent_all)
    }

    fn dmg_bonus_flat_all(&self) -> Result<f32> {
        self.read_value_from_offset(716)
    }

    fn write_dmg_bonus_flat_all(&self, dmg_bonus_flat_all: f32) -> Result<()> {
        self.write_value_to_offset(716, &dmg_bonus_flat_all)
    }

    fn acc_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(720)
    }

    fn write_acc_bonus_percent_all(&self, acc_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(720, &acc_bonus_percent_all)
    }

    fn ap_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(724)
    }

    fn write_ap_bonus_percent_all(&self, ap_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(724, &ap_bonus_percent_all)
    }

    fn dmg_reduce_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(728)
    }

    fn write_dmg_reduce_percent_all(&self, dmg_reduce_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(728, &dmg_reduce_percent_all)
    }

    fn dmg_reduce_flat_all(&self) -> Result<f32> {
        self.read_value_from_offset(732)
    }

    fn write_dmg_reduce_flat_all(&self, dmg_reduce_flat_all: f32) -> Result<()> {
        self.write_value_to_offset(732, &dmg_reduce_flat_all)
    }

    fn acc_reduce_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(736)
    }

    fn write_acc_reduce_percent_all(&self, acc_reduce_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(736, &acc_reduce_percent_all)
    }

    fn heal_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(740)
    }

    fn write_heal_bonus_percent_all(&self, heal_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(740, &heal_bonus_percent_all)
    }

    fn heal_inc_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(744)
    }

    fn write_heal_inc_bonus_percent_all(&self, heal_inc_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(744, &heal_inc_bonus_percent_all)
    }

    fn spell_charge_bonus_all(&self) -> Result<i32> {
        self.read_value_from_offset(752)
    }

    fn write_spell_charge_bonus_all(&self, spell_charge_bonus_all: i32) -> Result<()> {
        self.write_value_to_offset(752, &spell_charge_bonus_all)
    }

    fn power_pip_base(&self) -> Result<f32> {
        self.read_value_from_offset(756)
    }

    fn write_power_pip_base(&self, power_pip_base: f32) -> Result<()> {
        self.write_value_to_offset(756, &power_pip_base)
    }

    fn power_pip_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(792)
    }

    fn write_power_pip_bonus_percent_all(&self, power_pip_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(792, &power_pip_bonus_percent_all)
    }

    fn xp_percent_increase(&self) -> Result<f32> {
        self.read_value_from_offset(800)
    }

    fn write_xp_percent_increase(&self, xp_percent_increase: f32) -> Result<()> {
        self.write_value_to_offset(800, &xp_percent_increase)
    }

    fn pip_conversion_rating_per_school(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(264)
    }

    fn pip_conversion_rating_all(&self) -> Result<f32> {
        self.read_value_from_offset(288)
    }

    fn write_pip_conversion_rating_all(&self, pip_conversion_rating_all: f32) -> Result<()> {
        self.write_value_to_offset(288, &pip_conversion_rating_all)
    }

    fn pip_conversion_percent_per_school(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(296)
    }

    fn pip_conversion_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(320)
    }

    fn write_pip_conversion_percent_all(&self, pip_conversion_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(320, &pip_conversion_percent_all)
    }

    fn school_id(&self) -> Result<u32> {
        self.read_value_from_offset(328)
    }

    fn write_school_id(&self, school_id: u32) -> Result<()> {
        self.write_value_to_offset(328, &school_id)
    }

    fn level_scaled(&self) -> Result<i32> {
        self.read_value_from_offset(332)
    }

    fn write_level_scaled(&self, level_scaled: i32) -> Result<()> {
        self.write_value_to_offset(332, &level_scaled)
    }

    fn highest_character_world_on_account(&self) -> Result<i32> {
        self.read_value_from_offset(340)
    }

    fn write_highest_character_world_on_account(&self, highest_character_world_on_account: i32) -> Result<()> {
        self.write_value_to_offset(340, &highest_character_world_on_account)
    }

    fn fishing_luck_bonus_percent(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(568)
    }

    fn critical_hit_percent_by_school(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(616)
    }

    fn block_percent_by_school(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(640)
    }

    fn critical_hit_rating_by_school(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(664)
    }

    fn block_rating_by_school(&self) -> Result<Vec<f32>> {
        self.read_dynamic_vector(688)
    }

    fn fishing_luck_bonus_percent_all(&self) -> Result<f32> {
        self.read_value_from_offset(748)
    }

    fn write_fishing_luck_bonus_percent_all(&self, fishing_luck_bonus_percent_all: f32) -> Result<()> {
        self.write_value_to_offset(748, &fishing_luck_bonus_percent_all)
    }

    fn pip_conversion_base_all_schools(&self) -> Result<i32> {
        self.read_value_from_offset(760)
    }

    fn write_pip_conversion_base_all_schools(&self, pip_conversion_base_all_schools: i32) -> Result<()> {
        self.write_value_to_offset(760, &pip_conversion_base_all_schools)
    }

    fn pip_conversion_base_per_school(&self) -> Result<Vec<i32>> {
        self.read_dynamic_vector(768)
    }

    fn shadow_pip_bonus_percent(&self) -> Result<f32> {
        self.read_value_from_offset(796)
    }

    fn write_shadow_pip_bonus_percent(&self, shadow_pip_bonus_percent: f32) -> Result<()> {
        self.write_value_to_offset(796, &shadow_pip_bonus_percent)
    }

    fn wisp_bonus_percent(&self) -> Result<f32> {
        self.read_value_from_offset(824)
    }

    fn write_wisp_bonus_percent(&self, wisp_bonus_percent: f32) -> Result<()> {
        self.write_value_to_offset(824, &wisp_bonus_percent)
    }

    fn balance_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(832)
    }

    fn write_balance_mastery(&self, balance_mastery: i32) -> Result<()> {
        self.write_value_to_offset(832, &balance_mastery)
    }

    fn death_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(836)
    }

    fn write_death_mastery(&self, death_mastery: i32) -> Result<()> {
        self.write_value_to_offset(836, &death_mastery)
    }

    fn fire_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(840)
    }

    fn write_fire_mastery(&self, fire_mastery: i32) -> Result<()> {
        self.write_value_to_offset(840, &fire_mastery)
    }

    fn ice_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(844)
    }

    fn write_ice_mastery(&self, ice_mastery: i32) -> Result<()> {
        self.write_value_to_offset(844, &ice_mastery)
    }

    fn life_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(848)
    }

    fn write_life_mastery(&self, life_mastery: i32) -> Result<()> {
        self.write_value_to_offset(848, &life_mastery)
    }

    fn myth_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(852)
    }

    fn write_myth_mastery(&self, myth_mastery: i32) -> Result<()> {
        self.write_value_to_offset(852, &myth_mastery)
    }

    fn storm_mastery(&self) -> Result<i32> {
        self.read_value_from_offset(856)
    }

    fn write_storm_mastery(&self, storm_mastery: i32) -> Result<()> {
        self.write_value_to_offset(856, &storm_mastery)
    }

    fn maximum_number_of_islands(&self) -> Result<i32> {
        self.read_value_from_offset(860)
    }

    fn write_maximum_number_of_islands(&self, maximum_number_of_islands: i32) -> Result<()> {
        self.write_value_to_offset(860, &maximum_number_of_islands)
    }

    fn gardening_level(&self) -> Result<u8> {
        self.read_value_from_offset(864)
    }

    fn write_gardening_level(&self, gardening_level: u8) -> Result<()> {
        self.write_value_to_offset(864, &gardening_level)
    }

    fn gardening_xp(&self) -> Result<i32> {
        self.read_value_from_offset(868)
    }

    fn write_gardening_xp(&self, gardening_xp: i32) -> Result<()> {
        self.write_value_to_offset(868, &gardening_xp)
    }

    fn invisible_to_friends(&self) -> Result<bool> {
        self.read_value_from_offset(872)
    }

    fn write_invisible_to_friends(&self, invisible_to_friends: bool) -> Result<()> {
        self.write_value_to_offset(872, &invisible_to_friends)
    }

    fn show_item_lock(&self) -> Result<bool> {
        self.read_value_from_offset(873)
    }

    fn write_show_item_lock(&self, show_item_lock: bool) -> Result<()> {
        self.write_value_to_offset(873, &show_item_lock)
    }

    fn quest_finder_enabled(&self) -> Result<bool> {
        self.read_value_from_offset(874)
    }

    fn write_quest_finder_enabled(&self, quest_finder_enabled: bool) -> Result<()> {
        self.write_value_to_offset(874, &quest_finder_enabled)
    }

    fn buddy_list_limit(&self) -> Result<i32> {
        self.read_value_from_offset(876)
    }

    fn write_buddy_list_limit(&self, buddy_list_limit: i32) -> Result<()> {
        self.write_value_to_offset(876, &buddy_list_limit)
    }

    fn stun_resistance_percent(&self) -> Result<f32> {
        self.read_value_from_offset(880)
    }

    fn write_stun_resistance_percent(&self, stun_resistance_percent: f32) -> Result<()> {
        self.write_value_to_offset(880, &stun_resistance_percent)
    }

    fn dont_allow_friend_finder_codes(&self) -> Result<bool> {
        self.read_value_from_offset(884)
    }

    fn write_dont_allow_friend_finder_codes(&self, dont_allow_friend_finder_codes: bool) -> Result<()> {
        self.write_value_to_offset(884, &dont_allow_friend_finder_codes)
    }

    fn shadow_pip_max(&self) -> Result<i32> {
        self.read_value_from_offset(888)
    }

    fn write_shadow_pip_max(&self, shadow_pip_max: i32) -> Result<()> {
        self.write_value_to_offset(888, &shadow_pip_max)
    }

    fn shadow_magic_unlocked(&self) -> Result<bool> {
        self.read_value_from_offset(892)
    }

    fn write_shadow_magic_unlocked(&self, shadow_magic_unlocked: bool) -> Result<()> {
        self.write_value_to_offset(892, &shadow_magic_unlocked)
    }

    fn fishing_level(&self) -> Result<u8> {
        self.read_value_from_offset(893)
    }

    fn write_fishing_level(&self, fishing_level: u8) -> Result<()> {
        self.write_value_to_offset(893, &fishing_level)
    }

    fn fishing_xp(&self) -> Result<i32> {
        self.read_value_from_offset(896)
    }

    fn write_fishing_xp(&self, fishing_xp: i32) -> Result<()> {
        self.write_value_to_offset(896, &fishing_xp)
    }

    fn subscriber_benefit_flags(&self) -> Result<u32> {
        self.read_value_from_offset(900)
    }

    fn write_subscriber_benefit_flags(&self, subscriber_benefit_flags: u32) -> Result<()> {
        self.write_value_to_offset(900, &subscriber_benefit_flags)
    }

    fn elixir_benefit_flags(&self) -> Result<u32> {
        self.read_value_from_offset(904)
    }

    fn write_elixir_benefit_flags(&self, elixir_benefit_flags: u32) -> Result<()> {
        self.write_value_to_offset(904, &elixir_benefit_flags)
    }

    fn monster_magic_level(&self) -> Result<u8> {
        self.read_value_from_offset(908)
    }

    fn write_monster_magic_level(&self, monster_magic_level: u8) -> Result<()> {
        self.write_value_to_offset(908, &monster_magic_level)
    }

    fn monster_magic_xp(&self) -> Result<i32> {
        self.read_value_from_offset(912)
    }

    fn write_monster_magic_xp(&self, monster_magic_xp: i32) -> Result<()> {
        self.write_value_to_offset(912, &monster_magic_xp)
    }

    fn player_chat_channel_is_public(&self) -> Result<bool> {
        self.read_value_from_offset(916)
    }

    fn write_player_chat_channel_is_public(&self, player_chat_channel_is_public: bool) -> Result<()> {
        self.write_value_to_offset(916, &player_chat_channel_is_public)
    }

    fn extra_inventory_space(&self) -> Result<i32> {
        self.read_value_from_offset(920)
    }

    fn write_extra_inventory_space(&self, extra_inventory_space: i32) -> Result<()> {
        self.write_value_to_offset(920, &extra_inventory_space)
    }

    fn remember_last_realm(&self) -> Result<bool> {
        self.read_value_from_offset(924)
    }

    fn write_remember_last_realm(&self, remember_last_realm: bool) -> Result<()> {
        self.write_value_to_offset(924, &remember_last_realm)
    }

    fn new_spellbook_layout_warning(&self) -> Result<bool> {
        self.read_value_from_offset(925)
    }

    fn write_new_spellbook_layout_warning(&self, new_spellbook_layout_warning: bool) -> Result<()> {
        self.write_value_to_offset(925, &new_spellbook_layout_warning)
    }

    fn purchased_custom_emotes1(&self) -> Result<u32> {
        self.read_value_from_offset(928)
    }

    fn write_purchased_custom_emotes1(&self, purchased_custom_emotes1: u32) -> Result<()> {
        self.write_value_to_offset(928, &purchased_custom_emotes1)
    }

    fn purchased_custom_teleport_effects1(&self) -> Result<u32> {
        self.read_value_from_offset(932)
    }

    fn write_purchased_custom_teleport_effects1(&self, purchased_custom_teleport_effects1: u32) -> Result<()> {
        self.write_value_to_offset(932, &purchased_custom_teleport_effects1)
    }

    fn equipped_teleport_effect(&self) -> Result<u32> {
        self.read_value_from_offset(936)
    }

    fn write_equipped_teleport_effect(&self, equipped_teleport_effect: u32) -> Result<()> {
        self.write_value_to_offset(936, &equipped_teleport_effect)
    }

    fn purchased_custom_emotes2(&self) -> Result<u32> {
        self.read_value_from_offset(940)
    }

    fn write_purchased_custom_emotes2(&self, purchased_custom_emotes2: u32) -> Result<()> {
        self.write_value_to_offset(940, &purchased_custom_emotes2)
    }

    fn purchased_custom_teleport_effects2(&self) -> Result<u32> {
        self.read_value_from_offset(944)
    }

    fn write_purchased_custom_teleport_effects2(&self, purchased_custom_teleport_effects2: u32) -> Result<()> {
        self.write_value_to_offset(944, &purchased_custom_teleport_effects2)
    }

    fn purchased_custom_emotes3(&self) -> Result<u32> {
        self.read_value_from_offset(948)
    }

    fn write_purchased_custom_emotes3(&self, purchased_custom_emotes3: u32) -> Result<()> {
        self.write_value_to_offset(948, &purchased_custom_emotes3)
    }

    fn purchased_custom_teleport_effects3(&self) -> Result<u32> {
        self.read_value_from_offset(952)
    }

    fn write_purchased_custom_teleport_effects3(&self, purchased_custom_teleport_effects3: u32) -> Result<()> {
        self.write_value_to_offset(952, &purchased_custom_teleport_effects3)
    }

    fn highest_world1_id(&self) -> Result<u32> {
        self.read_value_from_offset(956)
    }

    fn write_highest_world1_id(&self, highest_world1_id: u32) -> Result<()> {
        self.write_value_to_offset(956, &highest_world1_id)
    }

    fn highest_world2_id(&self) -> Result<u32> {
        self.read_value_from_offset(960)
    }

    fn write_highest_world2_id(&self, highest_world2_id: u32) -> Result<()> {
        self.write_value_to_offset(960, &highest_world2_id)
    }

    fn active_class_projects_list(&self) -> Result<u32> {
        self.read_value_from_offset(968)
    }

    fn write_active_class_projects_list(&self, active_class_projects_list: u32) -> Result<()> {
        self.write_value_to_offset(968, &active_class_projects_list)
    }

    fn disabled_item_slot_ids(&self) -> Result<u32> {
        self.read_value_from_offset(984)
    }

    fn write_disabled_item_slot_ids(&self, disabled_item_slot_ids: u32) -> Result<()> {
        self.write_value_to_offset(984, &disabled_item_slot_ids)
    }

    fn adventure_power_cooldown_time(&self) -> Result<u32> {
        self.read_value_from_offset(1000)
    }

    fn write_adventure_power_cooldown_time(&self, adventure_power_cooldown_time: u32) -> Result<()> {
        self.write_value_to_offset(1000, &adventure_power_cooldown_time)
    }

    fn shadow_pip_rating(&self) -> Result<f32> {
        self.read_value_from_offset(1004)
    }

    fn write_shadow_pip_rating(&self, shadow_pip_rating: f32) -> Result<()> {
        self.write_value_to_offset(1004, &shadow_pip_rating)
    }

    fn bonus_shadow_pip_rating(&self) -> Result<f32> {
        self.read_value_from_offset(1008)
    }

    fn write_bonus_shadow_pip_rating(&self, bonus_shadow_pip_rating: f32) -> Result<()> {
        self.write_value_to_offset(1008, &bonus_shadow_pip_rating)
    }

    fn shadow_pip_rate_accumulated(&self) -> Result<f32> {
        self.read_value_from_offset(1012)
    }

    fn write_shadow_pip_rate_accumulated(&self, shadow_pip_rate_accumulated: f32) -> Result<()> {
        self.write_value_to_offset(1012, &shadow_pip_rate_accumulated)
    }

    fn shadow_pip_rate_threshold(&self) -> Result<f32> {
        self.read_value_from_offset(1016)
    }

    fn write_shadow_pip_rate_threshold(&self, shadow_pip_rate_threshold: f32) -> Result<()> {
        self.write_value_to_offset(1016, &shadow_pip_rate_threshold)
    }

    fn shadow_pip_rate_percentage(&self) -> Result<i32> {
        self.read_value_from_offset(1020)
    }

    fn write_shadow_pip_rate_percentage(&self, shadow_pip_rate_percentage: i32) -> Result<()> {
        self.write_value_to_offset(1020, &shadow_pip_rate_percentage)
    }

    fn friendly_player(&self) -> Result<bool> {
        self.read_value_from_offset(1024)
    }

    fn write_friendly_player(&self, friendly_player: bool) -> Result<()> {
        self.write_value_to_offset(1024, &friendly_player)
    }

    fn emoji_skin_tone(&self) -> Result<i32> {
        self.read_value_from_offset(1028)
    }

    fn write_emoji_skin_tone(&self, emoji_skin_tone: i32) -> Result<()> {
        self.write_value_to_offset(1028, &emoji_skin_tone)
    }

    fn show_pvp_option(&self) -> Result<u32> {
        self.read_value_from_offset(1032)
    }

    fn write_show_pvp_option(&self, show_pvp_option: u32) -> Result<()> {
        self.write_value_to_offset(1032, &show_pvp_option)
    }

    fn favorite_slot(&self) -> Result<i32> {
        self.read_value_from_offset(1036)
    }

    fn write_favorite_slot(&self, favorite_slot: i32) -> Result<()> {
        self.write_value_to_offset(1036, &favorite_slot)
    }

    fn cantrip_level(&self) -> Result<u8> {
        self.read_value_from_offset(1040)
    }

    fn write_cantrip_level(&self, cantrip_level: u8) -> Result<()> {
        self.write_value_to_offset(1040, &cantrip_level)
    }

    fn cantrip_xp(&self) -> Result<i32> {
        self.read_value_from_offset(1044)
    }

    fn write_cantrip_xp(&self, cantrip_xp: i32) -> Result<()> {
        self.write_value_to_offset(1044, &cantrip_xp)
    }

    fn archmastery_base(&self) -> Result<f32> {
        self.read_value_from_offset(1048)
    }

    fn write_archmastery_base(&self, archmastery_base: f32) -> Result<()> {
        self.write_value_to_offset(1048, &archmastery_base)
    }

    fn archmastery_bonus_flat(&self) -> Result<f32> {
        self.read_value_from_offset(1052)
    }

    fn write_archmastery_bonus_flat(&self, archmastery_bonus_flat: f32) -> Result<()> {
        self.write_value_to_offset(1052, &archmastery_bonus_flat)
    }

    fn archmastery_bonus_percentage(&self) -> Result<f32> {
        self.read_value_from_offset(1056)
    }

    fn write_archmastery_bonus_percentage(&self, archmastery_bonus_percentage: f32) -> Result<()> {
        self.write_value_to_offset(1056, &archmastery_bonus_percentage)
    }

    fn current_zone_name(&self) -> Result<String> {
        self.read_string_from_offset(1064)
    }

    fn write_current_zone_name(&self, current_zone_name: &str) -> Result<()> {
        self.write_string_to_offset(1064, current_zone_name)
    }

    fn mail_sent_today(&self) -> Result<u8> {
        self.read_value_from_offset(1096)
    }

    fn write_mail_sent_today(&self, mail_sent_today: u8) -> Result<()> {
        self.write_value_to_offset(1096, &mail_sent_today)
    }

    fn secondary_school(&self) -> Result<i32> {
        self.read_value_from_offset(1100)
    }

    fn write_secondary_school(&self, secondary_school: i32) -> Result<()> {
        self.write_value_to_offset(1100, &secondary_school)
    }

    fn disable_cross_play(&self) -> Result<bool> {
        self.read_value_from_offset(1104)
    }

    fn write_disable_cross_play(&self, disable_cross_play: bool) -> Result<()> {
        self.write_value_to_offset(1104, &disable_cross_play)
    }

    fn photo_filters(&self) -> Result<u32> {
        self.read_value_from_offset(1108)
    }

    fn write_photo_filters(&self, photo_filters: u32) -> Result<()> {
        self.write_value_to_offset(1108, &photo_filters)
    }

}

pub struct CurrentGameStats {
    _inner: DynamicMemoryObject,
}

impl CurrentGameStats {
    pub fn new(_reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>) -> Result<Self> {
        // Find base address dynamically as per python
        // but since we aren't provided with hook_handler, just return dummy or require base addr
        // Wait, Python does: return await self.hook_handler.read_current_player_stat_base()
        // I will just implement new() that takes base_address
        Err(crate::errors::WizWalkerError::Other("Not implemented".into()))
    }
}

pub struct DynamicGameStats {
    pub inner: DynamicMemoryObject,
}

impl DynamicGameStats {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }
}

impl MemoryObject for DynamicGameStats {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }
    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl GameStats for DynamicGameStats {}
