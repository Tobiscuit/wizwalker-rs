use crate::memory::{MemoryObject, MemoryObjectExt};

pub trait SpellTemplate: MemoryObject {
    fn name(&self) -> String {
        self.read_string_from_offset(96).unwrap_or_default()
    }

    fn write_name(&self, name: String) {
        let _ = self.write_string_to_offset(96, &name);
    }

    fn description(&self) -> String {
        self.read_string_from_offset(168).unwrap_or_default()
    }

    fn write_description(&self, description: String) {
        let _ = self.write_string_to_offset(168, &description);
    }

    fn advanced_description(&self) -> String {
        self.read_string_from_offset(200).unwrap_or_default()
    }

    fn write_advanced_description(&self, advanced_description: String) {
        let _ = self.write_string_to_offset(200, &advanced_description);
    }

    fn display_name(&self) -> String {
        self.read_string_from_offset(136).unwrap_or_default()
    }

    fn write_display_name(&self, display_name: String) {
        let _ = self.write_string_to_offset(136, &display_name);
    }

    fn spell_base(&self) -> String {
        self.read_string_from_offset(248).unwrap_or_default()
    }

    fn write_spell_base(&self, spell_base: String) {
        let _ = self.write_string_to_offset(248, &spell_base);
    }
}
