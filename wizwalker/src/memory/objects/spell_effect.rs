//! Spell effect memory objects — represents spell effects in the game's memory.
//!
//! # Python equivalent
//! `wizwalker/memory/memory_objects/spell_effect.py` — `SpellEffect` / `DynamicSpellEffect`.
//!
//! All offsets are taken directly from the Python source.

use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};

// Re-export spell effect types from enums for convenience.
pub use super::enums::{EffectTarget, SpellEffects};

/// A spell effect in the game's memory.
///
/// Wraps a `DynamicMemoryObject` pointing at a `SpellEffect` PropertyClass instance.
/// All offsets match the Python WizWalker source exactly.
#[derive(Clone)]
pub struct DynamicSpellEffect {
    pub inner: DynamicMemoryObject,
}

impl DynamicSpellEffect {
    /// Create from a `DynamicMemoryObject`.
    pub fn from_memory(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    /// Convenience: create from a reader and base address.
    pub fn from_addr(
        reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>,
        address: u64,
    ) -> Result<Self> {
        let inner = DynamicMemoryObject::new(reader, address)?;
        Ok(Self { inner })
    }

    // ── Core Properties (from Python SpellEffect) ───────────────────

    /// Effect type (damage, heal, charm, ward, etc.).
    /// Python offset: 72, read as enum (int32)
    pub fn effect_type(&self) -> Result<i32> {
        self.inner.read_value_from_offset(72)
    }

    pub fn write_effect_type(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(72, &val)
    }

    /// Effect parameter (damage/heal amount, etc.).
    /// Python offset: 76, Primitive.int32
    pub fn effect_param(&self) -> Result<i32> {
        self.inner.read_value_from_offset(76)
    }

    pub fn write_effect_param(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(76, &val)
    }

    /// Disposition (positive/negative/both).
    /// Python offset: 80, enum (int32)
    pub fn disposition(&self) -> Result<i32> {
        self.inner.read_value_from_offset(80)
    }

    pub fn write_disposition(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(80, &val)
    }

    /// Damage type.
    /// Python offset: 84, Primitive.uint32
    pub fn damage_type(&self) -> Result<u32> {
        self.inner.read_value_from_offset(84)
    }

    pub fn write_damage_type(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(84, &val)
    }

    /// Spell template ID.
    /// Python offset: 120, Primitive.uint32
    pub fn spell_template_id(&self) -> Result<u32> {
        self.inner.read_value_from_offset(120)
    }

    pub fn write_spell_template_id(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(120, &val)
    }

    /// Enchantment spell template ID.
    /// Python offset: 124, Primitive.uint32
    pub fn enchantment_spell_template_id(&self) -> Result<u32> {
        self.inner.read_value_from_offset(124)
    }

    pub fn write_enchantment_spell_template_id(&self, val: u32) -> Result<()> {
        self.inner.write_value_to_offset(124, &val)
    }

    /// Pip num.
    /// Python offset: 128, Primitive.int32
    pub fn pip_num(&self) -> Result<i32> {
        self.inner.read_value_from_offset(128)
    }

    pub fn write_pip_num(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(128, &val)
    }

    /// Act num.
    /// Python offset: 132, Primitive.int32
    pub fn act_num(&self) -> Result<i32> {
        self.inner.read_value_from_offset(132)
    }

    pub fn write_act_num(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(132, &val)
    }

    /// Act flag.
    /// Python offset: 136, Primitive.bool
    pub fn act(&self) -> Result<bool> {
        self.inner.read_value_from_offset(136)
    }

    pub fn write_act(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(136, &val)
    }

    /// Effect target (self, enemy team, friendly, etc.).
    /// Python offset: 140, enum (int32)
    pub fn effect_target(&self) -> Result<i32> {
        self.inner.read_value_from_offset(140)
    }

    pub fn write_effect_target(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(140, &val)
    }

    /// Number of rounds.
    /// Python offset: 144, Primitive.int32
    pub fn num_rounds(&self) -> Result<i32> {
        self.inner.read_value_from_offset(144)
    }

    pub fn write_num_rounds(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(144, &val)
    }

    /// Param per round.
    /// Python offset: 148, Primitive.int32
    pub fn param_per_round(&self) -> Result<i32> {
        self.inner.read_value_from_offset(148)
    }

    pub fn write_param_per_round(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(148, &val)
    }

    /// Heal modifier.
    /// Python offset: 152, Primitive.float32
    pub fn heal_modifier(&self) -> Result<f32> {
        self.inner.read_value_from_offset(152)
    }

    pub fn write_heal_modifier(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(152, &val)
    }

    /// Cloaked flag.
    /// Python offset: 157, Primitive.bool
    pub fn cloaked(&self) -> Result<bool> {
        self.inner.read_value_from_offset(157)
    }

    pub fn write_cloaked(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(157, &val)
    }

    /// Bypass protection.
    /// Python offset: 159, Primitive.bool
    pub fn bypass_protection(&self) -> Result<bool> {
        self.inner.read_value_from_offset(159)
    }

    pub fn write_bypass_protection(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(159, &val)
    }

    /// Armor piercing param.
    /// Python offset: 160, Primitive.int32
    pub fn armor_piercing_param(&self) -> Result<i32> {
        self.inner.read_value_from_offset(160)
    }

    pub fn write_armor_piercing_param(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(160, &val)
    }

    /// Chance per target.
    /// Python offset: 164, Primitive.int32
    pub fn chance_per_target(&self) -> Result<i32> {
        self.inner.read_value_from_offset(164)
    }

    pub fn write_chance_per_target(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(164, &val)
    }

    /// Protected flag.
    /// Python offset: 168, Primitive.bool
    pub fn protected(&self) -> Result<bool> {
        self.inner.read_value_from_offset(168)
    }

    pub fn write_protected(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(168, &val)
    }

    /// Converted flag.
    /// Python offset: 169, Primitive.bool
    pub fn converted(&self) -> Result<bool> {
        self.inner.read_value_from_offset(169)
    }

    pub fn write_converted(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(169, &val)
    }

    /// Rank.
    /// Python offset: 208, Primitive.int32
    pub fn rank(&self) -> Result<i32> {
        self.inner.read_value_from_offset(208)
    }

    pub fn write_rank(&self, val: i32) -> Result<()> {
        self.inner.write_value_to_offset(208, &val)
    }

    // ── Complex Methods ─────────────────────────────────────────────

    /// Read the type name of this effect (via RTTI vtable walk).
    /// Python: PropertyClass.maybe_read_type_name()
    /// TODO: Implement RTTI vtable walking for proper type name resolution.
    pub fn maybe_read_type_name(&self) -> Result<String> {
        // RTTI vtable walk not yet implemented — return empty string.
        // Full implementation requires reading vtable pointer at offset 0,
        // following it to the first function, then resolving the LEA instruction.
        Ok(String::new())
    }

    /// Read sub-effects (for variable/random/effectlist types).
    /// Python: SpellEffect.maybe_effect_list — reads shared_linked_list at offset 224.
    pub fn maybe_effect_list(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.inner.read_shared_linked_list(224)?;
        addrs
            .into_iter()
            .map(|addr| {
                let inner = DynamicMemoryObject::new(self.inner.reader(), addr as u64)?;
                Ok(DynamicSpellEffect::from_memory(inner))
            })
            .collect()
    }

    /// Read string damage type.
    /// Python offset: 88, via read_string_from_offset
    pub fn string_damage_type(&self) -> Result<String> {
        self.inner.read_string_from_offset(88)
    }
}

/// A conditional spell element (for conditional spell effects).
#[derive(Clone)]
pub struct DynamicConditionalSpellElement {
    pub inner: DynamicMemoryObject,
}

impl DynamicConditionalSpellElement {
    pub fn from_memory(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    /// Requirements list pointer.
    /// Python offset: 72, Primitive.uint64
    pub fn reqs_address(&self) -> Result<u64> {
        self.inner.read_value_from_offset(72)
    }

    /// Effect pointer.
    /// Python offset: 88, Primitive.uint64
    pub fn effect(&self) -> Result<DynamicSpellEffect> {
        let addr: u64 = self.inner.read_value_from_offset(88)?;
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(DynamicSpellEffect::from_memory(inner))
    }
}

/// Legacy trait kept for backward compatibility.
pub trait SpellEffect: MemoryObject {
    fn effect_type(&self) -> i32 {
        self.read_value_from_offset(72).unwrap_or(0)
    }

