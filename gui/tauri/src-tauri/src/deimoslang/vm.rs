//! DeimosLang VM — Virtual Machine for executing scripts.
//!
//! Faithfully ported from `deimos-reference/src/deimoslang/vm.py`.
#![allow(dead_code, unused_imports, non_snake_case, unused_mut, unused_variables, unused_assignments, unreachable_patterns, unused_parens)]

use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tokio::time::{sleep, Duration};
use tokio::task::JoinSet;
use tracing::{debug, error, info, warn};
use regex::Regex;
use lazy_static::lazy_static;

use crate::deimoslang::ir::{Instruction, InstructionKind, InstructionData};
use crate::deimoslang::types::*;
use crate::deimoslang::tokenizer::TokenKind;
use crate::automation::config_combat::{delegate_combat_configs, DEFAULT_CONFIG};
use crate::automation::utils::{
    get_quest_name, click_window_by_path, get_window_from_path, is_free,
    refill_potions, refill_potions_if_needed, logout_and_in, teleport_to_friend_from_list,
    is_visible_by_path, text_from_path
};
use crate::automation::paths;
use wizwalker::memory::objects::GameStats as _;
use crate::automation::sprinty_client::SprintyClient;
use crate::automation::teleport_math::{navmap_tp, calc_distance, are_xyzs_within_threshold, calc_point_on_3d_line, rotate_point};
use crate::automation::deck_encoder::{encode_deck, decode_deck, Deck};
use crate::automation::auto_pet::{dancedance};
use crate::automation::dance_game_hook::{attempt_activate_dance_hook, attempt_deactivate_dance_hook, read_current_dance_game_moves};
use crate::automation::drop_logger::{get_chat, filter_drops, find_new_stuff};

use wizwalker::client::Client;
use wizwalker::types::XYZ;
use wizwalker::constants::Keycode;
use wizwalker::errors::{Result as WizResult, WizWalkerError};
use wizwalker::memory::objects::quest_data::QuestData;
use wizwalker::memory::objects::window::DynamicWindow;
use wizwalker::memory::memory_object::MemoryObject;
use wizwalker::memory::reader::MemoryReaderExt;

lazy_static! {
    static ref GOAL_REGEX: Regex = Regex::new(r"<[^>]*>").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeimosValue {
    None,
    Bool(bool),
    Number(f64),
    String(String),
    XYZ(XYZ),
    Keycode(Keycode),
    List(Vec<DeimosValue>),
}

impl DeimosValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            DeimosValue::None => false,
            DeimosValue::Bool(b) => *b,
            DeimosValue::Number(n) => *n != 0.0,
            DeimosValue::String(s) => !s.is_empty(),
            DeimosValue::XYZ(_) => true,
            DeimosValue::Keycode(_) => true,
            DeimosValue::List(l) => !l.is_empty(),
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            DeimosValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            DeimosValue::Number(n) => *n,
            DeimosValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DeimosValue::String(s) => s.clone(),
            DeimosValue::Number(n) => n.to_string(),
            DeimosValue::Bool(b) => b.to_string(),
            DeimosValue::XYZ(xyz) => format!("XYZ({}, {}, {})", xyz.x, xyz.y, xyz.z),
            DeimosValue::List(l) => {
                let items: Vec<String> = l.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            DeimosValue::Keycode(k) => format!("{:?}", k),
            DeimosValue::None => "None".to_string(),
        }
    }

    pub fn to_i32(&self) -> i32 {
        self.to_f64() as i32
    }

    pub fn to_xyz(&self) -> Option<XYZ> {
        if let DeimosValue::XYZ(xyz) = self {
            Some(*xyz)
        } else {
            None
        }
    }
}

impl From<f64> for DeimosValue {
    fn from(v: f64) -> Self {
        DeimosValue::Number(v)
    }
}

impl From<bool> for DeimosValue {
    fn from(v: bool) -> Self {
        DeimosValue::Bool(v)
    }
}

impl From<String> for DeimosValue {
    fn from(v: String) -> Self {
        DeimosValue::String(v)
    }
}

impl From<&str> for DeimosValue {
    fn from(v: &str) -> Self {
        DeimosValue::String(v.to_string())
    }
}

impl From<XYZ> for DeimosValue {
    fn from(v: XYZ) -> Self {
        DeimosValue::XYZ(v)
    }
}

impl From<Keycode> for DeimosValue {
    fn from(v: Keycode) -> Self {
        DeimosValue::Keycode(v)
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub stack: Vec<DeimosValue>,
    pub ip: usize,
    pub running: bool,
    pub waitfor: Option<Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>>,
}

impl Task {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            ip: 0,
            running: true,
            waitfor: None,
        }
    }
}

pub struct Scheduler {
    pub tasks: Vec<Task>,
    pub current_task_index: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: vec![Task::new()],
            current_task_index: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, index: usize) {
        self.tasks.remove(index);
        if self.current_task_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.current_task_index = 0;
        }
    }

    pub fn get_current_task(&self) -> &Task {
        &self.tasks[self.current_task_index]
    }

    pub fn get_current_task_mut(&mut self) -> &mut Task {
        &mut self.tasks[self.current_task_index]
    }

    pub fn switch_task(&mut self) {
        if !self.tasks.is_empty() {
            self.current_task_index = (self.current_task_index + 1) % self.tasks.len();
        }
    }
}

#[derive(Debug, Clone)]
pub struct UntilInfo {
    pub expr: Expression,
    pub id: i32,
    pub exit_point: usize,
    pub stack_size: usize,
}

pub struct VM {
    pub clients: Vec<Client>,
    pub program: Vec<Instruction>,
    pub running: bool,
    pub killed: bool,
    pub scheduler: Scheduler,
    pub any_player_client: Vec<usize>, // indices of clients
    pub timers: HashMap<String, std::time::Instant>,
    pub logged_data: HashMap<String, HashMap<String, String>>,
    pub constants: HashMap<String, DeimosValue>,
    pub until_infos: Vec<UntilInfo>,
}

