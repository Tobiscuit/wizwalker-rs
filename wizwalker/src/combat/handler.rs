// I'll start the handler port
use crate::client::WizWalkerClient;
use crate::combat::card::CombatCard;
use crate::combat::member::CombatMember;
use crate::memory::objects::duel::{DuelPhase, DynamicDuel};
use crate::memory::objects::spell_effect::{EffectTarget, SpellEffects};
use crate::memory::objects::window::{DynamicWindow, WindowFlags};
use crate::utils::{maybe_wait_for_value_with_timeout, wait_for_value};
use crate::errors::{WizWalkerMemoryError, MemoryInvalidated, MemoryReadError, ReadingEnumFailed};

use std::sync::Arc;
use std::pin::Pin;
use tokio::time::{sleep, Duration};
use futures::future::Future;

pub struct CombatHandler {
    pub client: Arc<WizWalkerClient>,
    spell_check_boxes: tokio::sync::Mutex<Option<Vec<DynamicWindow>>>,
}

impl CombatHandler {
    pub fn new(client: Arc<WizWalkerClient>) -> Self {
        Self {
            client,
            spell_check_boxes: tokio::sync::Mutex::new(None),
        }
    }

    pub async fn handle_round(&self) -> Result<(), WizWalkerMemoryError> {
        unimplemented!()
    }

    pub async fn handle_combat(&self) -> Result<(), WizWalkerMemoryError> {
        while self.in_combat().await? {
            self.wait_for_planning_phase(0.5).await?;
            if let Ok(phase) = self.client.duel.duel_phase().await {
                if phase != DuelPhase::Planning {
                    break;
                }
            } else {
                break;
            }

            let round_number = self.round_number().await?;
            sleep(Duration::from_millis(200)).await;

            self.handle_round().await?;
            self.wait_until_next_round(round_number, 0.5).await?;
        }

        *self.spell_check_boxes.lock().await = None;
        Ok(())
    }

    pub async fn wait_for_planning_phase(&self, sleep_time: f32) -> Result<(), WizWalkerMemoryError> {
        loop {
            match self.client.duel.duel_phase().await {
                Ok(phase) if phase == DuelPhase::Planning || phase == DuelPhase::Ended => break,
                _ => sleep(Duration::from_secs_f32(sleep_time)).await,
            }
        }
        Ok(())
    }

    pub async fn wait_for_combat(&self, sleep_time: f32) -> Result<(), WizWalkerMemoryError> {
        loop {
            if let Ok(in_combat) = self.in_combat().await {
                if in_combat {
                    break;
                }
            }
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }

        self.handle_combat().await
    }

    pub async fn wait_until_next_round(&self, current_round: u32, sleep_time: f32) -> Result<(), WizWalkerMemoryError> {
        while self.in_combat().await? {
            if let Ok(new_round_number) = self.round_number().await {
                if new_round_number > current_round {
                    return Ok(());
                }
            }
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }
        Ok(())
    }

    pub async fn in_combat(&self) -> Result<bool, WizWalkerMemoryError> {
        self.client.in_battle().await
    }

    async fn get_card_windows(&self) -> Result<Vec<DynamicWindow>, WizWalkerMemoryError> {
        let mut cache = self.spell_check_boxes.lock().await;
        if let Some(ref windows) = *cache {
            return Ok(windows.clone());
        }

        let spell_checkbox_windows = self.client.root_window.get_windows_with_type("SpellCheckBox").await?;
        let mut filtered = Vec::new();
        for w in spell_checkbox_windows {
            if w.name().await? != "PetCard" {
                filtered.push(w);
            }
        }

        *cache = Some(filtered.clone());
        Ok(filtered)
    }

    pub async fn get_cards(self: &Arc<Self>) -> Result<Vec<CombatCard>, WizWalkerMemoryError> {
        let spell_checkbox_windows = self.get_card_windows().await?;
        let mut cards = Vec::new();

        for spell_checkbox in spell_checkbox_windows.into_iter().rev() {
            if spell_checkbox.flags().await?.contains(&WindowFlags::Visible) {
                cards.push(CombatCard::new(self.clone(), spell_checkbox));
            }
        }

        Ok(cards)
    }

    pub async fn get_members(self: &Arc<Self>) -> Result<Vec<CombatMember>, WizWalkerMemoryError> {
        let combatant_windows = self.client.root_window.get_windows_with_name("CombatantControl").await?;
        let mut members = Vec::new();

        for window in combatant_windows {
            members.push(CombatMember::new(self.clone(), window));
        }

        Ok(members)
    }