    fn write_effect_type(&self, val: i32) {
        let _ = self.write_value_to_offset(72, &val);
    }

    fn effect_param(&self) -> i32 {
        self.read_value_from_offset(76).unwrap_or(0)
    }

    fn write_effect_param(&self, val: i32) {
        let _ = self.write_value_to_offset(76, &val);
    }

    fn effect_target(&self) -> i32 {
        self.read_value_from_offset(140).unwrap_or(0)
    }

    fn write_effect_target(&self, val: i32) {
        let _ = self.write_value_to_offset(140, &val);
    }

    fn num_rounds(&self) -> i32 {
        self.read_value_from_offset(144).unwrap_or(0)
    }

    fn heal_modifier(&self) -> f32 {
        self.read_value_from_offset(152).unwrap_or(0.0)
    }
}

// ── Spell Effect Variant Types ──────────────────────────────────────

/// A hanging conversion spell effect (e.g., Balanceblade).
///
/// Extends the base `DynamicSpellEffect` with conversion-specific fields.
/// Python: `HangingConversionSpellEffect(DynamicSpellEffect)`
#[derive(Clone)]
pub struct HangingConversionSpellEffect {
    pub base: DynamicSpellEffect,
}

impl HangingConversionSpellEffect {
    pub fn new(base: DynamicSpellEffect) -> Self {
        Self { base }
    }