impl VM {
    pub fn new(clients: Vec<Client>) -> Self {
        Self {
            clients,
            program: Vec::new(),
            running: false,
            killed: false,
            scheduler: Scheduler::new(),
            any_player_client: Vec::new(),
            timers: HashMap::new(),
            logged_data: HashMap::from([
                ("goal".to_string(), HashMap::new()),
                ("quest".to_string(), HashMap::new()),
                ("zone".to_string(), HashMap::new()),
            ]),
            constants: HashMap::from([
                ("True".to_string(), DeimosValue::Bool(true)),
                ("False".to_string(), DeimosValue::Bool(false)),
            ]),
            until_infos: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.program.clear();
        self.scheduler = Scheduler::new();
        self.until_infos.clear();
        self.timers.clear();
        self.any_player_client.clear();
        self.constants = HashMap::from([
            ("True".to_string(), DeimosValue::Bool(true)),
            ("False".to_string(), DeimosValue::Bool(false)),
        ]);
        self.logged_data = HashMap::from([
            ("goal".to_string(), HashMap::new()),
            ("quest".to_string(), HashMap::new()),
            ("zone".to_string(), HashMap::new()),
        ]);
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn kill(&mut self) {
        self.stop();
        self.killed = true;
    }

    pub async fn define_constant(&mut self, name: String, value: DeimosValue) {
        let mut final_value = value;
        if let DeimosValue::String(ref s) = final_value {
            if let Some(kc) = self.string_to_keycode(s) {
                final_value = DeimosValue::Keycode(kc);
            }
        }
        self.constants.insert(name, final_value);
    }

    fn string_to_keycode(&self, s: &str) -> Option<Keycode> {
        match s.to_uppercase().as_str() {
            "W" => Some(Keycode::W),
            "A" => Some(Keycode::A),
            "S" => Some(Keycode::S),
            "D" => Some(Keycode::D),
            "X" => Some(Keycode::X),
            "Z" => Some(Keycode::Z),
            "ENTER" => Some(Keycode::Enter),
            "SPACE" => Some(Keycode::Spacebar),
            "ESCAPE" => Some(Keycode::Esc),
            "PAGEUP" => Some(Keycode::PageUp),
            "PAGEDOWN" => Some(Keycode::PageDown),
            "END" => Some(Keycode::End),
            _ => None,
        }
    }

    pub fn player_by_num(&self, num: i32) -> Option<&Client> {
        let i = (num - 1) as usize;
        self.clients.get(i)
    }

    pub fn select_players(&self, selector: &PlayerSelector) -> Vec<usize> {
        if selector.mass {
            (0..self.clients.len()).collect()
        } else if selector.any_player {
            Vec::new()
        } else if selector.same_any {
            self.any_player_client.clone()
        } else {
            let mut result = Vec::new();
            if selector.inverted {
                for i in 0..self.clients.len() {
                    if !selector.player_nums.contains(&(i as i32 + 1)) {
                        result.push(i);
                    }
                }
            } else {
                for &num in &selector.player_nums {
                    let i = (num - 1) as usize;
                    if i < self.clients.len() {
                        result.push(i);
                    }
                }
            }
            result
        }
    }

    pub fn select_players_any(&self, selector: &PlayerSelector) -> Vec<usize> {
        if selector.any_player {
            if !self.any_player_client.is_empty() {
                self.any_player_client.clone()
            } else {
                if self.clients.is_empty() { vec![] } else { vec![0] }
            }
        } else {
            self.select_players(selector)
        }
    }

    async fn fetch_tracked_quest(&self, client_idx: usize) -> Result<QuestData, String> {
        let client = &self.clients[client_idx];
        let tracked_id = client.quest_id().map_err(|e| e.to_string())?;
        let qm = client.quest_manager().map_err(|e| e.to_string())?;
        let quest_data = qm.quest_data().map_err(|e| e.to_string())?;
        if let Some(quest) = quest_data.get(&(tracked_id as i32)) {
            Ok(QuestData::new(quest.inner.clone()))
        } else {
            Err(format!("Unable to fetch the currently tracked quest for client with title {}", client.title()))
        }
    }

    async fn fetch_tracked_quest_text(&self, client_idx: usize) -> Result<String, String> {
        let quest = self.fetch_tracked_quest(client_idx).await?;
        let name_key = quest.inner.read_value_from_offset::<u64>(0x88).map_err(|e| e.to_string())?; 
        let name = self.clients[client_idx].read_wide_string_at(name_key as usize).unwrap_or_else(|| "Unknown".to_string());
        Ok(name.to_lowercase().trim().to_string())
    }

    async fn fetch_quests(&self, client_idx: usize) -> Result<Vec<(i32, QuestData)>, String> {
        let qm = self.clients[client_idx].quest_manager().map_err(|e| e.to_string())?;
        let quest_data = qm.quest_data().map_err(|e| e.to_string())?;
        Ok(quest_data.into_iter().map(|(id, q)| (id as i32, QuestData::new(q.inner))).collect())
    }

    async fn fetch_quest_text(&self, client_idx: usize, quest: &QuestData) -> Result<String, String> {
        let name_key = quest.inner.read_value_from_offset::<u64>(0x88).map_err(|e| e.to_string())?;
        if name_key == 0 {
            return Ok("quest finder".to_string());
        }
        let name = self.clients[client_idx].read_wide_string_at(name_key as usize).unwrap_or_else(|| "Unknown".to_string());
        Ok(name.to_lowercase().trim().to_string())
    }

    async fn fetch_tracked_goal_text(&self, client: &Client) -> String {
        let goal_txt = get_quest_name(client).unwrap_or_default();
        let goal_txt = GOAL_REGEX.replace_all(&goal_txt, "").to_string();
        let goal_txt = if let Some(pos) = goal_txt.find('(') {
            goal_txt[..pos].to_string()
        } else {
            goal_txt
        };
        goal_txt.to_lowercase().trim().to_string()
    }

    async fn check_drops(&self, client: &Client, item_name: &str) -> bool {
        let chat_text = get_chat(client).await;
        if chat_text.is_empty() { return false; }
        let drops = filter_drops(chat_text.split('\n').map(|s| s.to_string()).collect());
        
        let new_chat_content = find_new_stuff("", &drops.join("\n"));
        for drop in new_chat_content.split('\n') {
            if !drop.is_empty() && drop.to_lowercase().contains(&item_name.to_lowercase()) {
                debug!("Found new dropped item matching '{}': {}", item_name, drop);
                return true;
            }
        }
        false
    }

    async fn check_duel_round(&self, client: &Client) -> i32 {
        if !client.in_battle() { return 0; }
        if let Some(duel) = client.duel() {
            return duel.round_num().unwrap_or(0);
        }
        0 
    }

    async fn extract_data_info(&mut self, data: &InstructionData) -> DeimosValue {
        match data {
            InstructionData::String(s) => {
                if s.starts_with('$') {
                    let const_name = &s[1..];
                    if let Some(val) = self.constants.get(const_name) {
                        return val.clone();
                    }
                }
                DeimosValue::String(s.clone())
            }
            InstructionData::Expression(expr) => {
                self.eval(expr, None).await.unwrap_or(DeimosValue::None)
            }
            _ => DeimosValue::None,
        }
    }

    fn maybe_get_named_window<'a>(&'a self, window: &'a DynamicWindow, name: &'a str) -> Pin<Box<dyn Future<Output = Option<DynamicWindow>> + Send + 'a>> {
        let name_owned = name.to_string();
        Box::pin(async move {
            if window.name().unwrap_or_default() == name_owned { return Some(window.clone()); }
            let children = window.children().ok()?;
            for child in children {
                if let Some(found) = self.maybe_get_named_window(&child, &name_owned).await { return Some(found); }
            }
            None
        })
    }

    async fn select_friend_from_list_vm(&self, client: &Client, name: &str) -> bool {
        let root = if let Some(rw) = client.root_window() { rw.window.clone() } else { return false; };
        let friends_window = self.maybe_get_named_window(&root, "NewFriendsListWindow").await;
        if friends_window.is_none() {
            let friend_button = self.maybe_get_named_window(&root, "btnFriends").await;
            if let Some(btn) = friend_button {
                 let _ = client.mouse_handler.click_window(&btn, false).await;
                 sleep(Duration::from_millis(400)).await;
            } else { return false; }
        }
        teleport_to_friend_from_list(client, None, None, Some(name.to_string())).await.is_ok()
    }

