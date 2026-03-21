use crate::memory::MemoryObject;

pub trait CombatResolver: MemoryObject {
    fn bool_global_effect(&self) -> bool {
        self.read_value_from_offset(112).unwrap_or(false)
    }

    fn write_bool_global_effect(&self, bool_global_effect: bool) {
        let _ = self.write_value_to_offset(112, &bool_global_effect);
    }

    // fn global_effect(&self) -> Option<DynamicSpellEffect> {
    //     if let Ok(addr) = self.read_value_from_offset::<i64>(120) {
    //         if addr != 0 {
    //             return Some(DynamicSpellEffect::new(addr));
    //         }
    //     }
    //     None
    // }

    // fn battlefield_effects(&self) -> Vec<DynamicSpellEffect> {
    //     let mut effects = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(136) {
    //         for addr in addrs {
    //             effects.push(DynamicSpellEffect::new(addr));
    //         }
    //     }
    //     effects
    // }
}
