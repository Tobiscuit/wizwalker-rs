use crate::memory::MemoryObject;

pub struct DynamicClientObject {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicClientObject {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

pub struct DynamicGameStats {
    pub hook_handler: i64,
    pub address: i64,
}

impl DynamicGameStats {
    pub fn new(hook_handler: i64, address: i64) -> Self {
        Self { hook_handler, address }
    }
}

#[async_trait::async_trait]
pub trait CombatParticipant: MemoryObject {
    async fn owner_id_full(&self) -> i32 {
        self.read_value_from_offset(72).await.unwrap_or(0)
    }

    async fn write_owner_id_full(&self, owner_id_full: i32) {
        let _ = self.write_value_to_offset(72, &owner_id_full).await;
    }

    async fn health(&self) -> i32 {
        self.read_value_from_offset(76).await.unwrap_or(0)
    }

    async fn write_health(&self, health: i32) {
        let _ = self.write_value_to_offset(76, &health).await;
    }

    async fn max_health(&self) -> i32 {
        self.read_value_from_offset(80).await.unwrap_or(0)
    }

    async fn write_max_health(&self, max_health: i32) {
        let _ = self.write_value_to_offset(80, &max_health).await;
    }
}
