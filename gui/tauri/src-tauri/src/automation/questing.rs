//! Auto-Questing — faithfully ported from Deimos `src/questing.py` (1425 lines).
//!
//! Provides the `Quester` struct and all supporting functions for automated
//! quest completion.
#![allow(dead_code, unused_imports)]

use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info};

use wizwalker::client::Client;
use wizwalker::constants::Keycode;
use wizwalker::types::XYZ;

use super::utils::{
    click_window_by_path, is_visible_by_path, text_from_path,
    is_free, get_quest_name, get_popup_title, exit_menus, read_popup_message,
};
use super::paths::*;

// ── Helper Functions ──────────────────────────────────────────────────

/// Check if a client is free for questing (not in combat, loading, dialogue).
pub fn is_free_leader_questing(client: &Client) -> bool {
    let in_loading = client.is_loading();
    let in_battle = client.in_battle();
    let in_dialogue = is_visible_by_path(client, ADVANCE_DIALOG);
    let dialogue_text = read_dialogue_text(client);

    !in_loading && !in_battle && !in_dialogue && dialogue_text.is_empty()
}

/// Read dialogue text from the UI.
pub fn read_dialogue_text(client: &Client) -> String {
    text_from_path(client, DIALOG_TEXT).unwrap_or_default()
}

/// Calculate distance between two XYZ points.
fn calc_distance(a: &XYZ, b: &XYZ) -> f32 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

// ── Quester Struct ──────────────────────────────────────────────────

/// Main auto-questing controller.
pub struct Quester<'a> {
    pub client: &'a Client,
    pub clients: Vec<&'a Client>,
    pub current_leader_idx: usize,
}

impl<'a> Quester<'a> {
    pub fn new(client: &'a Client, clients: Vec<&'a Client>) -> Self {
        Self { client, clients, current_leader_idx: 0 }
    }

    pub fn leader(&self) -> &Client {
        self.clients.get(self.current_leader_idx).copied().unwrap_or(self.client)
    }

    // ── Quest Text Reading ──────────────────────────────────────────

    pub fn read_quest_txt(&self, client: &Client) -> String {
        get_quest_name(client).unwrap_or_default()
    }

    pub fn read_spiral_door_title(&self, client: &Client) -> String {
        text_from_path(client, SPIRAL_DOOR_TITLE).unwrap_or_default()
    }

    pub fn read_popup(&self, client: &Client) -> String {
        read_popup_message(client)
    }

    pub fn get_truncated_quest_objectives(&self, client: &Client) -> String {
        let quest = self.read_quest_txt(client);
        if let Some(paren_idx) = quest.find('(') {
            quest[..paren_idx].trim().to_string()
        } else {
            quest
        }
    }

