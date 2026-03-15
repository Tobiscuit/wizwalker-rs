use crate::combat::handler::CombatHandler;
use crate::memory::objects::window::DynamicWindow;
use crate::memory::objects::combat_participant::CombatParticipant;
use crate::memory::objects::game_stats::DynamicGameStats;
use crate::utils::maybe_wait_for_any_value_with_timeout;
use crate::errors::{WizWalkerMemoryError, MemoryInvalidated};

use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct CombatMember {
    pub combat_handler: Arc<CombatHandler>,
    pub combatant_control: DynamicWindow,
}

impl CombatMember {
    pub fn new(combat_handler: Arc<CombatHandler>, combatant_control: DynamicWindow) -> Self {
        Self {
            combat_handler,
            combatant_control,
        }
    }

    pub async fn get_participant(&self) -> Result<CombatParticipant, WizWalkerMemoryError> {
        let part = self.combatant_control.maybe_combat_participant().await?;
        if let Some(part) = part {
            Ok(part)
        } else {
            Err(WizWalkerMemoryError::MemoryInvalidated(MemoryInvalidated("This combat member is no longer valid; you most likely need to reget members".to_string())))
        }
    }

    pub async fn get_stats(&self) -> Result<DynamicGameStats, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.game_stats().await
    }

    pub async fn get_health_text_window(&self) -> Result<DynamicWindow, WizWalkerMemoryError> {
        let combatant_control = self.combatant_control.clone();
        let possible = maybe_wait_for_any_value_with_timeout(
            move || {
                let combatant_control = combatant_control.clone();
                Box::pin(async move {
                    let result = combatant_control.get_windows_with_name("Health").await?;
                    if result.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(result))
                    }
                })
            },
            5.0,
        ).await?;

        if let Some(possible) = possible {
            if !possible.is_empty() {
                return Ok(possible[0].clone());
            }
        }

        Err(WizWalkerMemoryError::Other("Couldn't find health child".to_string()))
    }

    pub async fn get_name_text_window(&self) -> Result<DynamicWindow, WizWalkerMemoryError> {
        let possible = self.combatant_control.get_windows_with_name("Name").await?;
        if !possible.is_empty() {
            return Ok(possible[0].clone());
        }

        Err(WizWalkerMemoryError::Other("Couldn't find name child".to_string()))
    }

    pub async fn is_dead(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        let stats = part.game_stats().await?;
        Ok(stats.current_hitpoints().await? == 0)
    }

    pub async fn is_client(&self) -> Result<bool, WizWalkerMemoryError> {
        let owner_id = self.owner_id().await?;
        let global_id = self.combat_handler.client.client_object.global_id_full().await?;
        Ok(owner_id == global_id)
    }

    pub async fn is_player(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.is_player().await
    }

    pub async fn is_monster(&self) -> Result<bool, WizWalkerMemoryError> {
        Ok(!self.is_player().await? && !self.is_minion().await?)
    }

    pub async fn is_minion(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.is_minion().await
    }

    pub async fn is_boss(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.boss_mob().await
    }

    pub async fn is_stunned(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        Ok(part.stunned().await? != 0)
    }

    pub async fn name(&self) -> Result<String, WizWalkerMemoryError> {
        let name_window = self.get_name_text_window().await?;
        name_window.maybe_text().await
    }

    pub async fn owner_id(&self) -> Result<u64, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.owner_id_full().await
    }

    pub async fn template_id(&self) -> Result<u32, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.template_id_full().await
    }

    pub async fn normal_pips(&self) -> Result<u32, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.num_pips().await
    }

    pub async fn power_pips(&self) -> Result<u32, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.num_power_pips().await
    }

    pub async fn shadow_pips(&self) -> Result<u32, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.num_shadow_pips().await
    }

    pub async fn health(&self) -> Result<u32, WizWalkerMemoryError> {
        let part = self.get_participant().await?;
        part.player_health().await
    }

    pub async fn max_health(&self) -> Result<u32, WizWalkerMemoryError> {
        let stats = self.get_stats().await?;
        stats.max_hitpoints().await
    }

    pub async fn mana(&self) -> Result<u32, WizWalkerMemoryError> {
        let stats = self.get_stats().await?;
        stats.current_mana().await
    }

    pub async fn max_mana(&self) -> Result<u32, WizWalkerMemoryError> {
        let stats = self.get_stats().await?;
        stats.max_mana().await
    }

    pub async fn level(&self) -> Result<u32, WizWalkerMemoryError> {
        let stats = self.get_stats().await?;
        stats.reference_level().await
    }
}
