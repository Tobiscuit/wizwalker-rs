//! Command parser — Parses user text commands into structured commands.
//!
//! Faithfully ported from `deimos-reference/src/command_parser.py`.
#![allow(dead_code, unused_imports, non_snake_case, unused_mut, unused_variables)]

use crate::automation::camera_utils::{OrbitParams, GlideParams, rotation_velocity, point_to_xyz_orient};
use crate::automation::sprinty_client::SprintyClient;
use crate::automation::utils::{
    click_window_by_path, get_window_from_path, is_free, wait_for_zone_change, wait_for_loading_screen, is_visible_by_path
};
use crate::automation::paths;
use futures::future::join_all;
use std::f32::consts::{E, PI, TAU};
use tracing::{debug, error};
use wizwalker::client::Client;
use wizwalker::constants::Keycode;
use wizwalker::memory::objects::camera_controller::DynamicCameraController;
use wizwalker::types::{Orient, XYZ};

#[derive(Debug, Clone, PartialEq)]
pub enum CommandToken {
    String(String),
    List(Vec<String>),
}

impl CommandToken {
    pub fn as_str(&self) -> &str {
        match self {
            CommandToken::String(s) => s,
            CommandToken::List(_) => "",
        }
    }
    pub fn is_list(&self) -> bool { matches!(self, CommandToken::List(_)) }
    pub fn as_list(&self) -> Vec<String> {
        match self {
            CommandToken::String(s) => vec![s.clone()],
            CommandToken::List(l) => l.clone(),
        }
    }
}

pub fn tokenize(l: &str) -> Vec<CommandToken> {
    let mut result = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = l.chars().collect();
    let mut in_brackets = false;
    let mut bracket_list = Vec::new();
    let mut word = String::new();
    while i < chars.len() {
        let c = chars[i];
        match c {
            '#' => break,
            '(' => {
                let mut res = String::from('('); i += 1;
                while i < chars.len() {
                    let x = chars[i]; res.push(x); i += 1;
                    if x == ')' { break; }
                }
                let s = (word + &res).trim().to_string();
                if !s.is_empty() { if in_brackets { bracket_list.push(s); } else { result.push(CommandToken::String(s)); } }
                word = String::new(); continue;
            }
            ')' => { word.push(c); }
            '[' => { in_brackets = true; i += 1; continue; }
            ']' => {
                let s = word.trim().to_string(); if !s.is_empty() { bracket_list.push(s); }
                word = String::new(); result.push(CommandToken::List(bracket_list.clone()));
                bracket_list.clear(); in_brackets = false; i += 1; continue;
            }
            '"' | '\'' => {
                let q = c; let mut s = String::new(); i += 1;
                while i < chars.len() && chars[i] != q { s.push(chars[i]); i += 1; }
                if i < chars.len() { i += 1; }
                if in_brackets { bracket_list.push(s); } else { result.push(CommandToken::String(s)); }
            }
            ',' | ' ' | '\t' | '\n' | '\r' => {
                let s = word.trim().to_string();
                if !s.is_empty() { if in_brackets { bracket_list.push(s); } else { result.push(CommandToken::String(s)); } }
                word = String::new();
            }
            _ => { word.push(c); }
        }
        i += 1;
    }
    let s = word.trim().to_string();
    if !s.is_empty() { if in_brackets { bracket_list.push(s); } else { result.push(CommandToken::String(s)); } }
    result
}

