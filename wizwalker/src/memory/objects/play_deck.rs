use crate::memory::MemoryObject;

pub trait PlayDeck: MemoryObject {
    // async fn deck_to_save(&self) -> Vec<DynamicPlaySpellData> {
    //     let mut spell_data = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(72).await {
    //         for addr in addrs {
    //             spell_data.push(DynamicPlaySpellData::new(self.hook_handler(), addr));
    //         }
    //     }
    //     spell_data
    // }

    // async fn graveyard_to_save(&self) -> Vec<DynamicPlaySpellData> {
    //     let mut spell_data = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(96).await {
    //         for addr in addrs {
    //             spell_data.push(DynamicPlaySpellData::new(self.hook_handler(), addr));
    //         }
    //     }
    //     spell_data
    // }
}

pub trait PlaySpellData: MemoryObject {
    async fn template_id(&self) -> u32 {
        self.read_value_from_offset(72).await.unwrap_or(0)
    }

    async fn enchantment(&self) -> u32 {
        self.read_value_from_offset(76).await.unwrap_or(0)
    }
}
