use crate::memory::MemoryObject;

pub trait PlayDeck: MemoryObject {
    // fn deck_to_save(&self) -> Vec<DynamicPlaySpellData> {
    //     let mut spell_data = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(72) {
    //         for addr in addrs {
    //             spell_data.push(DynamicPlaySpellData::new(addr));
    //         }
    //     }
    //     spell_data
    // }

    // fn graveyard_to_save(&self) -> Vec<DynamicPlaySpellData> {
    //     let mut spell_data = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(96) {
    //         for addr in addrs {
    //             spell_data.push(DynamicPlaySpellData::new(addr));
    //         }
    //     }
    //     spell_data
    // }
}

pub trait PlaySpellData: MemoryObject {
    fn template_id(&self) -> u32 {
        self.read_value_from_offset(72).unwrap_or(0)
    }

    fn enchantment(&self) -> u32 {
        self.read_value_from_offset(76).unwrap_or(0)
    }
}