pub fn is_numeric(s: &str) -> bool { s.parse::<f32>().is_ok() }
pub fn to_number(s: &str) -> f32 { match s { "pi" => PI, "tau" => TAU, "e" => E, _ => s.parse::<f32>().unwrap_or(0.0) } }
pub fn next_value(l: &[String], i: usize, d: f32, a: usize) -> f32 {
    let next_idx = if l.len() >= i + a + 1 { Some(i + a) } else if i >= a { Some(i - a) } else { None };
    if let Some(idx) = next_idx {
        let n = &l[idx]; if is_numeric(n) || ["pi", "tau", "e"].contains(&n.as_str()) { return to_number(n); }
    }
    d
}
pub fn param_input(s: &str, d: f32) -> f32 {
    if is_numeric(s) || ["pi", "tau", "e"].contains(&s) { return to_number(s); }
    let mut s = s.to_string(); if !is_numeric(s.split_whitespace().next().unwrap_or("")) { s = format!("{} {}", d, s); }
    let eq: Vec<String> = s.split_whitespace().map(|x| x.replace(' ', "")).collect();
    if eq.is_empty() { return d; }
    let mut v = eq[0].parse::<f32>().unwrap_or(d);
    for (i, p) in eq.iter().enumerate() {
        match p.as_str() {
            "+" => v += next_value(&eq, i, v, 1), "-" => v -= next_value(&eq, i, v, 1),
            "*" => v *= next_value(&eq, i, v, 1), "/" => v /= next_value(&eq, i, v, 1),
            "//" => v = (v / next_value(&eq, i, v, 1)).floor(), "**" => v = v.powf(next_value(&eq, i, v, 1)),
            "mod" | "%" | "modulus" => v %= next_value(&eq, i, v, 1), "sqrt" => v = v.sqrt(),
            "abs" => v = v.abs(), "floor" => v = v.floor(), "ceil" | "ceiling" => v = v.ceil(),
            "deg" | "degrees" => v = v.to_degrees(), "rad" | "radians" => v = v.to_radians(),
            "sin" | "sine" => v = v.sin(), "cos" | "cosine" => v = v.cos(), "tan" | "tangent" => v = v.tan(),
            _ => {}
        }
    }
    v
}

pub async fn parse_location(s: &[CommandToken], cam: Option<&DynamicCameraController>, cl: Option<&Client>) -> (Vec<XYZ>, Vec<Orient>) {
    let s_vec: Vec<String> = s.iter().map(|x| x.as_str().to_lowercase().replace(", ", "")).collect();
    let mut xyzs = Vec::new(); let mut orients = Vec::new();
    let (d_xyz, d_orient) = if let Some(c) = cam { (c.position().unwrap_or_default(), c.orientation().unwrap_or_default()) }
    else if let Some(c) = cl {
        if let Some(body) = c.body() {
            (body.position().unwrap_or_default(), body.orientation().unwrap_or_default())
        } else {
            (XYZ::default(), Orient::default())
        }
    }
    else { (XYZ::default(), Orient::default()) };
    for arg in &s_vec {
        if arg.contains("xyz") {
            let arg = arg.replace("xyz(", "").replace(")", "");
            let split: Vec<&str> = arg.split(',').collect();
            if split.len() >= 3 { xyzs.push(XYZ { x: param_input(split[0], d_xyz.x), y: param_input(split[1], d_xyz.y), z: param_input(split[2], d_xyz.z) }); }
        } else if arg.contains("orient") {
            let arg = arg.replace("orient(", "").replace(")", "");
            let split: Vec<&str> = arg.split(',').collect();
            if split.len() >= 3 { orients.push(Orient { pitch: param_input(split[0], d_orient.pitch), roll: param_input(split[1], d_orient.roll), yaw: param_input(split[2], d_orient.yaw) }); }
        }
    }
    (xyzs, orients)
}

pub fn handle_index<T: Clone>(l: &[T], i: usize, d: Option<T>) -> Option<T> { if l.len() <= i { d } else { Some(l[i].clone()) } }
pub fn client_from_titles<'a>(cls: &'a Vec<Client>, t: &str) -> Option<&'a Client> { cls.iter().find(|c| c.title().to_lowercase() == t.to_lowercase()) }

pub async fn wait_for_coro<F, Fut>(f: F, n: bool, i: f64) where F: Fn() -> Fut, Fut: std::future::Future<Output = bool> {
    if n { while f().await { tokio::time::sleep(tokio::time::Duration::from_secs_f64(i)).await; } }
    else { while !f().await { tokio::time::sleep(tokio::time::Duration::from_secs_f64(i)).await; } }
}

async fn use_potion(c: &Client) { if let Some(s) = c.stats() { use wizwalker::memory::objects::GameStats; if s.potion_charge().unwrap_or(0.0) >= 1.0 { click_window_by_path(c, paths::POTION_USAGE_PATH).await; } } }

