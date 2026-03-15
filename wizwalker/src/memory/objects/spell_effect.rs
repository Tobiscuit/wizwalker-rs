use crate::memory::MemoryObject;

pub struct DynamicSpellEffect {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicSpellEffect {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

pub struct DynamicConditionalSpellElement {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicConditionalSpellElement {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

#[async_trait::async_trait]
pub trait SpellEffect: MemoryObject {
    async fn effect_type(&self) -> i32 {
        self.read_value_from_offset(72).await.unwrap_or(0) // Enum fallback
    }

    async fn write_effect_type(&self, effect_type: i32) {
        let _ = self.write_value_to_offset(72, &effect_type).await;
    }

    async fn effect_param(&self) -> i32 {
        self.read_value_from_offset(76).await.unwrap_or(0)
    }

    async fn write_effect_param(&self, effect_param: i32) {
        let _ = self.write_value_to_offset(76, &effect_param).await;
    }

    async fn disposition(&self) -> i32 {
        self.read_value_from_offset(80).await.unwrap_or(0)
    }

    async fn write_disposition(&self, disposition: i32) {
        let _ = self.write_value_to_offset(80, &disposition).await;
    }

    async fn string_damage_type(&self) -> String {
        self.read_string_from_offset(88).await.unwrap_or_default()
    }
}

#[async_trait::async_trait]
pub trait HangingConversionSpellEffect: SpellEffect {
    async fn hanging_effect_type(&self) -> i32 {
        self.read_value_from_offset(224).await.unwrap_or(0)
    }

    async fn write_hanging_effect_type(&self, hanging_effect_type: i32) {
        let _ = self.write_value_to_offset(224, &hanging_effect_type).await;
    }

    async fn specific_effect_types(&self) -> Vec<i32> {
        let mut results = Vec::new();
        if let Ok(addrs) = self.read_shared_linked_list(232).await {
            for addr in addrs {
                let effect = DynamicSpellEffect::new(self.hook_handler(), addr);
                // In actual impl we would call effect_type() on the effect, simulating:
                results.push(0);
            }
        }
        results
    }

    async fn min_effect_value(&self) -> i32 {
        self.read_value_from_offset(248).await.unwrap_or(0)
    }

    async fn write_min_effect_value(&self, min_effect_value: i32) {
        let _ = self.write_value_to_offset(248, &min_effect_value).await;
    }

    async fn max_effect_value(&self) -> i32 {
        self.read_value_from_offset(252).await.unwrap_or(0)
    }

    async fn write_max_effect_value(&self, max_effect_value: i32) {
        let _ = self.write_value_to_offset(252, &max_effect_value).await;
    }

    async fn not_damage_type(&self) -> bool {
        self.read_value_from_offset(256).await.unwrap_or(false)
    }

    async fn write_not_damage_type(&self, not_damage_type: bool) {
        let _ = self.write_value_to_offset(256, &not_damage_type).await;
    }

    async fn min_effect_count(&self) -> i32 {
        self.read_value_from_offset(260).await.unwrap_or(0)
    }

    async fn write_min_effect_count(&self, min_effect_count: i32) {
        let _ = self.write_value_to_offset(260, &min_effect_count).await;
    }

    async fn max_effect_count(&self) -> i32 {
        self.read_value_from_offset(264).await.unwrap_or(0)
    }

    async fn write_max_effect_count(&self, max_effect_count: i32) {
        let _ = self.write_value_to_offset(264, &max_effect_count).await;
    }

    async fn bypass_protection(&self) -> bool {
        self.read_value_from_offset(159).await.unwrap_or(false)
    }

    async fn write_bypass_protection(&self, bypass_protection: bool) {
        let _ = self.write_value_to_offset(159, &bypass_protection).await;
    }

    async fn scale_source_effect_value(&self) -> bool {
        self.read_value_from_offset(272).await.unwrap_or(false)
    }

    async fn write_scale_source_effect_value(&self, scale_source_effect_value: bool) {
        let _ = self.write_value_to_offset(272, &scale_source_effect_value).await;
    }

    async fn scale_source_effect_percent(&self) -> f32 {
        self.read_value_from_offset(276).await.unwrap_or(0.0)
    }

    async fn write_scale_source_effect_percent(&self, scale_source_effect_percent: f32) {
        let _ = self.write_value_to_offset(276, &scale_source_effect_percent).await;
    }