    pub fn get_quest_zone_name(&self, client: &Client) -> String {
        let query = self.read_quest_txt(client);
        let cleaned = query.replace("<center>", "").replace("</center>", "");
        if let Some(in_idx) = cleaned.find(" in ") {
            let after_in = &cleaned[in_idx + 4..];
            if let Some(paren_idx) = after_in.find('(') {
                after_in[..paren_idx].trim().to_string()
            } else {
                after_in.trim().to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn get_collect_quest_object_name(&self) -> String {
        let text = self.read_quest_txt(self.leader());
        let cleaned = text.replace("<center>", "").replace("</center>", "");
        let words: Vec<&str> = cleaned.split_whitespace().collect();
        if let Some(in_pos) = words.iter().position(|w| *w == "in") {
            if in_pos > 1 { words[1..in_pos].join(" ") } else { String::new() }
        } else {
            String::new()
        }
    }

    // ── Zone Management ─────────────────────────────────────────────

    pub fn followers_in_correct_zone(&self) -> bool {
        let leader_zone = self.leader().zone_name().unwrap_or_default();
        self.clients.iter().all(|c| c.zone_name().unwrap_or_default() == leader_zone)
    }

    pub fn get_follower_clients(&self) -> Vec<&'a Client> {
        let leader_pid = self.leader().process_id;
        self.clients.iter()
            .filter(|c| c.process_id != leader_pid)
            .copied()
            .collect()
    }

    pub async fn zone_recorrect_hub(&self) {
        if self.followers_in_correct_zone() { return; }
        for client in &self.clients {
            client.send_key(Keycode::End);
            client.send_key(Keycode::End);
            sleep(Duration::from_secs(3)).await;
            for _ in 0..100 {
                if !client.is_loading() { break; }
                sleep(Duration::from_millis(100)).await;
            }
        }
        sleep(Duration::from_secs(2)).await;
    }

    pub async fn x_press_zone_recorrect(&self) {
        let leader_pid = self.leader().process_id;
        for client in &self.clients {
            if client.process_id != leader_pid {
                client.send_key(Keycode::X);
                sleep(Duration::from_millis(2500)).await;
            }
        }
        sleep(Duration::from_secs(2)).await;
        for client in &self.clients {
            for _ in 0..100 {
                if !client.is_loading() { break; }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // ── NPC Interaction ─────────────────────────────────────────────

    pub async fn handle_npc_talking_quests(&self, talking_client: &Client, present_clients: &[&Client]) {
        for _ in 0..100 {
            if !is_free_leader_questing(talking_client) { break; }
            sleep(Duration::from_millis(100)).await;
        }

        let start = Instant::now();
        let timeout = Duration::from_secs(3);
        while start.elapsed() < timeout {
            if !is_free_leader_questing(talking_client) {
                debug!("Detected dialogue — waiting for it to end");
                for _ in 0..200 {
                    if is_free_leader_questing(talking_client) { break; }
                    sleep(Duration::from_millis(100)).await;
                }
                let after_talking_paths: &[&[&str]] = &[
                    EXIT_ZAFARIA_CLASS_PICTURE, EXIT_PET_LEVELED_UP, AVALON_BADGE_EXIT,
                ];
                for client in present_clients {
                    exit_menus(client, after_talking_paths).await;
                }
                sleep(Duration::from_millis(400)).await;
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    // ── Dungeon Handling ────────────────────────────────────────────

    pub async fn handle_dungeon_entry(&self) {
        for client in &self.clients {
            client.wait_for_loading_screen().await;
        }
        sleep(Duration::from_secs(1)).await;
    }

    pub async fn handle_dungeon_recall(&self) {
        if is_visible_by_path(self.leader(), DUNGEON_RECALL) {
            click_window_by_path(self.leader(), DUNGEON_RECALL).await;
            sleep(Duration::from_secs(2)).await;
            self.leader().wait_for_loading_screen().await;

            for client in self.get_follower_clients() {
                if is_visible_by_path(client, DUNGEON_RECALL) {
                    click_window_by_path(client, DUNGEON_RECALL).await;
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    // ── Spiral Door Navigation ──────────────────────────────────────

    pub async fn handle_spiral_navigation(&self) {
        if is_visible_by_path(self.leader(), SPIRAL_DOOR_TELEPORT) {
            for _ in 0..20 {
                if is_visible_by_path(self.leader(), SPIRAL_DOOR_SELECTED) {
                    click_window_by_path(self.leader(), SPIRAL_DOOR_TELEPORT).await;
                    sleep(Duration::from_secs(2)).await;
                    self.leader().wait_for_loading_screen().await;
                    return;
                }
                click_window_by_path(self.leader(), SPIRAL_DOOR_CYCLE).await;
                sleep(Duration::from_millis(300)).await;
            }
            click_window_by_path(self.leader(), SPIRAL_DOOR_TELEPORT).await;
            sleep(Duration::from_secs(2)).await;
            self.leader().wait_for_loading_screen().await;
        }
    }

    // ── Healing / Potions ───────────────────────────────────────────

    pub async fn heal_and_handle_potions(&self) {
        for client in &self.clients {
            if is_free(client) {
                click_window_by_path(client, POTION_USAGE).await;
                sleep(Duration::from_millis(600)).await;
            }
        }
    }

    // ── Quest Handling ──────────────────────────────────────────────

    pub async fn handle_normal_quests(&self) {
        for client in &self.clients {
            if is_visible_by_path(client, CANCEL_CHEST_ROLL) {
                click_window_by_path(client, CANCEL_CHEST_ROLL).await;
            }
            if is_visible_by_path(client, EXIT_DUNGEON) {
                click_window_by_path(client, EXIT_DUNGEON).await;
            }
        }

        for client in &self.clients {
            for _ in 0..200 {
                if is_free_leader_questing(client) { break; }
                sleep(Duration::from_millis(100)).await;
            }
        }

        if is_free_leader_questing(self.leader()) {
            for client in &self.clients {
                for _ in 0..100 {
                    if !client.is_loading() { break; }
                    sleep(Duration::from_millis(100)).await;
                }
            }

            if is_visible_by_path(self.leader(), NPC_RANGE) {
                let popup_msg = self.read_popup(self.leader()).to_lowercase();

                if popup_msg.contains("to enter") {
                    debug!("Entering dungeon");
                    for client in &self.clients {
                        client.send_key(Keycode::X);
                    }
                    sleep(Duration::from_secs(1)).await;
                    for client in &self.clients {
                        if is_visible_by_path(client, DUNGEON_WARNING) {
                            client.send_key(Keycode::Enter);
                        }
                    }
                    self.handle_dungeon_entry().await;
                } else if popup_msg.contains("to talk") {
                    debug!("Talking to NPC");
                    for client in &self.clients {
                        client.send_key(Keycode::X);
                    }
                    self.handle_npc_talking_quests(self.leader(), &self.clients).await;
                } else if popup_msg.contains("magic raft") || popup_msg.contains("to ride") || popup_msg.contains("to teleport") {
                    self.leader().send_key(Keycode::X);
                    sleep(Duration::from_secs(1)).await;
                    for client in &self.clients {
                        client.send_key(Keycode::X);
                    }
                } else {
                    for client in &self.clients {
                        client.send_key(Keycode::X);
                    }
                }

                sleep(Duration::from_secs(2)).await;
                for client in &self.clients {
                    for _ in 0..100 {
                        if !client.is_loading() { break; }
                        sleep(Duration::from_millis(100)).await;
                    }
                }

                let end_of_loop_paths: &[&[&str]] = &[
                    EXIT_RECIPE_SHOP, EXIT_EQUIPMENT_SHOP, CANCEL_MULTIPLE_QUEST_MENU,
                    CANCEL_SPELL_VENDOR, EXIT_SNACK_SHOP, EXIT_REAGENT_SHOP,
                    EXIT_TC_VENDOR, EXIT_MINIGAME_SIGIL, EXIT_WYSTERIA_TOURNAMENT,
                    EXIT_DUNGEON, EXIT_ZAFARIA_CLASS_PICTURE, EXIT_PET_LEVELED_UP,
                    AVALON_BADGE_EXIT, POTION_EXIT,
                ];
                for client in &self.clients {
                    exit_menus(client, end_of_loop_paths).await;
                }
                sleep(Duration::from_millis(750)).await;

                if is_visible_by_path(self.leader(), SPIRAL_DOOR_TELEPORT) {
                    self.handle_spiral_navigation().await;
                }
            } else {
                let quest_obj = self.read_quest_txt(self.leader());
                if quest_obj.contains("Photomance") {
                    for client in &self.clients {
                        client.send_key(Keycode::Z);
                        client.send_key(Keycode::Z);
                    }
                }
            }

            for client in &self.clients {
                if is_visible_by_path(client, MISSING_AREA) {
                    for _ in 0..100 {
                        if is_visible_by_path(client, MISSING_AREA_RETRY) { break; }
                        sleep(Duration::from_millis(100)).await;
                    }
                    click_window_by_path(client, MISSING_AREA_RETRY).await;
                }
            }
        }
        sleep(Duration::from_millis(700)).await;
    }

    // ── Teleport to Quest ───────────────────────────────────────────

    pub async fn teleport_to_quest(&self) {
        for client in &self.clients {
            for _ in 0..200 {
                if is_free_leader_questing(client) { break; }
                sleep(Duration::from_millis(100)).await;
            }
        }

        if is_free_leader_questing(self.leader()) {
            let quest_xyz = self.leader().quest_position().unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 });
            let leader_obj = self.get_truncated_quest_objectives(self.leader());

            if leader_obj.to_lowercase().contains("defeat") {
                debug!("Defeat quest — staggering teleports");
                let _ = self.leader().teleport(&quest_xyz);
                sleep(Duration::from_secs(1)).await;
                for client in self.get_follower_clients() {
                    let _ = client.teleport(&quest_xyz);
                }
            } else {
                for client in &self.clients {
                    let _ = client.teleport(&quest_xyz);
                }
            }
        }
    }

    // ── Zone Correction ─────────────────────────────────────────────

    pub async fn handle_zone_correction(&self) {
        if !self.followers_in_correct_zone() {
            debug!("Clients in wrong zone — X press correction");
            self.x_press_zone_recorrect().await;

            let paths: &[&[&str]] = &[
                EXIT_RECIPE_SHOP, EXIT_EQUIPMENT_SHOP, CANCEL_MULTIPLE_QUEST_MENU,
                CANCEL_SPELL_VENDOR, EXIT_SNACK_SHOP, EXIT_REAGENT_SHOP,
                EXIT_TC_VENDOR, EXIT_MINIGAME_SIGIL, EXIT_WYSTERIA_TOURNAMENT,
                EXIT_DUNGEON, EXIT_ZAFARIA_CLASS_PICTURE, EXIT_PET_LEVELED_UP,
                AVALON_BADGE_EXIT, POTION_EXIT,
            ];
            for client in &self.clients {
                exit_menus(client, paths).await;
            }
        }
        if !self.followers_in_correct_zone() {
            self.zone_recorrect_hub().await;
        }
    }

    // ── Zone Change ─────────────────────────────────────────────────

    pub async fn handle_questing_zone_change(&self) {
        if is_visible_by_path(self.client, EXIT_DUNGEON) {
            sleep(Duration::from_secs(1)).await;
            click_window_by_path(self.client, EXIT_DUNGEON).await;
            self.client.wait_for_loading_screen().await;
            sleep(Duration::from_secs(1)).await;
        } else {
            for _ in 0..100 {
                if !self.client.is_loading() { break; }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // ── Main Quest Loops ────────────────────────────────────────────

    /// Solo auto-quest loop — single client.
    pub async fn auto_quest_solo(&self) {
        if !is_free(self.client) { return; }
        click_window_by_path(self.client, POTION_USAGE).await;

        let quest_xyz = self.client.quest_position().unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 });
        let distance = calc_distance(&quest_xyz, &XYZ { x: 0.0, y: 0.0, z: 0.0 });

        if distance > 1.0 {
            for _ in 0..200 {
                if !self.client.in_battle() { break; }
                sleep(Duration::from_millis(100)).await;
            }
            let _ = self.client.teleport(&quest_xyz);
            self.handle_questing_zone_change().await;
            sleep(Duration::from_millis(500)).await;

            if is_visible_by_path(self.client, CANCEL_CHEST_ROLL) {
                click_window_by_path(self.client, CANCEL_CHEST_ROLL).await;
            }

            if is_visible_by_path(self.client, NPC_RANGE) {
                let popup_msg = read_popup_message(self.client).to_lowercase();
                if popup_msg.contains("to enter") {
                    for client in &self.clients {
                        client.send_key(Keycode::X);
                    }
                    for client in &self.clients {
                        for _ in 0..100 {
                            if client.is_loading() { break; }
                            if is_visible_by_path(client, DUNGEON_WARNING) {
                                client.send_key(Keycode::Enter);
                            }
                            sleep(Duration::from_millis(100)).await;
                        }
                        for _ in 0..100 {
                            if !client.is_loading() { break; }
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                } else if popup_msg.contains("to talk") {
                    debug!("Talking to NPC");
                    self.client.send_key(Keycode::X);
                    self.handle_npc_talking_quests(self.client, &[self.client]).await;
                } else {
                    self.client.send_key(Keycode::X);
                    sleep(Duration::from_millis(750)).await;
                    if is_visible_by_path(self.client, SPIRAL_DOOR_TELEPORT) {
                        self.handle_spiral_navigation().await;
                    }
                }
            }

            let quest_obj = self.read_quest_txt(self.client);
            if quest_obj.contains("Photomance") {
                self.client.send_key(Keycode::Z);
                self.client.send_key(Keycode::Z);
            }

            if is_visible_by_path(self.client, MISSING_AREA) {
                for _ in 0..100 {
                    if is_visible_by_path(self.client, MISSING_AREA_RETRY) { break; }
                    sleep(Duration::from_millis(100)).await;
                }
                click_window_by_path(self.client, MISSING_AREA_RETRY).await;
            }
        } else {
            sleep(Duration::from_secs(3)).await;
            let quest_xyz = self.client.quest_position().unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 });
            let distance = calc_distance(&quest_xyz, &XYZ { x: 0.0, y: 0.0, z: 0.0 });
            if distance < 1.0 {
                debug!("Collect quest detected — entity collection (needs navmap)");
            }
        }
    }

    /// Leader-based auto-quest loop (multi-client).
    pub async fn auto_quest_leader(&self) {
        let mut iterations: u32 = 0;
        let mut last_quest = self.read_quest_txt(self.leader());
        let mut last_zone = self.leader().zone_name().unwrap_or_default();

        info!("Starting auto-quest leader loop");

        loop {
            sleep(Duration::from_millis(400)).await;

            for client in &self.clients {
                for _ in 0..200 {
                    if is_free_leader_questing(client) { break; }
                    sleep(Duration::from_millis(100)).await;
                }
            }

            self.heal_and_handle_potions().await;
            self.handle_dungeon_recall().await;
            self.handle_zone_correction().await;

            if is_free_leader_questing(self.leader()) {
                let quest_xyz = self.leader().quest_position().unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 });
                let distance = calc_distance(&quest_xyz, &XYZ { x: 0.0, y: 0.0, z: 0.0 });

                if distance > 1.0 {
                    if iterations >= 5 {
                        debug!("Quest stuck — recovery teleport");
                        if let Some(pos) = self.leader().body_position() {
                            for client in &self.clients {
                                let _ = client.teleport(&XYZ { x: pos.x + 500.0, y: pos.y, z: pos.z - 1500.0 });
                            }
                            sleep(Duration::from_secs(2)).await;
                        }
                    }
                    self.teleport_to_quest().await;
                    self.handle_normal_quests().await;
                } else {
                    sleep(Duration::from_secs(3)).await;
                    let quest_xyz = self.leader().quest_position().unwrap_or(XYZ { x: 0.0, y: 0.0, z: 0.0 });
                    if calc_distance(&quest_xyz, &XYZ { x: 0.0, y: 0.0, z: 0.0 }) < 1.0 {
                        debug!("Collect quest detected");
                    }
                }
            }

            let current_quest = self.read_quest_txt(self.leader());
            let current_zone = self.leader().zone_name().unwrap_or_default();
            if current_quest == last_quest && current_zone == last_zone {
                iterations += 1;
            } else {
                last_quest = current_quest;
                last_zone = current_zone;
                iterations = 0;
            }
        }
    }

    /// Simple auto-quest loop for a single client.
    pub async fn auto_quest(&self) {
        loop {
            sleep(Duration::from_secs(1)).await;
            self.auto_quest_solo().await;
        }
    }
}