    /// Type of hanging effect.
    /// Python offset: 224, read_enum(HangingEffectType)
    pub fn hanging_effect_type(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(224)
    }

    pub fn write_hanging_effect_type(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(224, &value)
    }

    /// Read specific effect types (shared linked list at offset 232).
    /// Returns spell effect type IDs that this conversion applies to.
    /// Python offset: 232, read_shared_linked_list
    pub fn specific_effect_types(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.base.inner.read_shared_linked_list(232)?;
        addrs
            .into_iter()
            .map(|addr| {
                let inner = DynamicMemoryObject::new(self.base.inner.reader(), addr as u64)?;
                Ok(DynamicSpellEffect::from_memory(inner))
            })
            .collect()
    }

    /// Minimum effect value for conversion.
    /// Python offset: 248, Primitive.int32
    pub fn min_effect_value(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(248)
    }

    pub fn write_min_effect_value(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(248, &value)
    }

    /// Maximum effect value for conversion.
    /// Python offset: 252, Primitive.int32
    pub fn max_effect_value(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(252)
    }

    pub fn write_max_effect_value(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(252, &value)
    }

    /// Whether this does NOT apply to the damage type.
    /// Python offset: 256, Primitive.bool
    pub fn not_damage_type(&self) -> Result<bool> {
        self.base.inner.read_value_from_offset(256)
    }

    pub fn write_not_damage_type(&self, value: bool) -> Result<()> {
        self.base.inner.write_value_to_offset(256, &value)
    }

    /// Minimum number of effects to trigger.
    /// Python offset: 260, Primitive.int32
    pub fn min_effect_count(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(260)
    }

    pub fn write_min_effect_count(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(260, &value)
    }

    /// Maximum number of effects to trigger.
    /// Python offset: 264, Primitive.int32
    pub fn max_effect_count(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(264)
    }

    pub fn write_max_effect_count(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(264, &value)
    }

    /// How to select the output effect.
    /// Python offset: 268, read_enum(OutputEffectSelector)
    pub fn output_selector(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(268)
    }

    pub fn write_output_selector(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(268, &value)
    }

    /// Whether to scale the source effect's value.
    /// Python offset: 272, Primitive.bool
    pub fn scale_source_effect_value(&self) -> Result<bool> {
        self.base.inner.read_value_from_offset(272)
    }

    pub fn write_scale_source_effect_value(&self, value: bool) -> Result<()> {
        self.base.inner.write_value_to_offset(272, &value)
    }

    /// Scaling percentage for the source effect.
    /// Python offset: 276, Primitive.float32
    pub fn scale_source_effect_percent(&self) -> Result<f32> {
        self.base.inner.read_value_from_offset(276)
    }