    pub fn eval_command_expression<'a>(&'a mut self, cmd: &'a Command) -> Pin<Box<dyn Future<Output = Result<DeimosValue, String>> + Send + 'a>> {
        let selector = cmd.player_selector.clone();
        let cmd_data_0 = cmd.data[0].clone();
        let cmd_data = cmd.data.clone();

        Box::pin(async move {
            let selector = selector.as_ref().ok_or("Command expression requires a player selector")?;

            let expr_kind_val = if let Expression::Number(n) = &*cmd_data_0 {
                 *n as i32
            } else {
                 -1
            };

            match expr_kind_val {
                // constant_check (38)
                38 => {
                     let constant_name = self.eval(&cmd_data[1], None).await?.to_string();
                     let expected_value = self.eval(&cmd_data[2], None).await?;
                     if let Some(actual_value) = self.constants.get(&constant_name) {
                         let mut final_actual = actual_value.clone();
                         if let (DeimosValue::Bool(_), DeimosValue::String(s)) = (&expected_value, &actual_value) {
                             if s.to_lowercase() == "true" { final_actual = DeimosValue::Bool(true); }
                             else if s.to_lowercase() == "false" { final_actual = DeimosValue::Bool(false); }
                         }
                         return Ok(DeimosValue::Bool(final_actual == expected_value));
                     }
                     return Ok(DeimosValue::Bool(false));
                }
                // zone_changed (37)
                37 => {
                    let mut expected_zone = None;
                    if cmd_data.len() > 1 {
                        expected_zone = Some(self.eval(&cmd_data[1], None).await?.to_string());
                    }
                    if selector.any_player {
                        self.any_player_client.clear();
                        let mut found_any = false;
                        for i in 0..self.clients.len() {
                            let current_zone = self.clients[i].zone_name().unwrap_or_default();
                            let last_zone = self.logged_data["zone"].get(&self.clients[i].title());
                            if let Some(expected) = &expected_zone {
                                if current_zone.to_lowercase() == expected.to_lowercase() {
                                // self.any_player_client.push(i);
                                // self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), current_zone);
                                    found_any = true;
                                }
                            } else {
                                if let Some(last) = last_zone {
                                    if current_zone.to_lowercase() != last.to_lowercase() {
                                    // self.any_player_client.push(i);
                                    // self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), current_zone.to_lowercase());
                                        found_any = true;
                                    }
                                } else {
                                    self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), current_zone.to_lowercase());
                                }
                            }
                        }
                        Ok(DeimosValue::Bool(found_any))
                    } else {
                        let indices = self.select_players(selector);
                        let mut all_valid = true;
                        for i in indices {
                            let current_zone = self.clients[i].zone_name().unwrap_or_default();
                            let last_zone = self.logged_data["zone"].get(&self.clients[i].title());
                            if let Some(expected) = &expected_zone {
                                if current_zone.to_lowercase() != expected.to_lowercase() {
                                    all_valid = false;
                                    break;
                                }
                                self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), current_zone);
                            } else {
                                if let Some(last) = last_zone {
                                    if current_zone.to_lowercase() == last.to_lowercase() {
                                        all_valid = false;
                                    } else {
                                        self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), current_zone.to_lowercase());
                                    }
                                } else {
                                    self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), current_zone.to_lowercase());
                                    all_valid = false;
                                }
                            }
                        }
                        Ok(DeimosValue::Bool(all_valid))
                    }
                }
                // goal_changed (35)
                35 => {
                    let mut expected_goal = None;
                    if cmd_data.len() > 1 {
                        expected_goal = Some(self.eval(&cmd_data[1], None).await?.to_string().to_lowercase());
                    }
                    if selector.any_player {
                        self.any_player_client.clear();
                        let mut found_any = false;
                        for i in 0..self.clients.len() {
                            let current_goal = self.fetch_tracked_goal_text(&self.clients[i]).await;
                            let last_goal = self.logged_data["goal"].get(&self.clients[i].title());
                            if let Some(expected) = &expected_goal {
                                if current_goal == *expected {
                                    self.any_player_client.push(i);
                                    self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), current_goal);
                                    found_any = true;
                                }
                            } else {
                                if let Some(last) = last_goal {
                                    if current_goal != *last {
                                        self.any_player_client.push(i);
                                        self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), current_goal);
                                        found_any = true;
                                    }
                                } else {
                                    self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), current_goal);
                                }
                            }
                        }
                        Ok(DeimosValue::Bool(found_any))
                    } else {
                        let indices = self.select_players(selector);
                        let mut all_valid = true;
                        for i in indices {
                            let current_goal = self.fetch_tracked_goal_text(&self.clients[i]).await;
                            let last_goal = self.logged_data["goal"].get(&self.clients[i].title());
                            if let Some(expected) = &expected_goal {
                                if current_goal != *expected {
                                    all_valid = false;
                                    break;
                                }
                                self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), current_goal);
                            } else {
                                if let Some(last) = last_goal {
                                    if current_goal == *last {
                                        all_valid = false;
                                    } else {
                                        self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), current_goal);
                                    }
                                } else {
                                    self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), current_goal);
                                    all_valid = false;
                                }
                            }
                        }
                        Ok(DeimosValue::Bool(all_valid))
                    }
                }
                // quest_changed (36)
                36 => {
                    let mut expected_quest = None;
                    if cmd_data.len() > 1 {
                        expected_quest = Some(self.eval(&cmd_data[1], None).await?.to_string().to_lowercase());
                    }
                    if selector.any_player {
                        self.any_player_client.clear();
                        let mut found_any = false;
                        for i in 0..self.clients.len() {
                        let current_quest = self.fetch_tracked_quest_text(i).await.unwrap_or_default();
                            let last_quest = self.logged_data["quest"].get(&self.clients[i].title());
                            if let Some(expected) = &expected_quest {
                                 if current_quest == *expected && last_quest.map_or(true, |l| current_quest != *l) {
                                     self.any_player_client.push(i);
                                     self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), current_quest);
                                     found_any = true;
                                 }
                            } else {
                                if let Some(last) = last_quest {
                                    if current_quest != *last {
                                        self.any_player_client.push(i);
                                        self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), current_quest);
                                        found_any = true;
                                    }
                                } else {
                                    self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), current_quest);
                                }
                            }
                        }
                        Ok(DeimosValue::Bool(found_any))
                    } else {
                        let indices = self.select_players(selector);
                        let mut all_valid = true;
                        for i in indices {
                        let current_quest = self.fetch_tracked_quest_text(i).await.unwrap_or_default();
                            let last_quest = self.logged_data["quest"].get(&self.clients[i].title());
                            if let Some(expected) = &expected_quest {
                                if current_quest != *expected || last_quest.map_or(false, |l| current_quest == *l) {
                                    all_valid = false;
                                    break;
                                }
                                self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), current_quest);
                            } else {
                                if let Some(last) = last_quest {
                                    if current_quest == *last { all_valid = false; }
                                    else { self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), current_quest); }
                                } else {
                                    self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), current_quest);
                                    all_valid = false;
                                }
                            }
                        }
                        Ok(DeimosValue::Bool(all_valid))
                    }
                }
            // duel_round (34)
            34 => {
                let expected_round = self.eval(&cmd_data[1], None).await?.to_i32();
                if selector.any_player {
                    self.any_player_client.clear();
                    let mut found_any = false;
                    for i in 0..self.clients.len() {
                        if self.check_duel_round(&self.clients[i]).await == expected_round {
                            self.any_player_client.push(i);
                            found_any = true;
                        }
                    }
                    Ok(DeimosValue::Bool(found_any))
                } else {
                    let indices = self.select_players(selector);
                    for i in indices {
                        if self.check_duel_round(&self.clients[i]).await != expected_round { return Ok(DeimosValue::Bool(false)); }
                    }
                    Ok(DeimosValue::Bool(true))
                }
            }
            // items_dropped (33)
            33 => {
                 let item_name = self.eval(&cmd_data[1], None).await?.to_string();
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if self.check_drops(&self.clients[i], &item_name).await {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if !self.check_drops(&self.clients[i], &item_name).await { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // window_visible (0)
            0 => {
                 let path_val = self.eval(&cmd_data[1], None).await?;
                 if let DeimosValue::List(l) = path_val {
                     let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                     let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                     if selector.any_player {
                         self.any_player_client.clear();
                         let mut found_any = false;
                         for i in 0..self.clients.len() {
                             if is_visible_by_path(&self.clients[i], &path_refs) {
                                 self.any_player_client.push(i);
                                 found_any = true;
                             }
                         }
                         Ok(DeimosValue::Bool(found_any))
                     } else {
                         let indices = self.select_players(selector);
                         for i in indices {
                             if !is_visible_by_path(&self.clients[i], &path_refs) { return Ok(DeimosValue::Bool(false)); }
                         }
                         Ok(DeimosValue::Bool(true))
                     }
                 } else { Ok(DeimosValue::Bool(false)) }
            }
            // window_disabled (26)
            26 => {
                 let path_val = self.eval(&cmd_data[1], None).await?;
                 if let DeimosValue::List(l) = path_val {
                     let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                     let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                     if selector.any_player {
                         self.any_player_client.clear();
                         let mut found_any = false;
                         for i in 0..self.clients.len() {
                             let root_opt = self.clients[i].root_window();
                             let root = root_opt.as_ref().map(|rw| &rw.window);
                             if let Some(r) = root {
                                 if let Some(win) = get_window_from_path(r, &path_refs) {
                                     if win.is_control_grayed().unwrap_or(false) {
                                         self.any_player_client.push(i);
                                         found_any = true;
                                     }
                                 }
                             }
                         }
                         Ok(DeimosValue::Bool(found_any))
                     } else {
                         let indices = self.select_players(selector);
                         for i in indices {
                             let root_opt = self.clients[i].root_window();
                             let root = root_opt.as_ref().map(|rw| &rw.window);
                             if let Some(r) = root {
                                 if let Some(win) = get_window_from_path(r, &path_refs) {
                                     if !win.is_control_grayed().unwrap_or(false) { return Ok(DeimosValue::Bool(false)); }
                                 } else { return Ok(DeimosValue::Bool(false)); }
                             } else { return Ok(DeimosValue::Bool(false)); }
                         }
                         Ok(DeimosValue::Bool(true))
                     }
                 } else { Ok(DeimosValue::Bool(false)) }
            }
                // in_range (28)
                28 => {
                     let target = self.eval(&cmd_data[1], None).await?.to_string().to_lowercase();
                     if selector.any_player {
                         self.any_player_client.clear();
                         let mut found_any = false;
                         for i in 0..self.clients.len() {
                             let sprinty = SprintyClient::new(&self.clients[i]);
                             let entities = sprinty.get_base_entity_list(None);
                             if entities.iter().any(|e| e.object_name.to_lowercase().contains(&target)) {
                                 self.any_player_client.push(i);
                                 found_any = true;
                             }
                         }
                         Ok(DeimosValue::Bool(found_any))
                     } else {
                         let indices = self.select_players(selector);
                         for i in indices {
                             let sprinty = SprintyClient::new(&self.clients[i]);
                             let entities = sprinty.get_base_entity_list(None);
                             if !entities.iter().any(|e| e.object_name.to_lowercase().contains(&target)) { return Ok(DeimosValue::Bool(false)); }
                         }
                         Ok(DeimosValue::Bool(true))
                     }
                }
            // same_place (27)
            27 => {
                 let indices = self.select_players(selector);
                 if indices.len() < 2 { return Ok(DeimosValue::Bool(true)); }
                 Ok(DeimosValue::Bool(true))
            }
            // in_zone (1)
            1 => {
                 let expected = self.eval(&cmd_data[1], None).await?.to_string();
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if self.clients[i].zone_name().unwrap_or_default() == expected {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if self.clients[i].zone_name().unwrap_or_default() != expected { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // same_zone (2)
            2 => {
                let indices = self.select_players(selector);
                if indices.is_empty() { return Ok(DeimosValue::Bool(true)); }
                let first_zone = self.clients[indices[0]].zone_name().unwrap_or_default();
                Ok(DeimosValue::Bool(indices.iter().all(|&i| self.clients[i].zone_name().unwrap_or_default() == first_zone)))
            }
            // same_quest (30)
            30 => {
                let indices = self.select_players(selector);
                if indices.is_empty() { return Ok(DeimosValue::Bool(true)); }
                let first_quest = self.fetch_tracked_quest_text(indices[0]).await.unwrap_or_default();
                let mut all_same = true;
                for &i in &indices[1..] {
                    if self.fetch_tracked_quest_text(i).await.unwrap_or_default() != first_quest {
                        all_same = false;
                        break;
                    }
                }
                Ok(DeimosValue::Bool(all_same))
            }
            // playercount (3)
            3 => {
                 let expected = self.eval(&cmd_data[1], None).await?.to_i32();
                 Ok(DeimosValue::Bool(self.clients.len() as i32 == expected))
            }
            // tracking_quest (4)
            4 => {
                 let expected = self.eval(&cmd_data[1], None).await?.to_string().to_lowercase();
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if self.fetch_tracked_quest_text(i).await.unwrap_or_default() == expected {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if self.fetch_tracked_quest_text(i).await.unwrap_or_default() != expected { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // tracking_goal (5)
            5 => {
                 let expected = self.eval(&cmd_data[1], None).await?.to_string().to_lowercase();
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if self.fetch_tracked_goal_text(&self.clients[i]).await == expected {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if self.fetch_tracked_goal_text(&self.clients[i]).await != expected { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // loading (6)
            6 => {
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if self.clients[i].is_loading() {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if !self.clients[i].is_loading() { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // in_combat (7)
            7 => {
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if self.clients[i].in_battle() {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if !self.clients[i].in_battle() { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // has_dialogue (8)
            8 => {
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if is_visible_by_path(&self.clients[i], paths::ADVANCE_DIALOG) {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if !is_visible_by_path(&self.clients[i], paths::ADVANCE_DIALOG) { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // has_xyz (9)
            9 => {
                 let target = self.eval(&cmd_data[1], None).await?.to_xyz().unwrap_or_default();
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         if calc_distance(&self.clients[i].body_position().unwrap_or_default(), &target) <= 1.0 {
                             self.any_player_client.push(i);
                             found_any = true;
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         if calc_distance(&self.clients[i].body_position().unwrap_or_default(), &target) > 1.0 { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // has_quest (10)
            10 => {
                 let expected = self.eval(&cmd_data[1], None).await?.to_string().to_lowercase();
                 if selector.any_player {
                     self.any_player_client.clear();
                     let mut found_any = false;
                     for i in 0..self.clients.len() {
                         let quests = self.fetch_quests(i).await.unwrap_or_default();
                         for (_, q) in quests {
                             if self.fetch_quest_text(i, &q).await.unwrap_or_default() == expected {
                                 self.any_player_client.push(i);
                                 found_any = true;
                                 break;
                             }
                         }
                     }
                     Ok(DeimosValue::Bool(found_any))
                 } else {
                     let indices = self.select_players(selector);
                     for i in indices {
                         let quests = self.fetch_quests(i).await.unwrap_or_default();
                         let mut found = false;
                         for (_, q) in quests {
                             if self.fetch_quest_text(i, &q).await.unwrap_or_default() == expected {
                                 found = true;
                                 break;
                             }
                         }
                         if !found { return Ok(DeimosValue::Bool(false)); }
                     }
                     Ok(DeimosValue::Bool(true))
                 }
            }
            // same_yaw (29)
            29 => {
                let indices = self.select_players(selector);
                if indices.len() < 2 { return Ok(DeimosValue::Bool(true)); }
                let first_yaw = (self.clients[indices[0]].body_read_yaw().unwrap_or(0.0) * 10.0).round() / 10.0;
                Ok(DeimosValue::Bool(indices.iter().all(|&i| (self.clients[i].body_read_yaw().unwrap_or(0.0) * 10.0).round() / 10.0 == first_yaw)))
            }
            // same_xyz (31)
            31 => {
                let indices = self.select_players(selector);
                if indices.len() < 2 { return Ok(DeimosValue::Bool(true)); }
                let first_pos = self.clients[indices[0]].body_position().unwrap_or_default();
                Ok(DeimosValue::Bool(indices.iter().all(|&i| calc_distance(&self.clients[i].body_position().unwrap_or_default(), &first_pos) <= 5.0)))
            }
            // has_yaw (32)
            32 => {
                let expected_yaw = (self.eval(&cmd_data[1], None).await?.to_f64() * 10.0).round() / 10.0;
                if selector.any_player {
                    self.any_player_client.clear();
                    let mut found_any = false;
                    for i in 0..self.clients.len() {
                        let cur_yaw = (self.clients[i].body_read_yaw().unwrap_or(0.0) * 10.0).round() / 10.0;
                        if (cur_yaw as f64 - expected_yaw).abs() < 0.1 {
                            self.any_player_client.push(i);
                            found_any = true;
                        }
                    }
                    Ok(DeimosValue::Bool(found_any))
                } else {
                    let indices = self.select_players(selector);
                    for i in indices {
                        let cur_yaw = (self.clients[i].body_read_yaw().unwrap_or(0.0) * 10.0).round() / 10.0;
                        if (cur_yaw as f64 - expected_yaw).abs() > 0.1 { return Ok(DeimosValue::Bool(false)); }
                    }
                    Ok(DeimosValue::Bool(true))
                }
            }
                _ => Ok(DeimosValue::Bool(false))
            }
        })
    }

    pub fn eval<'a>(&'a mut self, expression: &'a Expression, client_idx: Option<usize>) -> Pin<Box<dyn Future<Output = Result<DeimosValue, String>> + Send + 'a>> {
        let expression = expression.clone();
        Box::pin(async move {
            match expression {
                Expression::Ident(ident) => {
                    if ident.starts_with('$') {
                        let name = &ident[1..];
                        if let Some(val) = self.constants.get(name) {
                            return Ok(val.clone());
                        }
                    }
                    if let Some(val) = self.constants.get(&ident) {
                        return Ok(val.clone());
                    }
                    Ok(DeimosValue::String(ident.clone()))
                }
                Expression::ConstantReference(name) => {
                    if let Some(val) = self.constants.get(&name) {
                        Ok(val.clone())
                    } else {
                        Err(format!("Unknown constant: ${}", name))
                    }
                }
                Expression::ConstantCheck(name, value_expr) => {
                    if let Some(actual_value) = self.constants.get(&name).cloned() {
                        let expected_value = self.eval(&value_expr, client_idx).await?;
                        let mut final_actual = actual_value;
                        if let (DeimosValue::Bool(_), DeimosValue::String(s)) = (&expected_value, &final_actual) {
                             if s.to_lowercase() == "true" { final_actual = DeimosValue::Bool(true); }
                             else if s.to_lowercase() == "false" { final_actual = DeimosValue::Bool(false); }
                        }
                        Ok(DeimosValue::Bool(final_actual == expected_value))
                    } else {
                        Ok(DeimosValue::Bool(false))
                    }
                }
                Expression::And(exprs) => {
                    for e in exprs {
                        if !self.eval(&e, client_idx).await?.is_truthy() {
                            return Ok(DeimosValue::Bool(false));
                        }
                    }
                    Ok(DeimosValue::Bool(true))
                }
                Expression::Or(exprs) => {
                    for e in exprs {
                        if self.eval(&e, client_idx).await?.is_truthy() {
                            return Ok(DeimosValue::Bool(true));
                        }
                    }
                    Ok(DeimosValue::Bool(false))
                }
                Expression::Number(n) => Ok(DeimosValue::Number(n)),
                Expression::String(s) => Ok(DeimosValue::String(s.clone())),
                Expression::Sub(lhs, rhs) => {
                    let l = self.eval(&lhs, client_idx).await?.to_f64();
                    let r = self.eval(&rhs, client_idx).await?.to_f64();
                    Ok(DeimosValue::Number(l - r))
                }
                Expression::Divide(lhs, rhs) => {
                    let l = self.eval(&lhs, client_idx).await?.to_f64();
                    let r = self.eval(&rhs, client_idx).await?.to_f64();
                    if r == 0.0 { return Ok(DeimosValue::Number(0.0)); }
                    Ok(DeimosValue::Number(l / r))
                }
                Expression::Greater(lhs, rhs) => {
                    let l = self.eval(&lhs, client_idx).await?;
                    let r = self.eval(&rhs, client_idx).await?;
                    let lv = if let DeimosValue::List(ref list) = l { list.first().cloned().unwrap_or(DeimosValue::Number(0.0)) } else { l };
                    let rv = if let DeimosValue::List(ref list) = r { list.first().cloned().unwrap_or(DeimosValue::Number(0.0)) } else { r };
                    Ok(DeimosValue::Bool(lv.to_f64() > rv.to_f64()))
                }
                Expression::Equivalent(lhs, rhs) => {
                    let l = self.eval(&lhs, client_idx).await?;
                    let r = self.eval(&rhs, client_idx).await?;
                    let lv = if let DeimosValue::List(ref list) = l { list.first().cloned().unwrap_or(DeimosValue::Number(0.0)) } else { l };
                    let rv = if let DeimosValue::List(ref list) = r { list.first().cloned().unwrap_or(DeimosValue::Number(0.0)) } else { r };
                    Ok(DeimosValue::Bool(lv == rv))
                }
                Expression::CommandExpr(cmd) => {
                    self.eval_command_expression(&cmd).await
                }
                Expression::XYZ(x_expr, y_expr, z_expr) => {
                    let x = self.eval(&x_expr, client_idx).await?.to_f64() as f32;
                    let y = self.eval(&y_expr, client_idx).await?.to_f64() as f32;
                    let z = self.eval(&z_expr, client_idx).await?.to_f64() as f32;
                    Ok(DeimosValue::XYZ(XYZ { x, y, z }))
                }
                Expression::Unary(op, expr) => {
                    let val = self.eval(&expr, client_idx).await?;
                    match op.kind {
                        TokenKind::minus => Ok(DeimosValue::Number(-val.to_f64())),
                        TokenKind::keyword_not => {
                             let res = !val.is_truthy();
                             if let Expression::CommandExpr(ref cmd) = *expr {
                                 if cmd.player_selector.as_ref().map_or(false, |s| s.any_player) {
                                     let current_matches = self.any_player_client.clone();
                                     self.any_player_client = (0..self.clients.len())
                                         .filter(|i| !current_matches.contains(i))
                                         .collect();
                                 }
                             }
                             Ok(DeimosValue::Bool(res))
                        }
                        _ => Err(format!("Unimplemented unary operator: {:?}", op.kind)),
                    }
                }
                Expression::Eval(kind, args) => {
                    self.eval_expression_builtin(kind, &args, client_idx).await
                }
                Expression::SelectorGroup(selector, expr) => {
                    let indices = self.select_players(&selector);
                    if selector.any_player {
                        self.any_player_client.clear();
                        let mut found_any = false;
                        for i in 0..self.clients.len() {
                            let res = self.eval(&expr, Some(i)).await?;
                            if res.is_truthy() {
                                self.any_player_client.push(i);
                                found_any = true;
                            }
                        }
                        Ok(DeimosValue::Bool(found_any))
                    } else {
                        for i in indices {
                            if !self.eval(&expr, Some(i)).await?.is_truthy() {
                                return Ok(DeimosValue::Bool(false));
                            }
                        }
                        Ok(DeimosValue::Bool(true))
                    }
                }
                Expression::ReadVar(loc_expr) => {
                    let loc_val = self.eval(&loc_expr, client_idx).await?;
                    let loc = loc_val.to_f64() as usize;
                    let task = self.scheduler.get_current_task();
                    if loc < task.stack.len() {
                        Ok(task.stack[loc].clone())
                    } else {
                        Ok(DeimosValue::None)
                    }
                }
                Expression::StackLoc(offset) => Ok(DeimosValue::Number(offset as f64)),
                Expression::List(items) => {
                    let mut res = Vec::new();
                    for item in items {
                        let val = self.eval(&item, client_idx).await?;
                        if let DeimosValue::List(l) = val {
                            res.extend(l);
                        } else {
                            res.push(val);
                        }
                    }
                    Ok(DeimosValue::List(res))
                }
                Expression::ContainsString(lhs, rhs) => {
                    let l = self.eval(&lhs, client_idx).await?.to_string();
                    let r = self.eval(&rhs, client_idx).await?;
                    if let DeimosValue::List(list) = r {
                        Ok(DeimosValue::Bool(list.iter().any(|item| l.contains(&item.to_string()))))
                    } else {
                        Ok(DeimosValue::Bool(l.contains(&r.to_string())))
                    }
                }
                Expression::Key(k) => {
                    if let Some(kc) = self.string_to_keycode(&k) {
                        Ok(DeimosValue::Keycode(kc))
                    } else {
                        Ok(DeimosValue::String(k.clone()))
                    }
                }
                Expression::RangeMin(expr) => {
                    let range_str = self.eval(&expr, client_idx).await?.to_string();
                    let min_val = range_str.split('-').next().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    Ok(DeimosValue::Number(min_val))
                }
                Expression::RangeMax(expr) => {
                    let range_str = self.eval(&expr, client_idx).await?.to_string();
                    let max_val = range_str.split('-').nth(1).unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                    Ok(DeimosValue::Number(max_val))
                }
                Expression::IndexAccess(expr, index_expr) => {
                    let container = self.eval(&expr, client_idx).await?;
                    let index = self.eval(&index_expr, client_idx).await?.to_f64() as usize;
                    if let DeimosValue::List(l) = container {
                        Ok(l.get(index).cloned().unwrap_or(DeimosValue::Number(0.0)))
                    } else {
                        Ok(DeimosValue::Number(0.0))
                    }
                }
                Expression::StrFormat(fmt, args) => {
                    let mut final_str = fmt.clone();
                    for arg in args {
                        let val = self.eval(&arg, client_idx).await?.to_string();
                        if let Some(pos) = final_str.find('%') {
                            final_str.replace_range(pos..pos+1, &val);
                        }
                    }
                    Ok(DeimosValue::String(final_str))
                }
                _ => Err(format!("Unimplemented expression type: {:?}", expression)),
            }
        })
    }

    async fn eval_expression_builtin(&mut self, kind: EvalKind, args: &[Box<Expression>], client_idx: Option<usize>) -> Result<DeimosValue, String> {
        let client_idx = client_idx.ok_or("Builtin eval requires a client context")?;
        match kind {
            EvalKind::health => Ok(DeimosValue::Number(self.clients[client_idx].stats_current_hitpoints().unwrap_or(0) as f64)),
            EvalKind::max_health => Ok(DeimosValue::Number(self.clients[client_idx].stats_max_hitpoints().unwrap_or(0) as f64)),
            EvalKind::mana => Ok(DeimosValue::Number(self.clients[client_idx].stats_current_mana().unwrap_or(0) as f64)),
            EvalKind::max_mana => Ok(DeimosValue::Number(self.clients[client_idx].stats_max_mana().unwrap_or(0) as f64)),
            EvalKind::energy => Ok(DeimosValue::Number(self.clients[client_idx].stats_current_energy().unwrap_or(0) as f64)),
            EvalKind::bagcount => Ok(DeimosValue::Number(self.clients[client_idx].backpack_space().unwrap_or((0, 0)).0 as f64)),
            EvalKind::max_bagcount => Ok(DeimosValue::Number(self.clients[client_idx].backpack_space().unwrap_or((0, 0)).1 as f64)),
            EvalKind::gold => Ok(DeimosValue::Number(self.clients[client_idx].stats_current_gold().unwrap_or(0) as f64)),
            EvalKind::playercount => Ok(DeimosValue::Number(self.clients.len() as f64)),
            EvalKind::potioncount => Ok(DeimosValue::Number(self.clients[client_idx].stats_potion_charge().unwrap_or(0.0) as f64)),
            EvalKind::max_potioncount => Ok(DeimosValue::Number(self.clients[client_idx].stats_potion_max().unwrap_or(0.0) as f64)),
            EvalKind::windowtext => {
                 let path_val = self.eval(&args[0], Some(client_idx)).await?;
                 if let DeimosValue::List(l) = path_val {
                     let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                     let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                     Ok(DeimosValue::String(text_from_path(&self.clients[client_idx], &path_refs).unwrap_or_default().to_lowercase()))
                 } else {
                     Ok(DeimosValue::String("".into()))
                 }
            }
            EvalKind::windownum => {
                let path_val = self.eval(&args[0], Some(client_idx)).await?;
                if let DeimosValue::List(l) = path_val {
                    let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                    let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                    let text = text_from_path(&self.clients[client_idx], &path_refs).unwrap_or_default();
                    let nums: Vec<DeimosValue> = text.split('/')
                        .map(|s| {
                            let numeric: String = s.chars().filter(|c| c.is_digit(10) || *c == '.' || *c == '-').collect();
                            DeimosValue::Number(numeric.parse::<f64>().unwrap_or(0.0))
                        })
                        .collect();
                    Ok(DeimosValue::List(nums))
                } else {
                    Ok(DeimosValue::List(vec![DeimosValue::Number(0.0)]))
                }
            }
            EvalKind::any_player_list => {
                let titles: Vec<DeimosValue> = self.any_player_client.iter()
                    .map(|&i| DeimosValue::String(self.clients[i].title().clone()))
                    .collect();
                if titles.is_empty() && !self.clients.is_empty() {
                    Ok(DeimosValue::List(vec![DeimosValue::String(self.clients[0].title().clone())]))
                } else {
                    Ok(DeimosValue::List(titles))
                }
            }
            EvalKind::account_level => Ok(DeimosValue::Number(self.clients[client_idx].stats_reference_level().unwrap_or(0) as f64)),
            EvalKind::duel_round => Ok(DeimosValue::Number(self.check_duel_round(&self.clients[client_idx]).await as f64)),
            _ => Ok(DeimosValue::Number(0.0)),
        }
    }

    async fn exec_deimos_call(&mut self, instruction: &Instruction) -> Result<(), String> {
        let data = if let Some(InstructionData::List(d)) = &instruction.data { d } else { return Ok(()); };
        let selector = if let InstructionData::PlayerSelector(s) = &data[0] { s } else { return Ok(()); };
        let command_name = if let InstructionData::String(s) = &data[1] { s } else { return Ok(()); };
        let args_data = if let InstructionData::List(l) = &data[2] { l } else { return Ok(()); };

        let indices = self.select_players_any(selector);

        if indices.is_empty() { return Ok(()); }

        match command_name.as_str() {
            "set_zone" => {
                for &i in &indices {
                    let zone = self.clients[i].zone_name().unwrap_or_default();
                    self.logged_data.get_mut("zone").unwrap().insert(self.clients[i].title().clone(), zone);
                }
            }
            "set_goal" => {
                for &i in &indices {
                    let goal = self.fetch_tracked_goal_text(&self.clients[i]).await;
                    self.logged_data.get_mut("goal").unwrap().insert(self.clients[i].title().clone(), goal);
                }
            }
            "set_quest" => {
                for &i in &indices {
                    let quest = self.fetch_tracked_quest_text(i).await.unwrap_or_default();
                    self.logged_data.get_mut("quest").unwrap().insert(self.clients[i].title().clone(), quest);
                }
            }
            "autopet" => {
                for &i in &indices {
                    attempt_activate_dance_hook(&self.clients[i]).await;
                    let _ = dancedance(&self.clients[i]).await;
                }
            }
            "teleport" => {
                let tp_kind_val = if let InstructionData::Int(k) = &args_data[0] { *k } else { -1 };
                let tp_kind = match tp_kind_val {
                    0 => TeleportKind::position,
                    1 => TeleportKind::friend_icon,
                    2 => TeleportKind::friend_name,
                    3 => TeleportKind::entity_vague,
                    4 => TeleportKind::entity_literal,
                    5 => TeleportKind::mob,
                    6 => TeleportKind::quest,
                    7 => TeleportKind::client_num,
                    8 => TeleportKind::nav,
                    9 => TeleportKind::plusteleport,
                    10 => TeleportKind::minusteleport,
                    _ => TeleportKind::position,
                };
                match tp_kind {
                    TeleportKind::position => {
                         let pos_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         for &i in &indices {
                             let pos = self.eval(&pos_expr, Some(i)).await?.to_xyz().unwrap_or_default();
                             let _ = self.clients[i].teleport(&pos);
                         }
                    }
                    TeleportKind::entity_literal | TeleportKind::entity_vague => {
                         let mut use_navmap = false;
                         let mut start_idx = 1;
                         if args_data.len() > 2 && matches!(args_data[1], InstructionData::Int(8)) {
                             use_navmap = true;
                             start_idx = 2;
                         }
                         let name_expr = if let InstructionData::Expression(e) = &args_data[start_idx] { e.clone() } else { return Ok(()); };
                         let name = self.eval(&name_expr, Some(0)).await?.to_string();
                         for &i in &indices {
                             let sprinty = SprintyClient::new(&self.clients[i]);
                             let found = if tp_kind == TeleportKind::entity_literal {
                                 sprinty.get_base_entity_list(None).into_iter().find(|e| e.object_name == name)
                             } else {
                                 sprinty.get_entities_with_vague_name(&name, None).first().cloned()
                             };
                             if let Some(e) = found {
                                 if let Some(pos) = e.location {
                                     if use_navmap { let _ = navmap_tp(&self.clients[i], Some(&pos)).await; }
                                     else { let _ = self.clients[i].teleport(&pos); }
                                 }
                             }
                         }
                    }
                    TeleportKind::mob => {
                         for &i in &indices {
                             let sprinty = SprintyClient::new(&self.clients[i]);
                             let mobs = sprinty.get_mobs();
                             if let Some(ent) = sprinty.find_closest_of(&mobs, false) {
                                 if let Some(pos) = ent.location {
                                     let _ = self.clients[i].teleport(&pos);
                                 }
                             }
                         }
                    }
                    TeleportKind::quest => {
                         for &i in &indices {
                             if let Some(pos) = self.clients[i].quest_position() {
                                 let _ = navmap_tp(&self.clients[i], Some(&pos)).await;
                             }
                         }
                    }
                    TeleportKind::friend_icon => {
                         for &i in &indices {
                              let _ = teleport_to_friend_from_list(&self.clients[i], None, Some(2), None).await;
                         }
                    }
                    TeleportKind::friend_name => {
                         let name_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         let name = self.eval(&name_expr, Some(0)).await?.to_string();
                         for &i in &indices {
                              let _ = teleport_to_friend_from_list(&self.clients[i], None, None, Some(name.clone())).await;
                         }
                    }
                    TeleportKind::client_num => {
                         let num_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         let num = self.eval(&num_expr, Some(0)).await?.to_i32();
                         if let Some(target) = self.player_by_num(num) {
                             if let Some(pos) = target.body_position() {
                                 for &i in &indices { let _ = self.clients[i].teleport(&pos); }
                             }
                         }
                    }
                    TeleportKind::plusteleport => {
                         let pos_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         for &i in &indices {
                             let pluspos = self.eval(&pos_expr, Some(i)).await?.to_xyz().unwrap_or_default();
                             let curpos = self.clients[i].body_position().unwrap_or_default();
                             let newpos = XYZ { x: curpos.x + pluspos.x, y: curpos.y + pluspos.y, z: curpos.z + pluspos.z };
                             let _ = self.clients[i].teleport(&newpos);
                         }
                    }
                    TeleportKind::minusteleport => {
                         let pos_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         for &i in &indices {
                             let minuspos = self.eval(&pos_expr, Some(i)).await?.to_xyz().unwrap_or_default();
                             let curpos = self.clients[i].body_position().unwrap_or_default();
                             let newpos = XYZ { x: curpos.x - minuspos.x, y: curpos.y - minuspos.y, z: curpos.z - minuspos.z };
                             let _ = self.clients[i].teleport(&newpos);
                         }
                    }
                    _ => {}
                }
            }
            "goto" => {
                 let pos_expr = if let InstructionData::Expression(e) = &args_data[0] { e } else { return Ok(()); };
                 for &i in &indices {
                     let pos_expr_inner = pos_expr.clone();
                     let pos_val = self.eval(&pos_expr_inner, Some(i)).await?.to_xyz().unwrap_or_default();
                     let _ = self.clients[i].goto(pos_val.x, pos_val.y);
                 }
            }
            "waitfor" => {
                 let wait_kind_val = if let InstructionData::Int(k) = &args_data[0] { *k } else { -1 };
                 let wait_kind = match wait_kind_val {
                     0 => WaitforKind::dialog,
                     1 => WaitforKind::battle,
                     2 => WaitforKind::zonechange,
                     3 => WaitforKind::free,
                     4 => WaitforKind::window,
                     _ => WaitforKind::free,
                 };
                 let completion = if let InstructionData::Int(c) = args_data.last().unwrap() { *c != 0 } else { true };
                 match wait_kind {
                     WaitforKind::dialog => {
                         while indices.iter().any(|&i| is_visible_by_path(&self.clients[i], paths::ADVANCE_DIALOG) != completion) {
                             sleep(Duration::from_millis(250)).await;
                         }
                     }
                     WaitforKind::battle => {
                         while indices.iter().any(|&i| self.clients[i].in_battle() != completion) {
                             sleep(Duration::from_millis(250)).await;
                         }
                     }
                     WaitforKind::zonechange => {
                         if completion {
                             while indices.iter().any(|&i| self.clients[i].is_loading()) { sleep(Duration::from_millis(250)).await; }
                         } else {
                             sleep(Duration::from_millis(250)).await;
                         }
                     }
                     WaitforKind::free => {
                         while indices.iter().any(|&i| is_free(&self.clients[i]) != completion) {
                             sleep(Duration::from_millis(250)).await;
                         }
                     }
                     WaitforKind::window => {
                         let path_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         let path_val = self.eval(&path_expr, Some(0)).await?;
                         if let DeimosValue::List(l) = path_val {
                             let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                             let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                             while indices.iter().any(|&i| is_visible_by_path(&self.clients[i], &path_refs) != completion) {
                                 sleep(Duration::from_millis(250)).await;
                             }
                         }
                     }
                     _ => {}
                 }
            }
            "sendkey" => {
                let key_expr = if let InstructionData::Expression(e) = &args_data[0] { e } else { return Ok(()); };
                let time = if args_data.len() > 1 {
                    if let InstructionData::Expression(e) = &args_data[1] { self.eval(e, None).await?.to_f64() } else { 0.1 }
                } else { 0.1 };
                for &i in &indices {
                    let key_val = self.eval(key_expr, Some(i)).await?;
                    let key = if let DeimosValue::Keycode(kc) = key_val { kc } else { continue; };
                    let _ = self.clients[i].send_key(key);
                }
            }
            "usepotion" => {
                 for &i in &indices {
                      let _ = self.clients[i].mouse_handler.use_potion_if_needed(0, 0).await;
                 }
            }
            "buypotions" => {
                 let mut if_needed = false;
                 if !args_data.is_empty() {
                    if let InstructionData::Expression(e) = &args_data[0] {
                        if_needed = self.eval(e, None).await?.is_truthy();
                    }
                 }
                 for &i in &indices {
                      if if_needed { refill_potions_if_needed(&self.clients[i], true, true, None).await; }
                      else { refill_potions(&self.clients[i], true, true, None).await; }
                 }
            }
            "relog" => {
                 for &i in &indices {
                      logout_and_in(&self.clients[i]).await;
                 }
            }
            "cursor" => {
                let cur_kind_val = if let InstructionData::Int(k) = &args_data[0] { *k } else { -1 };
                match cur_kind_val {
                    0 => { // position
                         let x_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         let y_expr = if let InstructionData::Expression(e) = &args_data[2] { e.clone() } else { return Ok(()); };
                         let x = self.eval(&x_expr, Some(0)).await?.to_f64() as i32;
                         let y = self.eval(&y_expr, Some(0)).await?.to_f64() as i32;
                         for &i in &indices {
                             let _ = self.clients[i].mouse_handler.set_mouse_position(x, y, true, false).await;
                         }
                    }
                    1 => { // window
                         let path_expr = if let InstructionData::Expression(e) = &args_data[1] { e } else { return Ok(()); };
                         let path_val = self.eval(path_expr, Some(0)).await?;
                         if let DeimosValue::List(l) = path_val {
                             let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                             let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                             for &i in &indices {
                                 let root_opt = self.clients[i].root_window();
                                 let root = root_opt.as_ref().map(|rw| &rw.window);
                                 if let Some(r) = root {
                                     if let Some(win) = get_window_from_path(r, &path_refs) {
                                         let _ = self.clients[i].mouse_handler.set_mouse_position_to_window(&win).await;
                                     }
                                 }
                             }
                         }
                    }
                    _ => {}
                }
            }
            "click" => {
                let click_kind_val = if let InstructionData::Int(k) = &args_data[0] { *k } else { -1 };
                match click_kind_val {
                    1 => { // position
                         let x_expr = if let InstructionData::Expression(e) = &args_data[1] { e.clone() } else { return Ok(()); };
                         let y_expr = if let InstructionData::Expression(e) = &args_data[2] { e.clone() } else { return Ok(()); };
                         let x = self.eval(&x_expr, Some(0)).await?.to_f64() as i32;
                         let y = self.eval(&y_expr, Some(0)).await?.to_f64() as i32;
                         for &i in &indices {
                             let _ = self.clients[i].mouse_handler.click(x, y, false, 0.0, false).await;
                         }
                    }
                    0 => { // window
                         let path_expr = if let InstructionData::Expression(e) = &args_data[1] { e } else { return Ok(()); };
                         for &i in &indices {
                             let path_val = self.eval(path_expr, Some(i)).await?;
                             if let DeimosValue::List(l) = path_val {
                                 let path: Vec<String> = l.into_iter().map(|v| v.to_string()).collect();
                                 let path_refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
                                 let _ = click_window_by_path(&self.clients[i], &path_refs).await;
                             }
                         }
                    }
                    _ => {}
                }
            }
            "tozone" => {
                 let path_expr = if let InstructionData::Expression(e) = &args_data[0] { e } else { return Ok(()); };
                 let path_val = self.eval(path_expr, Some(0)).await?;
                 let zone_path = if let DeimosValue::List(l) = path_val {
                      l.into_iter().map(|v| v.to_string()).collect::<Vec<String>>().join("/")
                 } else { path_val.to_string() };
                 for &i in &indices {
                      let _ = self.clients[i].teleport_to_zone_display_name(&zone_path).await;
                 }
            }
            "select_friend" => {
                 let name_expr = if let InstructionData::Expression(e) = &args_data[0] { e } else { return Ok(()); };
                 let name = self.eval(name_expr, Some(0)).await?.to_string();
                 for &i in &indices {
                      let _ = self.select_friend_from_list_vm(&self.clients[i], &name).await;
                 }
            }
            _ => {}
        }
        Ok(())
    }

    async fn exec_compound_deimos_call(&mut self, entries: &Vec<InstructionData>) -> Result<(), String> {
        for entry in entries {
            if let InstructionData::List(data) = entry {
                let instr = Instruction {
                    kind: InstructionKind::deimos_call,
                    data: Some(InstructionData::List(data.clone())),
                };
                self.exec_deimos_call(&instr).await?;
            }
        }
        Ok(())
    }

    pub async fn step(&mut self) -> Result<(), String> {
        if !self.running { return Ok(()); }
        sleep(Duration::from_millis(0)).await;
        
        let current_task_idx = self.scheduler.current_task_index;
        
        // Process untils
        let mut exit_until = None;
        for i in (0..self.until_infos.len()).rev() {
            let expr = self.until_infos[i].expr.clone();
            if self.eval(&expr, None).await?.is_truthy() {
                exit_until = Some(i);
                break;
            }
        }

        if let Some(i) = exit_until {
            let info = self.until_infos[i].clone();
            self.until_infos.truncate(i);
            let task = &mut self.scheduler.tasks[current_task_idx];
            task.ip = info.exit_point;
            task.stack.truncate(info.stack_size);
            return Ok(());
        }

        let task = &mut self.scheduler.tasks[current_task_idx];
        if !task.running {
            self.scheduler.switch_task();
            return Ok(());
        }
        
        if task.ip >= self.program.len() {
            task.running = false;
            return Ok(());
        }
        
        let instr = self.program[task.ip].clone();
        match instr.kind {
            InstructionKind::restart_bot => {
                self.reset();
                self.scheduler.get_current_task_mut().ip = 0;
                debug!("Bot Restarted");
            }
            InstructionKind::kill => {
                self.any_player_client.clear();
                self.kill();
                debug!("Bot Killed");
            }
            InstructionKind::sleep => {
                if let Some(InstructionData::Expression(expr)) = instr.data {
                    let val = self.eval(&expr, None).await?;
                    sleep(Duration::from_secs_f64(val.to_f64())).await;
                }
                self.scheduler.get_current_task_mut().ip += 1;
            }
            InstructionKind::jump => {
                if let Some(InstructionData::Int(offset)) = instr.data {
                    self.scheduler.get_current_task_mut().ip = (self.scheduler.get_current_task_mut().ip as i32 + offset) as usize;
                }
            }
            InstructionKind::jump_if => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::Expression(expr), InstructionData::Int(offset)] = data.as_slice() {
                        let res = self.eval(expr, None).await?;
                        if res.is_truthy() {
                            self.scheduler.get_current_task_mut().ip = (self.scheduler.get_current_task_mut().ip as i32 + *offset) as usize;
                        } else {
                            self.scheduler.get_current_task_mut().ip += 1;
                        }
                    }
                }
            }
            InstructionKind::jump_ifn => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::Expression(expr), InstructionData::Int(offset)] = data.as_slice() {
                        let res = self.eval(expr, None).await?;
                        if !res.is_truthy() {
                            self.scheduler.get_current_task_mut().ip = (self.scheduler.get_current_task_mut().ip as i32 + *offset) as usize;
                        } else {
                            self.scheduler.get_current_task_mut().ip += 1;
                        }
                    }
                }
            }
            InstructionKind::call => {
                if let Some(InstructionData::Int(offset)) = instr.data {
                    let return_ip = task.ip + 1;
                    task.stack.push(DeimosValue::Number(return_ip as f64));
                    task.ip = (task.ip as i32 + offset) as usize;
                }
            }
            InstructionKind::ret => {
                if let Some(val) = task.stack.pop() {
                    task.ip = val.to_f64() as usize;
                }
            }
            InstructionKind::push_stack => {
                task.stack.push(DeimosValue::None);
                task.ip += 1;
            }
            InstructionKind::pop_stack => {
                task.stack.pop();
                task.ip += 1;
            }
            InstructionKind::write_stack => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::Int(offset), InstructionData::Expression(expr)] = data.as_slice() {
                        let offset = *offset as usize;
                        let val = self.eval(expr, None).await?;
                        let task = &mut self.scheduler.tasks[self.scheduler.current_task_index];
                        task.stack[offset] = val;
                        task.ip += 1;
                    }
                }
            }
            InstructionKind::declare_constant => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::String(name), InstructionData::Expression(expr)] = data.as_slice() {
                        let val = self.eval(expr, None).await?;
                        self.define_constant(name.clone(), val).await;
                        self.scheduler.get_current_task_mut().ip += 1;
                    }
                }
            }
            InstructionKind::set_timer => {
                if let Some(InstructionData::String(name)) = instr.data {
                    self.timers.insert(name, std::time::Instant::now());
                    self.scheduler.get_current_task_mut().ip += 1;
                }
            }
            InstructionKind::end_timer => {
                if let Some(InstructionData::String(name)) = instr.data {
                    if let Some(start) = self.timers.remove(&name) {
                        let elapsed = start.elapsed();
                        let hours = elapsed.as_secs() / 3600;
                        let minutes = (elapsed.as_secs() % 3600) / 60;
                        let seconds = elapsed.as_secs() % 60;
                        debug!("Timer '{}' ended - Elapsed time: {:02}:{:02}:{:02}", name, hours, minutes, seconds);
                    }
                    self.scheduler.get_current_task_mut().ip += 1;
                }
            }
            InstructionKind::enter_until => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::Expression(expr), InstructionData::Int(id), InstructionData::Int(exit_offset)] = data.as_slice() {
                        let exit_point = (task.ip as i32 + *exit_offset) as usize;
                        self.until_infos.push(UntilInfo {
                            expr: *expr.clone(),
                            id: *id,
                            exit_point,
                            stack_size: task.stack.len(),
                        });
                        task.ip += 1;
                    }
                }
            }
            InstructionKind::exit_until => {
                if let Some(InstructionData::Int(id)) = instr.data {
                    if let Some(pos) = self.until_infos.iter().rposition(|info| info.id == id) {
                        let info = &self.until_infos[pos];
                        task.stack.truncate(info.stack_size);
                        self.until_infos.truncate(pos);
                    }
                    task.ip += 1;
                }
            }
            InstructionKind::log_single => {
                if let Some(InstructionData::Expression(expr)) = instr.data {
                    let val = self.eval(&expr, None).await?;
                    debug!("{}", val.to_string());
                }
                self.scheduler.get_current_task_mut().ip += 1;
            }
            InstructionKind::log_multi => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::PlayerSelector(selector), InstructionData::Expression(expr)] = data.as_slice() {
                        let indices = self.select_players(selector);
                        for idx in indices {
                            let expr_clone = expr.clone();
                            let val = self.eval(&expr_clone, Some(idx)).await?;
                            debug!("{} - {}", self.clients[idx].title(), val.to_string());
                        }
                        self.scheduler.get_current_task_mut().ip += 1;
                    }
                }
            }
            InstructionKind::set_yaw => {
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::PlayerSelector(selector), InstructionData::Float(yaw)] = data.as_slice() {
                        let yaw = *yaw;
                        let indices = self.select_players_any(selector);
                        for idx in indices {
                             let _ = self.clients[idx].body_write_yaw(yaw as f32);
                        }
                        self.scheduler.get_current_task_mut().ip += 1;
                    }
                }
            }
            InstructionKind::load_playstyle => {
                if let Some(InstructionData::String(playstyle)) = instr.data {
                    let delegated = delegate_combat_configs(&playstyle, self.clients.len(), "");
                    for (i, config) in delegated {
                        if let Some(client) = self.clients.get_mut(i) {
                             // client.combat_config = config;
                        }
                    }
                    self.scheduler.get_current_task_mut().ip += 1;
                }
            }
            InstructionKind::deimos_call => {
                self.exec_deimos_call(&instr).await?;
                self.scheduler.get_current_task_mut().ip += 1;
            }
            InstructionKind::compound_deimos_call => {
                if let Some(InstructionData::List(entries)) = instr.data {
                    self.exec_compound_deimos_call(&entries).await?;
                }
                self.scheduler.get_current_task_mut().ip += 1;
            }
            InstructionKind::setdeck => {
                use wizwalker::memory::objects::GameStats;
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::PlayerSelector(selector), InstructionData::String(token)] = data.as_slice() {
                        let indices = self.select_players(selector);
                        if let Ok(deck) = decode_deck(token) {
                            for idx in indices {
                                if let Some(stats) = self.clients[idx].stats() {
                                     if let Ok(deck_obj) = stats.deck() {
                                          // deck_obj.set_from_preset(deck);
                                     }
                                }
                            }
                        }
                        self.scheduler.get_current_task_mut().ip += 1;
                    }
                }
            }
            InstructionKind::getdeck => {
                use wizwalker::memory::objects::GameStats;
                if let Some(InstructionData::List(data)) = instr.data {
                    if let [InstructionData::PlayerSelector(selector)] = data.as_slice() {
                        let indices = self.select_players(selector);
                        for idx in indices {
                            if let Some(stats) = self.clients[idx].stats() {
                                if let Ok(deck_obj) = stats.deck() {
                                    // let deck = deck_obj.to_preset();
                                    // let token = encode_deck(&deck);
                                }
                            }
                        }
                        self.scheduler.get_current_task_mut().ip += 1;
                    }
                }
            }
            InstructionKind::nop | InstructionKind::label => {
                self.scheduler.get_current_task_mut().ip += 1;
            }
            _ => {
                self.scheduler.get_current_task_mut().ip += 1;
            }
        }
        
        if self.scheduler.get_current_task().ip >= self.program.len() {
            self.scheduler.get_current_task_mut().running = false;
        }
        
        let any_running = self.scheduler.tasks.iter().any(|t| t.running);
        if !any_running || !self.running {
            self.stop();
        } else {
            self.scheduler.switch_task();
        }
        
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), String> {
        self.running = true;
        while self.running {
            self.step().await?;
        }
        Ok(())
    }
}