    pub async fn get_client_member(self: &Arc<Self>, retries: u32, sleep_time: f32) -> Result<CombatMember, WizWalkerMemoryError> {
        for _ in 0..retries {
            if let Ok(members) = self.get_members().await {
                for member in members {
                    if let Ok(is_client) = member.is_client().await {
                        if is_client {
                            return Ok(member);
                        }
                    }
                }
            }
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }
        Err(WizWalkerMemoryError::Other("Couldn't find client's CombatMember".to_string()))
    }

    pub async fn round_number(&self) -> Result<u32, WizWalkerMemoryError> {
        self.client.duel.round_num().await
    }

    pub async fn pass_button(&self) -> Result<(), WizWalkerMemoryError> {
        let pos_done_window = self.client.root_window.get_windows_with_name("DoneWindow").await?;
        if !pos_done_window.is_empty() {
            let done_window = &pos_done_window[0];
            if done_window.is_visible().await? {
                let pos_defeated_pass_button = done_window.get_windows_with_name("DefeatedPassButton").await?;
                if !pos_defeated_pass_button.is_empty() {
                    return self.client.mouse_handler.click_window(&pos_defeated_pass_button[0], false).await;
                }
            }
        }
        self.client.mouse_handler.click_window_with_name("Focus", false).await
    }
}

pub struct AoeHandler {
    handler: Arc<CombatHandler>,
}

impl AoeHandler {
    pub fn new(handler: Arc<CombatHandler>) -> Self {
        Self { handler }
    }

    pub async fn handle_combat(&self) -> Result<(), WizWalkerMemoryError> {
        while self.handler.in_combat().await? {
            self.handler.wait_for_planning_phase(0.5).await?;

            if let Ok(phase) = self.handler.client.duel.duel_phase().await {
                if phase == DuelPhase::Ended {
                    break;
                }
            }

            self.handle_round().await?;
            self.wait_for_non_planning_phase(0.5).await?;
        }

        *self.handler.spell_check_boxes.lock().await = None;
        Ok(())
    }

    async fn wait_for_non_planning_phase(&self, sleep_time: f32) -> Result<(), WizWalkerMemoryError> {
        loop {
            match self.handler.client.duel.duel_phase().await {
                Ok(phase) if phase != DuelPhase::Planning || phase == DuelPhase::Ended => break,
                _ => sleep(Duration::from_secs_f32(sleep_time)).await,
            }
        }
        Ok(())
    }

    pub async fn get_client_member(&self, retries: u32, sleep_time: f32) -> Result<CombatMember, WizWalkerMemoryError> {
        for _ in 0..retries {
            if let Ok(members) = self.handler.get_members().await {
                for member in members {
                    if let Ok(is_client) = member.is_client().await {
                        if is_client {
                            return Ok(member);
                        }
                    }
                }
            }
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }
        Err(WizWalkerMemoryError::Other("Couldn't find client's CombatMember".to_string()))
    }

    pub async fn handle_round(&self) -> Result<(), WizWalkerMemoryError> {
        let enchanted_aoes = self.handler.get_damaging_aoes(Some(true)).await?;
        if !enchanted_aoes.is_empty() {
            return enchanted_aoes[0].cast(crate::combat::card::CardTarget::None, Some(1.0), false).await;
        }

        let unenchanted_aoes = self.handler.get_damaging_aoes(Some(false)).await?;
        let enchants = self.handler.get_damage_enchants(true).await?;

        if !enchants.is_empty() && !unenchanted_aoes.is_empty() {
            enchants[0].cast(crate::combat::card::CardTarget::Card(&unenchanted_aoes[0]), Some(1.0), false).await?;
            let enchanted_aoes = self.handler.get_damaging_aoes(Some(true)).await?;
            if !enchanted_aoes.is_empty() {
                let to_cast = &enchanted_aoes[0];
                if to_cast.is_castable().await? {
                    to_cast.cast(crate::combat::card::CardTarget::None, Some(1.0), false).await?;
                } else {
                    self.handler.pass_button().await?;
                }
            } else {
                self.handler.pass_button().await?;
            }
        } else if enchants.is_empty() && !unenchanted_aoes.is_empty() {
            let to_cast = &unenchanted_aoes[0];
            if to_cast.is_castable().await? {
                to_cast.cast(crate::combat::card::CardTarget::None, Some(1.0), false).await?;
                return Ok(());
            }
        } else if !enchants.is_empty() && unenchanted_aoes.is_empty() {
            if self.handler.get_cards().await?.len() == 7 {
                enchants[0].discard(Some(1.0)).await?;
                return Ok(());
            } else {
                sleep(Duration::from_secs(1)).await;
                let aoes = self.handler.get_damaging_aoes(None).await?;
                if aoes.is_empty() {
                    return Err(WizWalkerMemoryError::Other("No hits in hand".to_string()));
                } else {
                    aoes[0].cast(crate::combat::card::CardTarget::None, Some(1.0), false).await?;
                    return Ok(());
                }
            }
        } else {
            self.handler.pass_button().await?;
            self.handler.pass_button().await?;
        }

        Ok(())
    }
}

