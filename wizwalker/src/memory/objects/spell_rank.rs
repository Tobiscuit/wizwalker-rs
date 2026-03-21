use crate::memory::MemoryObject;

pub trait SpellRank: MemoryObject {
    fn spell_rank(&self) -> u8 {
        self.read_value_from_offset(80).unwrap_or(0)
    }

    fn write_spell_rank(&self, spell_rank: u8) {
        let _ = self.write_value_to_offset(80, &spell_rank);
    }

    fn balance_pips(&self) -> u8 {
        self.read_value_from_offset(81).unwrap_or(0)
    }

    fn write_balance_pips(&self, balance_pips: u8) {
        let _ = self.write_value_to_offset(81, &balance_pips);
    }

    fn death_pips(&self) -> u8 {
        self.read_value_from_offset(82).unwrap_or(0)
    }

    fn write_death_pips(&self, death_pips: u8) {
        let _ = self.write_value_to_offset(82, &death_pips);
    }

    fn fire_pips(&self) -> u8 {
        self.read_value_from_offset(83).unwrap_or(0)
    }

    fn write_fire_pips(&self, fire_pips: u8) {
        let _ = self.write_value_to_offset(83, &fire_pips);
    }

    fn ice_pips(&self) -> u8 {
        self.read_value_from_offset(84).unwrap_or(0)
    }

    fn write_ice_pips(&self, ice_pips: u8) {
        let _ = self.write_value_to_offset(84, &ice_pips);
    }

    fn life_pips(&self) -> u8 {
        self.read_value_from_offset(85).unwrap_or(0)
    }

    fn write_life_pips(&self, life_pips: u8) {
        let _ = self.write_value_to_offset(85, &life_pips);
    }

    fn myth_pips(&self) -> u8 {
        self.read_value_from_offset(86).unwrap_or(0)
    }

    fn write_myth_pips(&self, myth_pips: u8) {
        let _ = self.write_value_to_offset(86, &myth_pips);
    }

    fn storm_pips(&self) -> u8 {
        self.read_value_from_offset(87).unwrap_or(0)
    }

    fn write_storm_pips(&self, storm_pips: u8) {
        let _ = self.write_value_to_offset(87, &storm_pips);
    }

    fn shadow_pips(&self) -> u8 {
        self.read_value_from_offset(88).unwrap_or(0)
    }

    fn write_shadow_pips(&self, shadow_pips: u8) {
        let _ = self.write_value_to_offset(88, &shadow_pips);
    }

    fn is_xpip_spell(&self) -> bool {
        self.read_value_from_offset(90).unwrap_or(false)
    }

    fn write_is_xpip_spell(&self, is_xpip: bool) {
        let _ = self.write_value_to_offset(90, &is_xpip);
    }
}
