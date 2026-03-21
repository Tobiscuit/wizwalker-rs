use crate::memory::MemoryObject;

pub trait PipCount: MemoryObject {
    fn generic_pips(&self) -> u8 {
        self.read_value_from_offset(80).unwrap_or(0)
    }

    fn write_generic_pips(&self, generic_pips: u8) {
        let _ = self.write_value_to_offset(80, &generic_pips);
    }

    fn power_pips(&self) -> u8 {
        self.read_value_from_offset(81).unwrap_or(0)
    }

    fn write_power_pips(&self, power_pips: u8) {
        let _ = self.write_value_to_offset(81, &power_pips);
    }

    fn balance_pips(&self) -> u8 {
        self.read_value_from_offset(82).unwrap_or(0)
    }

    fn write_balance_pips(&self, balance_pips: u8) {
        let _ = self.write_value_to_offset(82, &balance_pips);
    }

    fn death_pips(&self) -> u8 {
        self.read_value_from_offset(83).unwrap_or(0)
    }

    fn write_death_pips(&self, death_pips: u8) {
        let _ = self.write_value_to_offset(83, &death_pips);
    }

    fn fire_pips(&self) -> u8 {
        self.read_value_from_offset(84).unwrap_or(0)
    }

    fn write_fire_pips(&self, fire_pips: u8) {
        let _ = self.write_value_to_offset(84, &fire_pips);
    }

    fn ice_pips(&self) -> u8 {
        self.read_value_from_offset(85).unwrap_or(0)
    }

    fn write_ice_pips(&self, ice_pips: u8) {
        let _ = self.write_value_to_offset(85, &ice_pips);
    }

    fn life_pips(&self) -> u8 {
        self.read_value_from_offset(86).unwrap_or(0)
    }

    fn write_life_pips(&self, life_pips: u8) {
        let _ = self.write_value_to_offset(86, &life_pips);
    }

    fn myth_pips(&self) -> u8 {
        self.read_value_from_offset(87).unwrap_or(0)
    }

    fn write_myth_pips(&self, myth_pips: u8) {
        let _ = self.write_value_to_offset(87, &myth_pips);
    }

    fn storm_pips(&self) -> u8 {
        self.read_value_from_offset(88).unwrap_or(0)
    }

    fn write_storm_pips(&self, storm_pips: u8) {
        let _ = self.write_value_to_offset(88, &storm_pips);
    }

    fn shadow_pips(&self) -> u8 {
        self.read_value_from_offset(89).unwrap_or(0)
    }

    fn write_shadow_pips(&self, shadow_pips: u8) {
        let _ = self.write_value_to_offset(89, &shadow_pips);
    }
}