    pub fn write_scale_source_effect_percent(&self, value: f32) -> Result<()> {
        self.base.inner.write_value_to_offset(276, &value)
    }

    /// Whether to apply to the effect source.
    /// Python offset: 280, Primitive.bool
    pub fn apply_to_effect_source(&self) -> Result<bool> {
        self.base.inner.read_value_from_offset(280)
    }

    pub fn write_apply_to_effect_source(&self, value: bool) -> Result<()> {
        self.base.inner.write_value_to_offset(280, &value)
    }

    /// Output effects (shared linked list at offset 288).
    /// Python offset: 288, read_shared_linked_list
    pub fn output_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.base.inner.read_shared_linked_list(288)?;
        addrs
            .into_iter()
            .map(|addr| {
                let inner = DynamicMemoryObject::new(self.base.inner.reader(), addr as u64)?;
                Ok(DynamicSpellEffect::from_memory(inner))
            })
            .collect()
    }
}

/// A compound spell effect (base for Random, Variable, EffectList).
///
/// Python: `CompoundSpellEffect(DynamicSpellEffect)` — not in type dump,
/// exists only to reduce code repetition.
#[derive(Clone)]
pub struct CompoundSpellEffect {
    pub base: DynamicSpellEffect,
}

impl CompoundSpellEffect {
    pub fn new(base: DynamicSpellEffect) -> Self {
        Self { base }
    }

    /// Sub-effects list (shared linked list at offset 224).
    /// Python offset: 224, read_shared_linked_list
    pub fn effects_list(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.base.inner.read_shared_linked_list(224)?;
        addrs
            .into_iter()
            .map(|addr| {
                let inner = DynamicMemoryObject::new(self.base.inner.reader(), addr as u64)?;
                Ok(DynamicSpellEffect::from_memory(inner))
            })
            .collect()
    }
}

/// A conditional spell effect (e.g., "if target has X, do Y").
///
/// Python: `ConditionalSpellEffect(DynamicSpellEffect)`
#[derive(Clone)]
pub struct ConditionalSpellEffect {
    pub base: DynamicSpellEffect,
}

impl ConditionalSpellEffect {
    pub fn new(base: DynamicSpellEffect) -> Self {
        Self { base }
    }

    /// Read conditional elements (shared linked list at offset 224).
    /// Each element has a requirements list and an effect.
    /// Python offset: 224, read_shared_linked_list
    pub fn elements(&self) -> Result<Vec<DynamicConditionalSpellElement>> {
        let addrs = self.base.inner.read_shared_linked_list(224)?;
        addrs
            .into_iter()
            .map(|addr| {
                let inner = DynamicMemoryObject::new(self.base.inner.reader(), addr as u64)?;
                Ok(DynamicConditionalSpellElement::from_memory(inner))
            })
            .collect()
    }
}

/// A count-based spell effect (scales with pip count or other counters).
///
/// Python: `CountBasedSpellEffect(DynamicSpellEffect)`
#[derive(Clone)]
pub struct CountBasedSpellEffect {
    pub base: DynamicSpellEffect,
}

impl CountBasedSpellEffect {
    pub fn new(base: DynamicSpellEffect) -> Self {
        Self { base }
    }

    /// The mode/type of count-based scaling.
    /// Python offset: 224, read_enum(CountBasedType)
    pub fn mode(&self) -> Result<i32> {
        self.base.inner.read_value_from_offset(224)
    }

    pub fn write_mode(&self, value: i32) -> Result<()> {
        self.base.inner.write_value_to_offset(224, &value)
    }

    /// Effect list (shared linked list at offset 232).
    /// Python offset: 232, read_shared_linked_list
    pub fn effect_list(&self) -> Result<Vec<DynamicSpellEffect>> {
        let addrs = self.base.inner.read_shared_linked_list(232)?;
        addrs
            .into_iter()
            .map(|addr| {
                let inner = DynamicMemoryObject::new(self.base.inner.reader(), addr as u64)?;
                Ok(DynamicSpellEffect::from_memory(inner))
            })
            .collect()
    }
}

/// A shadow spell effect (extends CompoundSpellEffect/EffectListSpellEffect).
///
/// Python: `ShadowSpellEffect(EffectListSpellEffect)`
#[derive(Clone)]
pub struct ShadowSpellEffect {
    pub base: CompoundSpellEffect,
}

impl ShadowSpellEffect {
    pub fn new(base: DynamicSpellEffect) -> Self {
        Self {
            base: CompoundSpellEffect::new(base),
        }
    }

