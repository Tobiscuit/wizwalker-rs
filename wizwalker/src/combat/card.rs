use crate::combat::handler::CombatHandler;
use crate::combat::member::CombatMember;
use crate::memory::objects::spell::DynamicGraphicalSpell;
use crate::memory::objects::spell_effect::DynamicSpellEffect;
use crate::memory::objects::window::DynamicWindow;
use crate::utils::maybe_wait_for_value_with_timeout;
use crate::errors::WizWalkerMemoryError;

use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub enum CardTarget<'a> {
    Card(&'a CombatCard),
    Member(&'a CombatMember),
    Cards(Vec<&'a CombatCard>),
    Members(Vec<&'a CombatMember>),
    None,
}

pub struct CombatCard {
    pub combat_handler: Arc<CombatHandler>,
    pub spell_window: DynamicWindow,
}

impl CombatCard {
    pub fn new(combat_handler: Arc<CombatHandler>, spell_window: DynamicWindow) -> Self {
        Self {
            combat_handler,
            spell_window,
        }
    }

    pub async fn cast(&self, target: CardTarget<'_>, sleep_time: Option<f32>, debug_paint: bool) -> Result<(), WizWalkerMemoryError> {
        match target {
            CardTarget::Card(target_card) => {
                let cards_len_before = self.combat_handler.get_cards().await?.len();

                self.combat_handler.client.mouse_handler.click_window(&self.spell_window, false).await?;

                if let Some(t) = sleep_time {
                    sleep(Duration::from_secs_f32(t)).await;
                }

                self.combat_handler.client.mouse_handler.set_mouse_position_to_window(&target_card.spell_window).await?;

                if let Some(t) = sleep_time {
                    sleep(Duration::from_secs_f32(t)).await;
                }

                if debug_paint {
                    target_card.spell_window.debug_paint().await?;
                }

                self.combat_handler.client.mouse_handler.click_window(&target_card.spell_window, false).await?;

                while self.combat_handler.get_cards().await?.len() > cards_len_before {
                    sleep(Duration::from_millis(100)).await;
                }

                if let Some(t) = sleep_time {
                    sleep(Duration::from_secs_f32(t)).await;
                }
            }
            CardTarget::None => {
                self.combat_handler.client.mouse_handler.click_window(&self.spell_window, false).await?;
            }
            CardTarget::Members(targets) => {
                self.combat_handler.client.mouse_handler.click_window(&self.spell_window, false).await?;

                if let Some(t) = sleep_time {
                    sleep(Duration::from_secs_f32(t)).await;
                }

                for t in targets {
                    let health_window = t.get_health_text_window().await?;
                    self.combat_handler.client.mouse_handler.click_window(&health_window, false).await?;
                    if let Some(t) = sleep_time {
                        sleep(Duration::from_secs_f32(t)).await;
                    }
                }

                let confirm_windows = self.combat_handler.client.root_window.get_windows_with_name("ConfirmTargetsWindow").await?;
                if !confirm_windows.is_empty() {
                    let confirm_window = &confirm_windows[0];
                    if confirm_window.is_visible().await? {
                        let _ = self.combat_handler.client.mouse_handler.click_window_with_name("ConfirmTargetsConfirm", false).await;
                    }
                }
            }
            CardTarget::Member(target_member) => {
                self.combat_handler.client.mouse_handler.click_window(&self.spell_window, false).await?;

                if let Some(t) = sleep_time {
                    sleep(Duration::from_secs_f32(t)).await;
                }

                let health_window = target_member.get_health_text_window().await?;
                self.combat_handler.client.mouse_handler.click_window(&health_window, false).await?;
            }
            CardTarget::Cards(_) => {
                // To be implemented if list of cards targeting is needed
            }
        }

        Ok(())
    }

    pub async fn discard(&self, sleep_time: Option<f32>) -> Result<(), WizWalkerMemoryError> {
        let cards_len_before = self.combat_handler.get_cards().await?.len();
        self.combat_handler.client.mouse_handler.click_window(&self.spell_window, true).await?;

        while self.combat_handler.get_cards().await?.len() > cards_len_before {
            sleep(Duration::from_millis(100)).await;
        }

        if let Some(t) = sleep_time {
            sleep(Duration::from_secs_f32(t)).await;
        }

        Ok(())
    }

    pub async fn get_graphical_spell(&self) -> Result<DynamicGraphicalSpell, WizWalkerMemoryError> {
        if let Some(res) = self.spell_window.maybe_graphical_spell().await? {
            Ok(res)
        } else {
            Err(WizWalkerMemoryError::Other("Graphical spell not found; probably reading too fast".to_string()))
        }
    }

    pub async fn wait_for_graphical_spell(&self, timeout: f32) -> Result<DynamicGraphicalSpell, WizWalkerMemoryError> {
        let spell_window = self.spell_window.clone();

        let res = maybe_wait_for_value_with_timeout(
            move || {
                let spell_window = spell_window.clone();
                Box::pin(async move { spell_window.maybe_graphical_spell().await })
            },
            true,
            Some(timeout),
        ).await?;

        if let Some(Some(spell)) = res {
            Ok(spell)
        } else {
            Err(WizWalkerMemoryError::Other("Timeout waiting for graphical spell".to_string()))
        }
    }

    pub async fn get_spell_effects(&self) -> Result<Vec<DynamicSpellEffect>, WizWalkerMemoryError> {
        let spell = self.wait_for_graphical_spell(2.0).await?;
        spell.spell_effects().await
    }

    pub async fn name(&self) -> Result<String, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        let spell_template = graphical_spell.spell_template().await?;
        spell_template.name().await
    }

    pub async fn display_name_code(&self) -> Result<String, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        let spell_template = graphical_spell.spell_template().await?;
        spell_template.display_name().await
    }

    pub async fn display_name(&self) -> Result<String, WizWalkerMemoryError> {
        let code = self.display_name_code().await?;
        self.combat_handler.client.cache_handler.get_langcode_name(&code).await
    }

    pub async fn type_name(&self) -> Result<String, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        let spell_template = graphical_spell.spell_template().await?;
        spell_template.type_name().await
    }

    pub async fn template_id(&self) -> Result<u32, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.template_id().await
    }

    pub async fn spell_id(&self) -> Result<u32, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.spell_id().await
    }

    pub async fn accuracy(&self) -> Result<u32, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.accuracy().await
    }

    pub async fn is_castable(&self) -> Result<bool, WizWalkerMemoryError> {
        Ok(!self.spell_window.maybe_spell_grayed().await?)
    }

    pub async fn is_enchanted(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        Ok(graphical_spell.enchantment().await? != 0)
    }

    pub async fn is_treasure_card(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.treasure_card().await
    }

    pub async fn is_item_card(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.item_card().await
    }

    pub async fn is_side_board(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.side_board().await
    }

    pub async fn is_cloaked(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.cloaked().await
    }

    pub async fn is_enchanted_from_item_card(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.enchantment_spell_is_item_card().await
    }

    pub async fn is_pve_only(&self) -> Result<bool, WizWalkerMemoryError> {
        let graphical_spell = self.wait_for_graphical_spell(2.0).await?;
        graphical_spell.pve().await
    }
}
