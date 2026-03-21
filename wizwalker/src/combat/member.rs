use crate::combat::handler::CombatHandler;
use crate::memory::objects::window::DynamicWindow;
use crate::memory::objects::combat_participant::DynamicCombatParticipant;
use crate::memory::objects::game_stats::{DynamicGameStats, GameStats};
use crate::errors::WizWalkerError;

// Alias: Jules used WizWalkerMemoryError throughout — map it for compatibility.
type WizWalkerMemoryError = WizWalkerError;

use std::sync::Arc;

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

    pub fn get_participant(&self) -> Result<DynamicCombatParticipant, WizWalkerMemoryError> {
        let part = self.combatant_control.maybe_combat_participant()?;
        if let Some(part) = part {
            Ok(part)
        } else {
            Err(WizWalkerError::Other(
                "This combat member is no longer valid; you most likely need to reget members".to_string(),
            ))
        }
    }

    pub fn get_stats(&self) -> Result<DynamicGameStats, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.game_stats()?.ok_or_else(|| {
            WizWalkerError::Other("Game stats pointer is null".to_string())
        })
    }

    pub fn get_health_text_window(&self) -> Result<DynamicWindow, WizWalkerMemoryError> {
        let possible = self.combatant_control.get_windows_with_name("Health")?;
        if !possible.is_empty() {
            return Ok(possible[0].clone());
        }
        Err(WizWalkerError::Other("Couldn't find health child".to_string()))
    }

    pub fn get_name_text_window(&self) -> Result<DynamicWindow, WizWalkerMemoryError> {
        let possible = self.combatant_control.get_windows_with_name("Name")?;
        if !possible.is_empty() {
            return Ok(possible[0].clone());
        }
        Err(WizWalkerError::Other("Couldn't find name child".to_string()))
    }

    pub fn is_dead(&self) -> Result<bool, WizWalkerMemoryError> {
        let stats = self.get_stats()?;
        Ok(stats.current_hitpoints()? == 0)
    }

    pub fn is_client(&self) -> Result<bool, WizWalkerMemoryError> {
        let owner_id = self.owner_id()?;
        // TODO: Compare against the client's global ID once Client.client_object is implemented.
        // For now, a stub that always returns false.
        let _ = owner_id;
        Ok(false)
    }

    pub fn is_player(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.is_player()
    }

    pub fn is_monster(&self) -> Result<bool, WizWalkerMemoryError> {
        Ok(!self.is_player()? && !self.is_minion()?)
    }

    pub fn is_minion(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.is_minion()
    }

    pub fn is_boss(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.boss_mob()
    }

    pub fn is_stunned(&self) -> Result<bool, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        Ok(part.stunned()? != 0)
    }

    pub fn name(&self) -> Result<String, WizWalkerMemoryError> {
        let name_window = self.get_name_text_window()?;
        name_window.maybe_text()
    }

    pub fn owner_id(&self) -> Result<u64, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.owner_id_full()
    }

    pub fn template_id(&self) -> Result<u64, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.template_id_full()
    }

    pub fn normal_pips(&self) -> Result<i32, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.num_pips()
    }

    pub fn power_pips(&self) -> Result<i32, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.num_power_pips()
    }

    pub fn shadow_pips(&self) -> Result<i32, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.num_shadow_pips()
    }

    pub fn health(&self) -> Result<i32, WizWalkerMemoryError> {
        let part = self.get_participant()?;
        part.player_health()
    }

    pub fn max_health(&self) -> Result<i32, WizWalkerMemoryError> {
        let stats = self.get_stats()?;
        stats.max_hitpoints()
    }

    pub fn mana(&self) -> Result<i32, WizWalkerMemoryError> {
        let stats = self.get_stats()?;
        stats.current_mana()
    }

    pub fn max_mana(&self) -> Result<i32, WizWalkerMemoryError> {
        let stats = self.get_stats()?;
        stats.max_mana()
    }

    pub fn level(&self) -> Result<i32, WizWalkerMemoryError> {
        let stats = self.get_stats()?;
        stats.reference_level()
    }
}