impl CombatHandler {
    // Adding the rest of the methods

    // Using simple loop to bypass high-rank lifetime issues with async predicates
    pub async fn get_cards_with_name(self: &Arc<Self>, name: &str) -> Result<Vec<CombatCard>, WizWalkerMemoryError> {
        let cards = self.get_cards().await?;
        let mut matches = Vec::new();
        for card in cards {
            if let Ok(card_name) = card.name().await {
                if card_name.to_lowercase() == name.to_lowercase() {
                    matches.push(card);
                }
            }
        }
        Ok(matches)
    }

    pub async fn get_card_named(self: &Arc<Self>, name: &str) -> Result<CombatCard, WizWalkerMemoryError> {
        let mut possible = self.get_cards_with_name(name).await?;
        if !possible.is_empty() {
            Ok(possible.remove(0))
        } else {
            Err(WizWalkerMemoryError::Other(format!("Couldn't find a card named {}", name)))
        }
    }

    pub async fn get_cards_with_display_name(self: &Arc<Self>, display_name: &str) -> Result<Vec<CombatCard>, WizWalkerMemoryError> {
        let cards = self.get_cards().await?;
        let mut matches = Vec::new();
        for card in cards {
            if let Ok(card_name) = card.display_name().await {
                if card_name.to_lowercase().contains(&display_name.to_lowercase()) {
                    matches.push(card);
                }
            }
        }
        Ok(matches)
    }

    pub async fn get_card_with_display_name(self: &Arc<Self>, display_name: &str) -> Result<CombatCard, WizWalkerMemoryError> {
        let mut possible = self.get_cards_with_display_name(display_name).await?;
        if !possible.is_empty() {
            Ok(possible.remove(0))
        } else {
            Err(WizWalkerMemoryError::Other(format!("Couldn't find a card display named {}", display_name)))
        }
    }