    /// Effects list (delegated from CompoundSpellEffect).
    pub fn effects_list(&self) -> Result<Vec<DynamicSpellEffect>> {
        self.base.effects_list()
    }

    /// Initial backlash damage from the shadow magic.
    /// Python offset: 240, Primitive.int32
    pub fn initial_backlash(&self) -> Result<i32> {
        self.base.base.inner.read_value_from_offset(240)
    }

    pub fn write_initial_backlash(&self, value: i32) -> Result<()> {
        self.base.base.inner.write_value_to_offset(240, &value)
    }
}

/// A shadow pact spell effect (extends ShadowSpellEffect).
///
/// Python: `ShadowPactSpellEffect(ShadowSpellEffect)`
#[derive(Clone)]
pub struct ShadowPactSpellEffect {
    pub base: ShadowSpellEffect,
}

impl ShadowPactSpellEffect {
    pub fn new(base: DynamicSpellEffect) -> Self {
        Self {
            base: ShadowSpellEffect::new(base),
        }
    }

    /// Effects list (delegated from ShadowSpellEffect → CompoundSpellEffect).
    pub fn effects_list(&self) -> Result<Vec<DynamicSpellEffect>> {
        self.base.effects_list()
    }

    /// Initial backlash (delegated from ShadowSpellEffect).
    pub fn initial_backlash(&self) -> Result<i32> {
        self.base.initial_backlash()
    }

    /// Caster spell count modifier.
    /// Python offset: 248, Primitive.int32
    pub fn caster_sc(&self) -> Result<i32> {
        self.base.base.base.inner.read_value_from_offset(248)
    }

    pub fn write_caster_sc(&self, value: i32) -> Result<()> {
        self.base.base.base.inner.write_value_to_offset(248, &value)
    }

    /// Target spell count modifier.
    /// Python offset: 252, Primitive.int32
    pub fn target_sc(&self) -> Result<i32> {
        self.base.base.base.inner.read_value_from_offset(252)
    }

    pub fn write_target_sc(&self, value: i32) -> Result<()> {
        self.base.base.base.inner.write_value_to_offset(252, &value)
    }

    /// Pact effect kind.
    /// Python offset: 256, read_enum(EffectKinds)
    pub fn pact_effect_kind(&self) -> Result<i32> {
        self.base.base.base.inner.read_value_from_offset(256)
    }

    pub fn write_pact_effect_kind(&self, value: i32) -> Result<()> {
        self.base.base.base.inner.write_value_to_offset(256, &value)
    }

    /// Backlash damage applied each round.
    /// Python offset: 260, Primitive.int32
    pub fn backlash_per_round(&self) -> Result<i32> {
        self.base.base.base.inner.read_value_from_offset(260)
    }

    pub fn write_backlash_per_round(&self, value: i32) -> Result<()> {
        self.base.base.base.inner.write_value_to_offset(260, &value)
    }

    /// Round this pact was added.
    /// Python offset: 264, Primitive.int32
    pub fn added_in_round(&self) -> Result<i32> {
        self.base.base.base.inner.read_value_from_offset(264)
    }

