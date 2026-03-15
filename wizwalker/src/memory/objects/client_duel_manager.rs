use std::collections::HashMap;
use crate::memory::MemoryObject;
use super::duel::Duel;

pub trait ClientDuelManager: MemoryObject {
    // async fn duelmap(&self) -> HashMap<i32, DynamicDuel> {
    //     self.read_std_map(8).await.unwrap_or_default()
    // }
}