async fn buy_potions_logic(c: &Client, r: bool, o: Option<String>) {
    if let Some(s) = c.stats() {
        use wizwalker::memory::objects::GameStats;
        let max = s.potion_max().unwrap_or(3.0);
        for i in 0..2 {
            let orig = s.potion_charge().unwrap_or(0.0); let mut curr = orig;
            while curr == orig && curr < max {
                while !is_visible_by_path(c, paths::POTION_SHOP_BASE) { c.send_key(Keycode::X); tokio::time::sleep(tokio::time::Duration::from_millis(500)).await; }
                click_window_by_path(c, paths::POTION_FILL_ALL).await; tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                click_window_by_path(c, paths::POTION_BUY).await; tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                while is_visible_by_path(c, paths::POTION_SHOP_BASE) { click_window_by_path(c, paths::POTION_EXIT_PATH).await; tokio::time::sleep(tokio::time::Duration::from_millis(125)).await; }
                curr = s.potion_charge().unwrap_or(0.0); tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            if i == 0 && s.potion_charge().unwrap_or(0.0) >= 1.0 { click_window_by_path(c, paths::POTION_USAGE_PATH).await; tokio::time::sleep(tokio::time::Duration::from_secs(3)).await; }
        }
    }
    if r { if let Some(orig) = o { let curr = c.zone_name().unwrap_or_default(); if orig != curr { loop { c.send_key(Keycode::PageUp); if wait_for_zone_change(c, Some(&curr), 10.0) { break; } } } } }
}

async fn refill_potions(c: &Client, m: bool, r: bool) {
    if let Some(s) = c.stats() {
        use wizwalker::memory::objects::GameStats;
        if s.reference_level().unwrap_or(1) >= 6 {
            let orig = c.zone_name(); if m && orig.as_deref() != Some("WizardCity/WC_Hub") { c.send_key(Keycode::PageDown); tokio::time::sleep(tokio::time::Duration::from_secs(2)).await; }
            c.send_key(Keycode::Home); wait_for_zone_change(c, orig.as_deref(), 10.0); buy_potions_logic(c, r, orig).await;
        }
    }
}

async fn logout_and_in(c: &Client) {
    c.send_key(Keycode::Esc);
    while !is_visible_by_path(c, paths::QUIT_BUTTON) { tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; }
    click_window_by_path(c, paths::QUIT_BUTTON).await; tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    if is_visible_by_path(c, paths::DUNGEON_WARNING) { c.send_key(Keycode::Enter); }
    while !is_visible_by_path(c, paths::PLAY_BUTTON) { tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; }
    click_window_by_path(c, paths::PLAY_BUTTON).await; tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
    if c.is_loading() { wait_for_loading_screen(c, 60.0); }
}

pub async fn teleport_to_friend_from_list(c: &Client, n: Option<String>, il: Option<i32>) -> Result<(), String> {
    c.send_key(Keycode::F); tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    if let Some(root) = &c.root_window {
        if let Some(btn) = get_window_from_path(&root.window, &["NewFriendsListWindow", "listFriends", "Friend0"]) {
            let _ = c.mouse_handler.click_window(&btn, false).await; tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if let Some(tp) = get_window_from_path(&root.window, &["wndCharacter", "ButtonLayout", "btnTeleport"]) {
                let _ = c.mouse_handler.click_window(&tp, false).await;
                return Ok(());
            }
        }
    }
    Err("Friend not found or teleport failed".to_string())
}

pub async fn parse_command(clients: Vec<Client>, command_str: &str) -> std::result::Result<(), String> {
    let all_clients = &clients; let mut cmd_str = command_str.replace(", ", ",");
    let check = ["tozone", "to_zone", "waitforzonechange", "wait_for_zone_change"];
    if !check.iter().any(|&s| cmd_str.contains(s)) { cmd_str = cmd_str.replace('_', ""); }
    let split = tokenize(&cmd_str); if split.is_empty() { return Ok(()); }
    let cmd_f = split[0].as_str().to_lowercase();
    match cmd_f.as_str() {
        "kill" | "killbot" | "stop" | "stopbot" | "end" | "exit" => { debug!("Bot Killed"); return Err("CancelledError".to_string()); }
        "sleep" | "wait" | "delay" => { if let Some(arg) = split.last() { if let Ok(s) = arg.as_str().parse::<f64>() { tokio::time::sleep(tokio::time::Duration::from_secs_f64(s)).await; } } }
        "log" | "debug" | "print" => {
            if split.len() >= 3 && split[1].as_str().to_lowercase() == "window" && split[2].is_list() {
                let p_vec = split[2].as_list(); let path: Vec<&str> = p_vec.iter().map(|s| s.as_str()).collect();
                for c in &clients { if let Some(root) = &c.root_window { if let Some(w) = get_window_from_path(&root.window, &path) { debug!("{} - {}", c.title(), w.maybe_text().unwrap_or_default()); } } }
            } else { debug!("{}", split[1..].iter().map(|t| t.as_str()).collect::<Vec<_>>().join(" ")); }
        }
        _ => {
            let mut split = split;
            let mut c_str = split[0].as_str().replace(' ', ""); let mut excl = false;
            if c_str.contains("except") { split.remove(0); c_str = split[0].as_str().to_string(); excl = true; }
            let mut active_clients: Vec<&Client> = all_clients.iter().collect();
            if !c_str.contains("mass") {
                let mut prov = Vec::new();
                if c_str.contains(':') { for t in c_str.split(':') { if let Some(c) = client_from_titles(all_clients, t) { prov.push(c); } } }
                else if let Some(c) = client_from_titles(all_clients, &c_str) { prov.push(c); }
                if !c_str.is_empty() && c_str.chars().nth(1).map_or(false, |c| c.is_numeric()) {
                    if excl { active_clients = all_clients.iter().filter(|c| !prov.iter().any(|pc| pc.title() == c.title())).collect(); }
                    else { active_clients = prov; }
                }
            }
            if split.len() < 2 { return Ok(()); }
            match split[1].as_str().to_lowercase().as_str() {
                "teleport" | "tp" | "setpos" => {
                    if split.len() < 3 { return Ok(()); }
                    match split[2].as_str().to_lowercase().as_str() {
                        "closestmob" | "mob" => { join_all(active_clients.iter().map(|&c| async move { let s = SprintyClient::new(c); let m = s.get_mobs(); if let Some(ent) = s.find_closest_of(&m, false) { if let Some(l) = ent.location { let _ = c.teleport(&l); } } })).await; }
                        "quest" | "questpos" | "questposition" => { if !active_clients.is_empty() { if let Some(qp) = active_clients[0].quest_position() { for c in &active_clients { let _ = c.teleport(&qp); } } } }
                        _ => {
                            let mut loc = None; for p in all_clients { if p.title() == split[2].as_str() { loc = p.body().and_then(|b| b.position().ok()); break; } }
                            if let Some(l) = loc { for c in &active_clients { let _ = c.teleport(&l); } }
                            else {
                                let mut xyzs = Vec::new(); for c in &active_clients { let (cx, _) = parse_location(&split, None, Some(c)).await; xyzs.push(cx.get(0).cloned().unwrap_or_default()); }
                                for (c, x) in active_clients.iter().zip(xyzs) { let _ = c.teleport(&x); }
                            }
                        }
                    }
                }
                "walkto" | "goto" => {
                    let mut xyzs = Vec::new(); for c in &active_clients { let (cx, _) = parse_location(&split, None, Some(c)).await; xyzs.push(cx.get(0).cloned().unwrap_or_default()); }
                    for (c, x) in active_clients.iter().zip(xyzs) { c.goto(x.x, x.y); }
                }
                "sendkey" | "press" | "presskey" => {
                    if split.len() >= 3 {
                        let k_s = split[2].as_str().to_lowercase(); let mut t = 0.1; if split.len() >= 4 { t = split[3].as_str().parse::<f64>().unwrap_or(0.1); }
                        let k = match k_s.as_str() { "w" => Some(Keycode::W), "a" => Some(Keycode::A), "s" => Some(Keycode::S), "d" => Some(Keycode::D), "space" => Some(Keycode::Spacebar), "enter" => Some(Keycode::Enter), _ => None };
                        if let Some(kv) = k { for c in active_clients { c.send_key(kv); } }
                    }
                }
                "waitfordialog" | "waitfordialogue" => {
                    join_all(active_clients.iter().map(|c| async move { wait_for_coro(|| async { c.is_in_dialog() }, false, 0.25).await; })).await;
                    if split.last().unwrap().as_str().to_lowercase() == "completion" { join_all(active_clients.iter().map(|c| async move { wait_for_coro(|| async { c.is_in_dialog() }, true, 0.25).await; })).await; }
                }
                "waitforbattle" | "waitforcombat" => {
                    join_all(active_clients.iter().map(|c| async move { wait_for_coro(|| async { c.in_battle() }, false, 0.25).await; })).await;
                    if split.last().unwrap().as_str().to_lowercase() == "completion" { join_all(active_clients.iter().map(|c| async move { wait_for_coro(|| async { c.in_battle() }, true, 0.25).await; })).await; }
                }
                "waitforzonechange" | "wait_for_zone_change" => {
                    if split.len() >= 4 && split[split.len()-2].as_str().to_lowercase() == "from" { let f = split.last().unwrap().as_str(); join_all(active_clients.iter().map(|c| async move { wait_for_zone_change(c, Some(f), 60.0); })).await; }
                    else if split.len() >= 4 && split[split.len()-2].as_str().to_lowercase() == "to" { let t = split.last().unwrap().as_str(); join_all(active_clients.iter().map(|c| async move { while c.zone_name().unwrap_or_default() != t { tokio::time::sleep(tokio::time::Duration::from_millis(250)).await; } })).await; }
                    else {
                        join_all(active_clients.iter().map(|c| async move { let s = c.zone_name().unwrap_or_default(); while c.zone_name().unwrap_or_default() == s { tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; } })).await;
                        if split.last().unwrap().as_str().to_lowercase() == "completion" { join_all(active_clients.iter().map(|c| async move { wait_for_coro(|| async { c.is_loading() }, true, 0.25).await; })).await; }
                    }
                }
                "waitforfree" => {
                    join_all(active_clients.iter().map(|c| async move { while !is_free(c) { tokio::time::sleep(tokio::time::Duration::from_millis(250)).await; } })).await;
                    if split.last().unwrap().as_str().to_lowercase() == "completion" { join_all(active_clients.iter().map(|c| async move { while is_free(c) { tokio::time::sleep(tokio::time::Duration::from_millis(250)).await; } })).await; }
                }
                "usepotion" => {
                    if split.len() > 3 {
                        let h = split[2].as_str().parse::<i32>().unwrap_or(20); let m = split[3].as_str().parse::<i32>().unwrap_or(10);
                        join_all(active_clients.iter().map(|c| async move { if let Some(st) = c.stats() { use wizwalker::memory::objects::game_stats::GameStats; let hr = st.current_hitpoints().ok().unwrap_or(1) as f32 / st.max_hitpoints().ok().unwrap_or(1) as f32; let mr = st.current_mana().ok().unwrap_or(1) as f32 / st.max_mana().ok().unwrap_or(1) as f32; if hr * 100.0 <= h as f32 || mr * 100.0 <= m as f32 { use_potion(c).await; } } })).await;
                    } else { join_all(active_clients.iter().map(|c| use_potion(c))).await; }
                }
                "buypotions" | "refillpotions" | "buypots" | "refillpots" => {
                            if split.len() > 2 && split[2].as_str() == "ifneeded" { join_all(active_clients.iter().map(|c| async move { if let Some(st) = c.stats() { use wizwalker::memory::objects::game_stats::GameStats; if st.potion_charge().ok().unwrap_or(0.0) < 1.0 { refill_potions(c, false, true).await; } } })).await; }
                    else { join_all(active_clients.iter().map(|c| refill_potions(c, true, true))).await; }
                }
                "logoutandin" | "relog" => { join_all(active_clients.iter().map(|c| logout_and_in(c))).await; }
                "click" => {
                    let split_clone = split.clone();
                    join_all(active_clients.iter().map(|c| {
                        let split_inner = split_clone.clone();
                        async move {
                            let x = split_inner[2].as_str().parse::<i32>().unwrap_or(0);
                            let y = split_inner[3].as_str().parse::<i32>().unwrap_or(0);
                            let _ = c.mouse_handler.click(x, y, false, 0.0, false).await;
                        }
                    })).await;
                }
                "clickwindow" => {
                    if split.len() >= 3 && split[2].is_list() {
                        let p_v = split[2].as_list();
                        join_all(active_clients.iter().map(|c| {
                            let p_v_inner = p_v.clone();
                            async move {
                                let p_refs: Vec<&str> = p_v_inner.iter().map(|s| s.as_str()).collect();
                                click_window_by_path(c, &p_refs).await;
                            }
                        })).await;
                    }
                }
                "waitforwindow" | "waitforpath" => {
                    if split.len() >= 3 && split[2].is_list() {
                        let p_v = split[2].as_list();
                        let p_v_clone = p_v.clone();
                        join_all(active_clients.iter().map(|c| {
                            let p_v_inner = p_v_clone.clone();
                            async move {
                                let p_refs: Vec<&str> = p_v_inner.iter().map(|s| s.as_str()).collect();
                                while !is_visible_by_path(c, &p_refs) { tokio::time::sleep(tokio::time::Duration::from_millis(250)).await; }
                            }
                        })).await;
                        if split.last().unwrap().as_str().to_lowercase() == "completion" {
                            let p_v_clone2 = p_v.clone();
                            join_all(active_clients.iter().map(|c| {
                                let p_v_inner = p_v_clone2.clone();
                                async move {
                                    let p_refs: Vec<&str> = p_v_inner.iter().map(|s| s.as_str()).collect();
                                    while is_visible_by_path(c, &p_refs) { tokio::time::sleep(tokio::time::Duration::from_millis(250)).await; }
                                }
                            })).await;
                        }
                    }
                }
                "friendtp" | "friendteleport" => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                    if split.len() >= 3 && split[2].as_str() == "icon" {
                        join_all(active_clients.iter().map(|c| async move {
                            let _ = teleport_to_friend_from_list(c, None, Some(2)).await;
                        })).await;
                    } else {
                        let n = split[2..].iter().map(|t| t.as_str()).collect::<Vec<_>>().join(" ");
                        join_all(active_clients.iter().map(|c| {
                            let n_inner = n.clone();
                            async move {
                                let _ = teleport_to_friend_from_list(c, Some(n_inner), None).await;
                            }
                        })).await;
                    }
                }
                "entitytp" | "entityteleport" => { if split.len() >= 3 { let n = split[2].as_str(); join_all(active_clients.iter().map(|c| async move { let s = SprintyClient::new(c); let e = s.get_entities_with_vague_name(n, None); if let Some(ent) = s.find_closest_of(&e, false) { if let Some(l) = ent.location { let _ = c.teleport(&l); } } })).await; } }
                "tozone" | "to_zone" => { if split.len() >= 3 { let z = split[2].as_str(); join_all(active_clients.iter().map(|c| async move { while c.zone_name().unwrap_or_default() != z { tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await; } })).await; } }
                _ => { tokio::time::sleep(tokio::time::Duration::from_millis(250)).await; }
            }
        }
    }
    Ok(())
}

