use crate::memory::MemoryObject;

pub trait PipCount: MemoryObject {
    async fn generic_pips(&self) -> u8 {
        self.read_value_from_offset(80).await.unwrap_or(0)
    }

    async fn write_generic_pips(&self, generic_pips: u8) {
        let _ = self.write_value_to_offset(80, &generic_pips).await;
    }

    async fn power_pips(&self) -> u8 {
        self.read_value_from_offset(81).await.unwrap_or(0)
    }

    async fn write_power_pips(&self, power_pips: u8) {
        let _ = self.write_value_to_offset(81, &power_pips).await;
    }

    async fn balance_pips(&self) -> u8 {
        self.read_value_from_offset(82).await.unwrap_or(0)
    }

    async fn write_balance_pips(&self, balance_pips: u8) {
        let _ = self.write_value_to_offset(82, &balance_pips).await;
    }

    async fn death_pips(&self) -> u8 {
        self.read_value_from_offset(83).await.unwrap_or(0)
    }

    async fn write_death_pips(&self, death_pips: u8) {
        let _ = self.write_value_to_offset(83, &death_pips).await;
    }

    async fn fire_pips(&self) -> u8 {
        self.read_value_from_offset(84).await.unwrap_or(0)
    }

    async fn write_fire_pips(&self, fire_pips: u8) {
        let _ = self.write_value_to_offset(84, &fire_pips).await;
    }

    async fn ice_pips(&self) -> u8 {
        self.read_value_from_offset(85).await.unwrap_or(0)
    }

    async fn write_ice_pips(&self, ice_pips: u8) {
        let _ = self.write_value_to_offset(85, &ice_pips).await;
    }

    async fn life_pips(&self) -> u8 {
        self.read_value_from_offset(86).await.unwrap_or(0)
    }

    async fn write_life_pips(&self, life_pips: u8) {
        let _ = self.write_value_to_offset(86, &life_pips).await;
    }

    async fn myth_pips(&self) -> u8 {
        self.read_value_from_offset(87).await.unwrap_or(0)
    }

    async fn write_myth_pips(&self, myth_pips: u8) {
        let _ = self.write_value_to_offset(87, &myth_pips).await;
    }

    async fn storm_pips(&self) -> u8 {
        self.read_value_from_offset(88).await.unwrap_or(0)
    }

    async fn write_storm_pips(&self, storm_pips: u8) {
        let _ = self.write_value_to_offset(88, &storm_pips).await;
    }

    async fn shadow_pips(&self) -> u8 {
        self.read_value_from_offset(89).await.unwrap_or(0)
    }

    async fn write_shadow_pips(&self, shadow_pips: u8) {
        let _ = self.write_value_to_offset(89, &shadow_pips).await;
    }
}
