use crate::memory::MemoryObject;

pub struct DynamicSpellTemplate {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicSpellTemplate {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

pub struct DynamicSpellRank {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicSpellRank {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

pub struct DynamicSpellEffect {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicSpellEffect {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

#[async_trait::async_trait]
pub trait Spell: MemoryObject {
    async fn template_id(&self) -> u32 {
        self.read_value_from_offset(128).await.unwrap_or(0)
    }

    async fn write_template_id(&self, template_id: u32) {
        let _ = self.write_value_to_offset(128, &template_id).await;
    }

    async fn spell_template(&self) -> Option<DynamicSpellTemplate> {
        if let Ok(addr) = self.read_value_from_offset::<i64>(120).await {
            if addr != 0 {
                return Some(DynamicSpellTemplate::new(self.hook_handler(), addr));
            }
        }
        None
    }

    async fn enchantment(&self) -> u32 {
        self.read_value_from_offset(80).await.unwrap_or(0)
    }

    async fn write_enchantment(&self, enchantment: u32) {
        let _ = self.write_value_to_offset(80, &enchantment).await;
    }

    async fn pip_cost(&self) -> Option<DynamicSpellRank> {
        if let Ok(addr) = self.read_value_from_offset::<i64>(176).await {
            if addr != 0 {
                return Some(DynamicSpellRank::new(self.hook_handler(), addr));
            }
        }
        None
    }

    async fn regular_adjust(&self) -> i32 {
        self.read_value_from_offset(192).await.unwrap_or(0)
    }

    async fn write_regular_adjust(&self, regular_adjust: i32) {
        let _ = self.write_value_to_offset(192, &regular_adjust).await;
    }

    async fn magic_school_id(&self) -> u32 {
        self.read_value_from_offset(136).await.unwrap_or(0)
    }

    async fn write_magic_school_id(&self, magic_school_id: u32) {
        let _ = self.write_value_to_offset(136, &magic_school_id).await;
    }

    async fn accuracy(&self) -> u8 {
        self.read_value_from_offset(132).await.unwrap_or(0)
    }

    async fn write_accuracy(&self, accuracy: u8) {
        let _ = self.write_value_to_offset(132, &accuracy).await;
    }

    async fn spell_effects(&self) -> Vec<DynamicSpellEffect> {
        let mut effects = Vec::new();
        if let Ok(addrs) = self.read_shared_vector(88).await {
            for addr in addrs {
                effects.push(DynamicSpellEffect::new(self.hook_handler(), addr));
            }
        }
        effects
    }

    async fn treasure_card(&self) -> bool {
        self.read_value_from_offset(197).await.unwrap_or(false)
    }

    async fn write_treasure_card(&self, treasure_card: bool) {
        let _ = self.write_value_to_offset(197, &treasure_card).await;
    }

    async fn battle_card(&self) -> bool {
        self.read_value_from_offset(198).await.unwrap_or(false)
    }

    async fn write_battle_card(&self, battle_card: bool) {
        let _ = self.write_value_to_offset(198, &battle_card).await;
    }

    async fn item_card(&self) -> bool {
        self.read_value_from_offset(199).await.unwrap_or(false)
    }

    async fn write_item_card(&self, item_card: bool) {
        let _ = self.write_value_to_offset(199, &item_card).await;
    }

    async fn side_board(&self) -> bool {
        self.read_value_from_offset(200).await.unwrap_or(false)
    }

    async fn write_side_board(&self, side_board: bool) {
        let _ = self.write_value_to_offset(200, &side_board).await;
    }

    async fn spell_id(&self) -> u32 {
        self.read_value_from_offset(204).await.unwrap_or(0)
    }

    async fn write_spell_id(&self, spell_id: u32) {
        let _ = self.write_value_to_offset(204, &spell_id).await;
    }

    async fn leaves_play_when_cast_override(&self) -> bool {
        self.read_value_from_offset(216).await.unwrap_or(false)
    }

    async fn write_leaves_play_when_cast_override(&self, leaves_play_when_cast_override: bool) {
        let _ = self.write_value_to_offset(216, &leaves_play_when_cast_override).await;
    }

    async fn cloaked(&self) -> bool {
        self.read_value_from_offset(196).await.unwrap_or(false)
    }

    async fn write_cloaked(&self, cloaked: bool) {
        let _ = self.write_value_to_offset(196, &cloaked).await;
    }

    async fn enchantment_spell_is_item_card(&self) -> bool {
        self.read_value_from_offset(76).await.unwrap_or(false)
    }

    async fn write_enchantment_spell_is_item_card(&self, enchantment_spell_is_item_card: bool) {
        let _ = self.write_value_to_offset(76, &enchantment_spell_is_item_card).await;
    }

    async fn premutation_spell_id(&self) -> u32 {
        self.read_value_from_offset(112).await.unwrap_or(0)
    }

    async fn write_premutation_spell_id(&self, premutation_spell_id: u32) {
        let _ = self.write_value_to_offset(112, &premutation_spell_id).await;
    }

    async fn enchanted_this_combat(&self) -> bool {
        self.read_value_from_offset(77).await.unwrap_or(false)
    }

    async fn write_enchanted_this_combat(&self, enchanted_this_combat: bool) {
        let _ = self.write_value_to_offset(77, &enchanted_this_combat).await;
    }

    async fn delay_enchantment(&self) -> bool {
        self.read_value_from_offset(257).await.unwrap_or(false)
    }

    async fn write_delay_enchantment(&self, delay_enchantment: bool) {
        let _ = self.write_value_to_offset(257, &delay_enchantment).await;
    }

    async fn pve(&self) -> bool {
        self.read_value_from_offset(264).await.unwrap_or(false)
    }

    async fn write_pve(&self, pve: bool) {
        let _ = self.write_value_to_offset(264, &pve).await;
    }

    async fn round_added_tc(&self) -> i32 {
        self.read_value_from_offset(260).await.unwrap_or(0)
    }

    async fn write_round_added_tc(&self, round_added_tc: i32) {
        let _ = self.write_value_to_offset(260, &round_added_tc).await;
    }

    async fn secondary_school_id(&self) -> u32 {
        self.read_value_from_offset(304).await.unwrap_or(0)
    }

    async fn write_secondary_school_id(&self, secondary_school_id: u32) {
        let _ = self.write_value_to_offset(304, &secondary_school_id).await;
    }
}
