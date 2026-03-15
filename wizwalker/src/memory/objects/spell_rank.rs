use crate::memory::MemoryObject;

#[async_trait::async_trait]
pub trait SpellRank: MemoryObject {
    async fn spell_rank(&self) -> u8 {
        self.read_value_from_offset(80).await.unwrap_or(0)
    }

    async fn write_spell_rank(&self, spell_rank: u8) {
        let _ = self.write_value_to_offset(80, &spell_rank).await;
    }

    async fn balance_pips(&self) -> u8 {
        self.read_value_from_offset(81).await.unwrap_or(0)
    }

    async fn write_balance_pips(&self, balance_pips: u8) {
        let _ = self.write_value_to_offset(81, &balance_pips).await;
    }

    async fn death_pips(&self) -> u8 {
        self.read_value_from_offset(82).await.unwrap_or(0)
    }

    async fn write_death_pips(&self, death_pips: u8) {
        let _ = self.write_value_to_offset(82, &death_pips).await;
    }

    async fn fire_pips(&self) -> u8 {
        self.read_value_from_offset(83).await.unwrap_or(0)
    }

    async fn write_fire_pips(&self, fire_pips: u8) {
        let _ = self.write_value_to_offset(83, &fire_pips).await;
    }

    async fn ice_pips(&self) -> u8 {
        self.read_value_from_offset(84).await.unwrap_or(0)
    }

    async fn write_ice_pips(&self, ice_pips: u8) {
        let _ = self.write_value_to_offset(84, &ice_pips).await;
    }

    async fn life_pips(&self) -> u8 {
        self.read_value_from_offset(85).await.unwrap_or(0)
    }

    async fn write_life_pips(&self, life_pips: u8) {
        let _ = self.write_value_to_offset(85, &life_pips).await;
    }

    async fn myth_pips(&self) -> u8 {
        self.read_value_from_offset(86).await.unwrap_or(0)
    }

    async fn write_myth_pips(&self, myth_pips: u8) {
        let _ = self.write_value_to_offset(86, &myth_pips).await;
    }

    async fn storm_pips(&self) -> u8 {
        self.read_value_from_offset(87).await.unwrap_or(0)
    }

    async fn write_storm_pips(&self, storm_pips: u8) {
        let _ = self.write_value_to_offset(87, &storm_pips).await;
    }

    async fn shadow_pips(&self) -> u8 {
        self.read_value_from_offset(88).await.unwrap_or(0)
    }

    async fn write_shadow_pips(&self, shadow_pips: u8) {
        let _ = self.write_value_to_offset(88, &shadow_pips).await;
    }

    async fn is_xpip_spell(&self) -> bool {
        self.read_value_from_offset(90).await.unwrap_or(false)
    }

    async fn write_is_xpip_spell(&self, is_xpip: bool) {
        let _ = self.write_value_to_offset(90, &is_xpip).await;
    }
}