pub async fn execute_flythrough(c: &Client, data: &str, sep: &str) -> Result<(), String> {
    let actions: Vec<&str> = data.split(sep).collect(); let mut cmds = Vec::new();
    for s in actions {
        let toks = tokenize(s); if toks.is_empty() { continue; }
        if ["webpage", "pull", "embed"].contains(&toks[0].as_str().to_lowercase().as_str()) {
            if let Ok(resp) = reqwest::blocking::get(toks[1].as_str()) { if let Ok(text) = resp.text() { for l in text.lines() { cmds.push(l.to_string()); } } }
        } else { cmds.push(s.to_string()); }
    }
    if let Some(gc) = c.game_client() { if let Ok(f) = gc.is_freecam() { if !f { c.camera_freecam(); } } }
    if let Some(gc) = c.game_client() { if let Some(cam_ptr) = gc.free_camera_controller().ok().flatten() {
        use wizwalker::memory::memory_object::DynamicMemoryObject;
        if let Some(reader) = c.reader() {
            if let Ok(inner) = DynamicMemoryObject::new(reader, cam_ptr) {
                let cam = DynamicCameraController::new(inner);
                for a in cmds { parse_camera_command(&cam, &a).await; }
            }
        }
    } }
    Ok(())
}

pub async fn parse_camera_command(cam: &DynamicCameraController, s: &str) {
    let mut s = s.replace(", ", ","); s = s.replace('_', ""); let split = tokenize(&s); if split.is_empty() { return; }
    let orig_p = cam.position().unwrap_or_default(); let orig_o = cam.orientation().unwrap_or_default();
    let (xyzs, orients) = parse_location(&split, Some(cam), None).await;
    let mut t = 0.0; if let Some(last) = split.last() { if let Ok(v) = last.as_str().parse::<f32>() { t = v; } }
    let cmd = split[0].as_str().to_lowercase();
    match cmd.as_str() {
        "glideto" => {
            let t_xyz = handle_index(&xyzs, 0, None).unwrap_or_default(); let t_o = handle_index(&orients, 0, Some(orig_o)).unwrap(); let f_xyz = handle_index(&xyzs, 1, None);
            let glide = GlideParams::new(&orig_p, &t_xyz, t); let start = std::time::Instant::now();
            while start.elapsed().as_secs_f32() < t { let dt = start.elapsed().as_secs_f32(); let c_xyz = glide.position_at(&orig_p, dt); let _ = cam.write_position(&c_xyz); if let Some(f) = f_xyz { let _ = cam.update_orientation(Some(point_to_xyz_orient(&c_xyz, &f))); } else { let _ = cam.update_orientation(Some(t_o)); } tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; }
            let _ = cam.write_position(&t_xyz);
        }
        "rotatingglideto" => {
            let t_xyz = handle_index(&xyzs, 0, None).unwrap_or_default(); let t_o = handle_index(&orients, 0, Some(Orient::default())).unwrap();
            let glide = GlideParams::new(&orig_p, &t_xyz, t); let rot_v = rotation_velocity(&t_o, t); let start = std::time::Instant::now(); let mut c_o = orig_o;
            while start.elapsed().as_secs_f32() < t { let _ = cam.write_position(&glide.position_at(&orig_p, start.elapsed().as_secs_f32())); c_o.pitch += rot_v.pitch * 0.01; c_o.roll += rot_v.roll * 0.01; c_o.yaw += rot_v.yaw * 0.01; let _ = cam.update_orientation(Some(c_o)); tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; }
            let _ = cam.write_position(&t_xyz);
        }
        "orbit" => {
            let t_xyz = handle_index(&xyzs, 0, None).unwrap_or_default(); let deg = if split.len() >= 2 { param_input(split[split.len()-2].as_str(), 360.0) } else { 360.0 };
            let orb = OrbitParams::new(&orig_p, &t_xyz, deg, t); let start = std::time::Instant::now();
            while start.elapsed().as_secs_f32() < t { let dt = start.elapsed().as_secs_f32(); let c_xyz = orb.position_at(dt); let _ = cam.write_position(&c_xyz); let _ = cam.update_orientation(Some(orb.orientation_at(&c_xyz, orig_o.roll))); tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; }
        }
        "lookat" => { if let Some(t_x) = handle_index(&xyzs, 0, None) { let _ = cam.update_orientation(Some(point_to_xyz_orient(&orig_p, &t_x))); } }
        "setpos" => { if let Some(t_x) = handle_index(&xyzs, 0, None) { let _ = cam.write_position(&t_x); } }
        "setorient" => { if let Some(t_o) = handle_index(&orients, 0, None) { let _ = cam.update_orientation(Some(t_o)); } }
        _ => {}
    }
}