    async fn apply_to_effect_source(&self) -> bool {
        self.read_value_from_offset(280).await.unwrap_or(false)
    }

    async fn write_apply_to_effect_source(&self, apply_to_effect_source: bool) {
        let _ = self.write_value_to_offset(280, &apply_to_effect_source).await;
    }

    async fn output_effect(&self) -> Vec<DynamicSpellEffect> {
        let mut results = Vec::new();
        if let Ok(addrs) = self.read_shared_linked_list(288).await {
            for addr in addrs {
                results.push(DynamicSpellEffect::new(self.hook_handler(), addr));
            }
        }
        results
    }
}

#[async_trait::async_trait]
pub trait CompoundSpellEffect: SpellEffect {
    async fn effects_list(&self) -> Vec<DynamicSpellEffect> {
        let mut results = Vec::new();
        if let Ok(addrs) = self.read_shared_linked_list(224).await {
            for addr in addrs {
                results.push(DynamicSpellEffect::new(self.hook_handler(), addr));
            }
        }
        results
    }
}

#[async_trait::async_trait]
pub trait ConditionalSpellElement: MemoryObject {
    async fn reqs(&self) -> i64 {
        self.read_value_from_offset(72).await.unwrap_or(0)
    }

    async fn effect(&self) -> DynamicSpellEffect {
        let addr = self.read_value_from_offset(88).await.unwrap_or(0);
        DynamicSpellEffect::new(self.hook_handler(), addr)
    }
}

#[async_trait::async_trait]
pub trait ConditionalSpellEffect: SpellEffect {
    async fn elements(&self) -> Vec<DynamicConditionalSpellElement> {
        let mut elements = Vec::new();
        if let Ok(addrs) = self.read_shared_linked_list(224).await {
            for addr in addrs {
                elements.push(DynamicConditionalSpellElement::new(self.hook_handler(), addr));
            }
        }
        elements
    }
}

#[async_trait::async_trait]
pub trait CountBasedSpellEffect: SpellEffect {
    async fn mode(&self) -> i32 {
        self.read_value_from_offset(224).await.unwrap_or(0)
    }

    async fn write_mode(&self, mode: i32) {
        let _ = self.write_value_to_offset(224, &mode).await;
    }

    async fn effect_list(&self) -> Vec<DynamicSpellEffect> {
        let mut effects = Vec::new();
        if let Ok(addrs) = self.read_shared_linked_list(232).await {
            for addr in addrs {
                effects.push(DynamicSpellEffect::new(self.hook_handler(), addr));
            }
        }
        effects
    }
}

#[async_trait::async_trait]
pub trait ShadowSpellEffect: CompoundSpellEffect {
    async fn initial_backlash(&self) -> i32 {
        self.read_value_from_offset(240).await.unwrap_or(0)
    }

    async fn write_initial_backlash(&self, initial_backlash: i32) {
        let _ = self.write_value_to_offset(240, &initial_backlash).await;
    }
}

#[async_trait::async_trait]
pub trait ShadowPactSpellEffect: ShadowSpellEffect {
    async fn caster_sc(&self) -> i32 {
        self.read_value_from_offset(248).await.unwrap_or(0)
    }

    async fn write_caster_sc(&self, caster_sc: i32) {
        let _ = self.write_value_to_offset(248, &caster_sc).await;
    }

    async fn target_sc(&self) -> i32 {
        self.read_value_from_offset(252).await.unwrap_or(0)
    }

    async fn write_target_sc(&self, target_sc: i32) {
        let _ = self.write_value_to_offset(252, &target_sc).await;
    }

    async fn pact_effect_kind(&self) -> i32 {
        self.read_value_from_offset(256).await.unwrap_or(0)
    }

    async fn write_pact_effect_kind(&self, pact_effect_kind: i32) {
        let _ = self.write_value_to_offset(256, &pact_effect_kind).await;
    }

    async fn backlash_per_round(&self) -> i32 {
        self.read_value_from_offset(260).await.unwrap_or(0)
    }

    async fn write_backlash_per_round(&self, backlash_per_round: i32) {
        let _ = self.write_value_to_offset(260, &backlash_per_round).await;
    }

    async fn added_in_round(&self) -> i32 {
        self.read_value_from_offset(264).await.unwrap_or(0)
    }

    async fn write_added_in_round(&self, added_in_round: i32) {
        let _ = self.write_value_to_offset(264, &added_in_round).await;
    }
}