    pub async fn get_damaging_aoes(self: &Arc<Self>, check_enchanted: Option<bool>) -> Result<Vec<CombatCard>, WizWalkerMemoryError> {
        let cards = self.get_cards().await?;
        let mut matches = Vec::new();
        for card in cards {
            let mut is_match = true;
            if let Some(check) = check_enchanted {
                if let Ok(enchanted) = card.is_enchanted().await {
                    if check && !enchanted { is_match = false; }
                    if !check && enchanted { is_match = false; }
                }
            }
            if !is_match { continue; }

            if let Ok(type_name) = card.type_name().await {
                if type_name != "AOE" && type_name != "Steal" {
                    is_match = false;
                }
            } else {
                is_match = false;
            }
            if !is_match { continue; }

            let mut has_aoe_effect = false;
            if let Ok(effects) = card.get_spell_effects().await {
                for effect in effects {
                    if let Ok(effect_type) = effect.maybe_read_type_name().await {
                        if effect_type.to_lowercase().contains("variable") || effect_type.to_lowercase().contains("random") {
                            if let Ok(sub_effects) = effect.maybe_effect_list().await {
                                for sub_effect in sub_effects {
                                    if let Ok(target) = sub_effect.effect_target().await {
                                        if target == EffectTarget::EnemyTeam || target == EffectTarget::EnemyTeamAllAtOnce {
                                            has_aoe_effect = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            if let Ok(target) = effect.effect_target().await {
                                if target == EffectTarget::EnemyTeam || target == EffectTarget::EnemyTeamAllAtOnce {
                                    has_aoe_effect = true;
                                    break;
                                }
                            }
                        }
                    }
                    if has_aoe_effect { break; }
                }
            }
            if has_aoe_effect {
                matches.push(card);
            }
        }
        Ok(matches)
    }

    pub async fn get_damage_enchants(self: &Arc<Self>, sort_by_damage: bool) -> Result<Vec<CombatCard>, WizWalkerMemoryError> {
        let cards = self.get_cards().await?;
        let mut matches = Vec::new();
        for card in cards {
            if let Ok(type_name) = card.type_name().await {
                if type_name == "Enchantment" {
                    if let Ok(effects) = card.get_spell_effects().await {
                        for effect in effects {
                            if let Ok(et) = effect.effect_type().await {
                                if et == SpellEffects::ModifyCardDamage {
                                    matches.push(card);
                                    break; // Found matching effect, go to next card
                                }
                            }
                        }
                    }
                }
            }
        }

        if sort_by_damage {
            // Fetch damages for sorting
            let mut damages = Vec::new();
            for card in &matches {
                let mut dmg = 0;
                if let Ok(effects) = card.get_spell_effects().await {
                    if !effects.is_empty() {
                        if let Ok(d) = effects[0].effect_param().await {
                            dmg = d;
                        }
                    }
                }
                damages.push(dmg);
            }

            let mut paired: Vec<_> = matches.into_iter().zip(damages.into_iter()).collect();
            paired.sort_by(|a, b| a.1.cmp(&b.1));
            return Ok(paired.into_iter().map(|(card, _)| card).collect());
        }

        Ok(matches)
    }

    pub async fn get_all_monster_members(self: &Arc<Self>) -> Result<Vec<CombatMember>, WizWalkerMemoryError> {
        let members = self.get_members().await?;
        let mut matches = Vec::new();
        for member in members {
            if let Ok(is_monster) = member.is_monster().await {
                if is_monster {
                    matches.push(member);
                }
            }
        }
        Ok(matches)
    }

    pub async fn get_all_player_members(self: &Arc<Self>) -> Result<Vec<CombatMember>, WizWalkerMemoryError> {
        let members = self.get_members().await?;
        let mut matches = Vec::new();
        for member in members {
            if let Ok(is_player) = member.is_player().await {
                if is_player {
                    matches.push(member);
                }
            }
        }
        Ok(matches)
    }

    pub async fn get_member_named(self: &Arc<Self>, name: &str) -> Result<CombatMember, WizWalkerMemoryError> {
        let members = self.get_members().await?;
        for member in members {
            if let Ok(member_name) = member.name().await {
                if member_name.to_lowercase().contains(&name.to_lowercase()) {
                    return Ok(member);
                }
            }
        }
        Err(WizWalkerMemoryError::Other(format!("Couldn't find a member named {}", name)))
    }

    pub async fn attempt_cast(self: &Arc<Self>, name: &str, on_member: Option<&str>, on_card: Option<&str>, on_client: bool) -> Result<bool, WizWalkerMemoryError> {
        if let Ok(card) = self.get_card_named(name).await {
            if let Some(member_name) = on_member {
                if let Ok(target) = self.get_member_named(member_name).await {
                    card.cast(crate::combat::card::CardTarget::Member(&target), Some(1.0), false).await?;
                } else {
                    return Ok(false);
                }
            } else if let Some(card_name) = on_card {
                if let Ok(target) = self.get_card_named(card_name).await {
                    card.cast(crate::combat::card::CardTarget::Card(&target), Some(1.0), false).await?;
                } else {
                    return Ok(false);
                }
            } else if on_client {
                if let Ok(target) = self.get_client_member(5, 0.5).await {
                    card.cast(crate::combat::card::CardTarget::Member(&target), Some(1.0), false).await?;
                } else {
                    return Ok(false);
                }
            } else {
                card.cast(crate::combat::card::CardTarget::None, Some(1.0), false).await?;
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn attempt_willcast(self: &Arc<Self>, on_member: Option<&str>, on_client: bool) -> Result<bool, WizWalkerMemoryError> {
        let spell_checkbox_windows = self.client.root_window.get_windows_with_type("SpellCheckBox").await?;
        let mut pet_card_win = None;
        for w in spell_checkbox_windows {
            if w.name().await? == "PetCard" {
                pet_card_win = Some(w);
                break;
            }
        }

        if let Some(win) = pet_card_win {
            let card = CombatCard::new(self.clone(), win);
            if let Some(member_name) = on_member {
                if let Ok(target) = self.get_member_named(member_name).await {
                    card.cast(crate::combat::card::CardTarget::Member(&target), Some(1.0), false).await?;
                } else {
                    return Ok(false);
                }
            } else if on_client {
                if let Ok(target) = self.get_client_member(5, 0.5).await {
                    card.cast(crate::combat::card::CardTarget::Member(&target), Some(1.0), false).await?;
                } else {
                    return Ok(false);
                }
            } else {
                card.cast(crate::combat::card::CardTarget::None, Some(1.0), false).await?;
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn draw_button(&self) -> Result<(), WizWalkerMemoryError> {
        self.client.mouse_handler.click_window_with_name("Draw", false).await
    }

    pub async fn flee_button(&self) -> Result<(), WizWalkerMemoryError> {
        let pos_done_window = self.client.root_window.get_windows_with_name("DoneWindow").await?;
        if !pos_done_window.is_empty() {
            let done_window = &pos_done_window[0];
            if done_window.is_visible().await? {
                let pos_defeated_flee_button = done_window.get_windows_with_name("DefeatedFleeButton").await?;
                if !pos_defeated_flee_button.is_empty() {
                    return self.client.mouse_handler.click_window(&pos_defeated_flee_button[0], false).await;
                }
            }
        }
        self.client.mouse_handler.click_window_with_name("Flee", false).await
    }
}
