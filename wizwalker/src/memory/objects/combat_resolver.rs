use crate::memory::MemoryObject;
use super::spell_effect::SpellEffect;

pub trait CombatResolver: MemoryObject {
    async fn bool_global_effect(&self) -> bool {
        self.read_value_from_offset(112).await.unwrap_or(false)
    }

    async fn write_bool_global_effect(&self, bool_global_effect: bool) {
        let _ = self.write_value_to_offset(112, &bool_global_effect).await;
    }

    // async fn global_effect(&self) -> Option<DynamicSpellEffect> {
    //     if let Ok(addr) = self.read_value_from_offset::<i64>(120).await {
    //         if addr != 0 {
    //             return Some(DynamicSpellEffect::new(self.hook_handler(), addr));
    //         }
    //     }
    //     None
    // }

    // async fn battlefield_effects(&self) -> Vec<DynamicSpellEffect> {
    //     let mut effects = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(136).await {
    //         for addr in addrs {
    //             effects.push(DynamicSpellEffect::new(self.hook_handler(), addr));
    //         }
    //     }
    //     effects
    // }
}
