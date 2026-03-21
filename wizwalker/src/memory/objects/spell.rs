use crate::memory::reader::MemoryReaderExt;
use crate::memory::{MemoryObject, MemoryObjectExt};
use crate::memory::memory_object::DynamicMemoryObject;
use crate::errors::Result;
use crate::memory::objects::spell_effect::DynamicSpellEffect;

/// Concrete graphical spell memory object (used in combat/deck UI).
///
/// Wraps a `DynamicMemoryObject` pointing to the spell's memory region.
/// Provides read/write access to spell properties like template ID, accuracy,
/// enchantment status, and spell effects.
#[derive(Clone)]
pub struct DynamicGraphicalSpell {
    pub inner: DynamicMemoryObject,
}

impl DynamicGraphicalSpell {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    /// Read the spell template pointer and return it as a `DynamicSpellTemplate`.
    pub fn spell_template(&self) -> Result<DynamicSpellTemplate> {
        let addr: u64 = self.inner.read_value_from_offset(120)?;
        if addr == 0 {
            return Err(crate::errors::WizWalkerError::AddressOutOfRange(0));
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(DynamicSpellTemplate::new(inner))
    }

    /// Read the spell's template ID.
    pub fn template_id(&self) -> Result<u32> {
        self.inner.read_value_from_offset(128)
    }

    pub fn write_template_id(&self, template_id: u32) -> Result<()> {
        self.inner.write_value_to_offset(128, &template_id)
    }

    /// Read the spell ID.
    pub fn spell_id(&self) -> Result<u32> {
        self.inner.read_value_from_offset(204)
    }

    pub fn write_spell_id(&self, spell_id: u32) -> Result<()> {
        self.inner.write_value_to_offset(204, &spell_id)
    }

    /// Read the spell's accuracy (0–100).
    pub fn accuracy(&self) -> Result<u8> {
        self.inner.read_value_from_offset(132)
    }

    pub fn write_accuracy(&self, accuracy: u8) -> Result<()> {
        self.inner.write_value_to_offset(132, &accuracy)
    }

    /// Read the magic school ID.
    pub fn magic_school_id(&self) -> Result<u32> {
        self.inner.read_value_from_offset(136)
    }

    pub fn write_magic_school_id(&self, magic_school_id: u32) -> Result<()> {
        self.inner.write_value_to_offset(136, &magic_school_id)
    }

    /// Read the enchantment value (0 = not enchanted).
    pub fn enchantment(&self) -> Result<u32> {
        self.inner.read_value_from_offset(80)
    }

    pub fn write_enchantment(&self, enchantment: u32) -> Result<()> {
        self.inner.write_value_to_offset(80, &enchantment)
    }

    /// Read the list of spell effects.
    pub fn spell_effects(&self) -> Result<Vec<DynamicSpellEffect>> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();

        // Effects stored as a std::vector<shared_ptr> at offset 88.
        let vec_begin: u64 = reader.read_typed((base_address + 88) as usize)?;
        let vec_end: u64 = reader.read_typed((base_address + 96) as usize)?;

        if vec_begin == 0 || vec_end == 0 || vec_end <= vec_begin {
            return Ok(Vec::new());
        }

        let count = ((vec_end - vec_begin) / 8) as u32;
        let mut effects = Vec::with_capacity(count as usize);
        for i in 0..count {
            let effect_addr: u64 = reader.read_typed((vec_begin + (i as u64) * 8) as usize)?;
            if effect_addr != 0 {
                effects.push(DynamicSpellEffect::from_addr(self.inner.reader(), effect_addr)?);
            }
        }
        Ok(effects)
    }

    /// Whether this is a treasure card.
    pub fn treasure_card(&self) -> Result<bool> {
        self.inner.read_value_from_offset(197)
    }

    pub fn write_treasure_card(&self, treasure_card: bool) -> Result<()> {
        self.inner.write_value_to_offset(197, &treasure_card)
    }

    /// Whether this is a battle card.
    pub fn battle_card(&self) -> Result<bool> {
        self.inner.read_value_from_offset(198)
    }

    pub fn write_battle_card(&self, battle_card: bool) -> Result<()> {
        self.inner.write_value_to_offset(198, &battle_card)
    }

    /// Whether this is an item card.
    pub fn item_card(&self) -> Result<bool> {
        self.inner.read_value_from_offset(199)
    }

    pub fn write_item_card(&self, item_card: bool) -> Result<()> {
        self.inner.write_value_to_offset(199, &item_card)
    }

    /// Whether this is a sideboard card.
    pub fn side_board(&self) -> Result<bool> {
        self.inner.read_value_from_offset(200)
    }