    pub fn write_added_in_round(&self, value: i32) -> Result<()> {
        self.base.base.base.inner.write_value_to_offset(264, &value)
    }
}

// ── Variant Enum & Dispatcher ───────────────────────────────────────

/// All possible spell effect variant types.
///
/// Python uses class inheritance for this (DynamicSpellEffect → ShadowSpellEffect etc.).
/// In Rust, we use an enum to represent the dispatched variant.
#[derive(Clone)]
pub enum SpellEffectVariant {
    /// Base spell effect (no special variant).
    Base(DynamicSpellEffect),
    /// Hanging conversion (e.g., Balanceblade, trap conversion).
    HangingConversion(HangingConversionSpellEffect),
    /// Conditional (if X then apply Y).
    Conditional(ConditionalSpellEffect),
    /// Count-based (scales with pip count or other counters).
    CountBased(CountBasedSpellEffect),
    /// Random (picks one sub-effect at random).
    Random(CompoundSpellEffect),
    /// Random per target (picks one sub-effect per target).
    RandomPerTarget(CompoundSpellEffect),
    /// Variable (picks sub-effect based on variables).
    Variable(CompoundSpellEffect),
    /// Effect list (applies all sub-effects in sequence).
    EffectList(CompoundSpellEffect),
    /// Shadow (shadow magic with backlash).
    Shadow(ShadowSpellEffect),
    /// Shadow pact (shadow pact with per-round backlash).
    ShadowPact(ShadowPactSpellEffect),
}

impl SpellEffectVariant {
    /// Access the base `DynamicSpellEffect` regardless of variant.
    pub fn base_effect(&self) -> &DynamicSpellEffect {
        match self {
            Self::Base(e) => e,
            Self::HangingConversion(e) => &e.base,
            Self::Conditional(e) => &e.base,
            Self::CountBased(e) => &e.base,
            Self::Random(e) | Self::RandomPerTarget(e) | Self::Variable(e) | Self::EffectList(e) => {
                &e.base
            }
            Self::Shadow(e) => &e.base.base,
            Self::ShadowPact(e) => &e.base.base.base,
        }
    }
}

/// Dispatch a base `DynamicSpellEffect` to its proper variant type.
///
/// Python: `cast_effect_variant(read_effect)` — reads the RTTI type name
/// and returns the appropriate subclass.
///
/// **Note:** This currently returns `SpellEffectVariant::Base` because
/// RTTI vtable walking (`read_type_name()`) is not yet implemented.
/// Once RTTI is implemented, this function will read the type name and
/// return the correct variant.
pub fn cast_effect_variant(effect: DynamicSpellEffect) -> SpellEffectVariant {
    // TODO: Implement RTTI-based dispatch once maybe_read_type_name() works.
    // Once RTTI is available, the dispatch will be:
    //
    // match effect.maybe_read_type_name().unwrap_or_default().as_str() {
    //     "HangingConversionSpellEffect" => SpellEffectVariant::HangingConversion(...),
    //     "ConditionalSpellEffect" => SpellEffectVariant::Conditional(...),
    //     "ShadowSpellEffect" => SpellEffectVariant::Shadow(...),
    //     "ShadowPactSpellEffect" => SpellEffectVariant::ShadowPact(...),
    //     "CountBasedSpellEffect" => SpellEffectVariant::CountBased(...),
    //     "RandomSpellEffect" => SpellEffectVariant::Random(...),
    //     "RandomPerTargetSpellEffect" => SpellEffectVariant::RandomPerTarget(...),
    //     "VariableSpellEffect" => SpellEffectVariant::Variable(...),
    //     "EffectListSpellEffect" => SpellEffectVariant::EffectList(...),
    //     _ => SpellEffectVariant::Base(effect),
    // }
    SpellEffectVariant::Base(effect)
}

/// Read spell effects from a memory object at the given offset and cast each
/// to its proper variant type.
///
/// Python: `get_spell_effects(base, offset)` — reads shared_vector, then casts each.
pub fn get_spell_effects(
    reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>,
    base_address: u64,
    offset: u64,
) -> Result<Vec<SpellEffectVariant>> {
    let start: u64 = {
        let r = reader.clone();
        crate::memory::reader::MemoryReaderExt::read_typed(&*r, (base_address + offset) as usize)?
    };
    let end: u64 = {
        let r = reader.clone();
        crate::memory::reader::MemoryReaderExt::read_typed(&*r, (base_address + offset + 8) as usize)?
    };

    if start == 0 || end == 0 || end <= start {
        return Ok(Vec::new());
    }

    let count = ((end - start) / 8) as usize;
    let mut effects = Vec::with_capacity(count);

    for i in 0..count {
        let addr: u64 = crate::memory::reader::MemoryReaderExt::read_typed(
            &*reader,
            (start + (i as u64) * 8) as usize,
        )?;
        if addr != 0 {
            let inner = DynamicMemoryObject::new(reader.clone(), addr)?;
            let effect = DynamicSpellEffect::from_memory(inner);
            effects.push(cast_effect_variant(effect));
        }
    }

    Ok(effects)
}
