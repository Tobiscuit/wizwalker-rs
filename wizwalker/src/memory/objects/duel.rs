use crate::memory::MemoryObject;
use super::combat_participant::CombatParticipant;
use super::combat_resolver::CombatResolver;

pub trait Duel: MemoryObject {
    // async fn participant_list(&self) -> Vec<DynamicCombatParticipant> {
    //     let mut participants = Vec::new();
    //     if let Ok(addrs) = self.read_shared_vector(80).await {
    //         for addr in addrs {
    //             participants.push(DynamicCombatParticipant::new(self.hook_handler(), addr));
    //         }
    //     }
    //     participants
    // }

    async fn dynamic_turn(&self) -> u32 {
        self.read_value_from_offset(120).await.unwrap_or(0)
    }

    async fn write_dynamic_turn(&self, dynamic_turn: u32) {
        let _ = self.write_value_to_offset(120, &dynamic_turn).await;
    }

    async fn dynamic_turn_subcircles(&self) -> u32 {
        self.read_value_from_offset(124).await.unwrap_or(0)
    }

    async fn write_dynamic_turn_subcircle(&self, dynamic_turn_subcircle: u32) {
        let _ = self.write_value_to_offset(124, &dynamic_turn_subcircle).await;
    }

    async fn dynamic_turn_counter(&self) -> i32 {
        self.read_value_from_offset(128).await.unwrap_or(0)
    }

    async fn write_dynamic_turn_counter(&self, dynamic_turn_counter: i32) {
        let _ = self.write_value_to_offset(128, &dynamic_turn_counter).await;
    }

    async fn duel_id_full(&self) -> i32 {
        self.read_value_from_offset(216).await.unwrap_or(0)
    }

    async fn write_duel_id_full(&self, duel_id_full: i32) {
        let _ = self.write_value_to_offset(216, &duel_id_full).await;
    }

    async fn planning_timer(&self) -> f32 {
        self.read_value_from_offset(220).await.unwrap_or(0.0)
    }

    async fn write_planning_timer(&self, planning_timer: f32) {
        let _ = self.write_value_to_offset(220, &planning_timer).await;
    }
}