    pub fn write_side_board(&self, side_board: bool) -> Result<()> {
        self.inner.write_value_to_offset(200, &side_board)
    }

    /// Whether this spell is cloaked.
    pub fn cloaked(&self) -> Result<bool> {
        self.inner.read_value_from_offset(196)
    }

    pub fn write_cloaked(&self, cloaked: bool) -> Result<()> {
        self.inner.write_value_to_offset(196, &cloaked)
    }

    /// Whether the enchantment came from an item card.
    pub fn enchantment_spell_is_item_card(&self) -> Result<bool> {
        self.inner.read_value_from_offset(76)
    }

    pub fn write_enchantment_spell_is_item_card(&self, value: bool) -> Result<()> {
        self.inner.write_value_to_offset(76, &value)
    }

    /// Whether this spell is PvE only.
    pub fn pve(&self) -> Result<bool> {
        self.inner.read_value_from_offset(264)
    }

    pub fn write_pve(&self, pve: bool) -> Result<()> {
        self.inner.write_value_to_offset(264, &pve)
    }

    /// Read the pip cost pointer.
    pub fn pip_cost(&self) -> Result<Option<DynamicSpellRank>> {
        let addr: u64 = self.inner.read_value_from_offset(176)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicSpellRank::new(inner)))
    }
}

// ── Supporting structs ──────────────────────────────────────────────

/// A spell template containing metadata (name, description, type, school).
#[derive(Clone)]
pub struct DynamicSpellTemplate {
    pub inner: DynamicMemoryObject,
}

impl DynamicSpellTemplate {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    /// The spell's internal name.
    pub fn name(&self) -> Result<String> {
        self.inner.read_string_from_offset(72)
    }

    /// The spell's display name (localization code).
    pub fn display_name(&self) -> Result<String> {
        self.inner.read_string_from_offset(104)
    }

    /// The spell's description text.
    pub fn description(&self) -> Result<String> {
        self.inner.read_string_from_offset(168)
    }

    /// The spell's type name.
    pub fn type_name(&self) -> Result<String> {
        self.inner.read_string_from_offset(200)
    }
}

/// A spell rank entry (pip cost information).
#[derive(Clone)]
pub struct DynamicSpellRank {
    pub inner: DynamicMemoryObject,
}

impl DynamicSpellRank {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn regular_pips(&self) -> Result<i32> {
        self.inner.read_value_from_offset(72)
    }

    pub fn power_pips(&self) -> Result<i32> {
        self.inner.read_value_from_offset(76)
    }

    pub fn shadow_pips(&self) -> Result<i32> {
        self.inner.read_value_from_offset(80)
    }
}

// ── Legacy trait (kept for backward compatibility with memory objects) ──

pub trait Spell: MemoryObject {
    fn template_id(&self) -> u32 {
        self.read_value_from_offset(128).unwrap_or(0)
    }

    fn write_template_id(&self, template_id: u32) {
        let _ = self.write_value_to_offset(128, &template_id);
    }

    fn spell_id(&self) -> u32 {
        self.read_value_from_offset(204).unwrap_or(0)
    }

    fn write_spell_id(&self, spell_id: u32) {
        let _ = self.write_value_to_offset(204, &spell_id);
    }

    fn accuracy(&self) -> u8 {
        self.read_value_from_offset(132).unwrap_or(0)
    }

    fn write_accuracy(&self, accuracy: u8) {
        let _ = self.write_value_to_offset(132, &accuracy);
    }

    fn magic_school_id(&self) -> u32 {
        self.read_value_from_offset(136).unwrap_or(0)
    }

    fn write_magic_school_id(&self, magic_school_id: u32) {
        let _ = self.write_value_to_offset(136, &magic_school_id);
    }

    fn enchantment(&self) -> u32 {
        self.read_value_from_offset(80).unwrap_or(0)
    }

    fn write_enchantment(&self, enchantment: u32) {
        let _ = self.write_value_to_offset(80, &enchantment);
    }

    fn treasure_card(&self) -> bool {
        self.read_value_from_offset(197).unwrap_or(false)
    }

    fn cloaked(&self) -> bool {
        self.read_value_from_offset(196).unwrap_or(false)
    }

    fn item_card(&self) -> bool {
        self.read_value_from_offset(199).unwrap_or(false)
    }

    fn side_board(&self) -> bool {
        self.read_value_from_offset(200).unwrap_or(false)
    }

    fn pve(&self) -> bool {
        self.read_value_from_offset(264).unwrap_or(false)
    }
}
