//! Drop Logger — Ported from Deimos `src/drop_logger.py`.
//!
//! Handles chat window reading and filtering for item drops.

use wizwalker::client::Client;
use crate::automation::utils::{is_visible_by_path, get_window_from_path};
use crate::automation::paths;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref DROP_REGEX_1: Regex = Regex::new(r"(?<=> <).*|$").unwrap();
    static ref DROP_REGEX_2: Regex = Regex::new(r"(?<=;).*?[^>]*|$").unwrap();
    static ref DROP_REGEX_3: Regex = Regex::new(r">.*?<|$").unwrap();
    static ref DROP_REGEX_4: Regex = Regex::new(r"[^>]+[^<]+|$").unwrap();
    static ref DROP_REGEX_COLON: Regex = Regex::new(r"(?<=:).*?[^<]*|$").unwrap();
}

pub const DROP_TYPES: &[&str] = &[
    "PetSnack", "Reagent", "Housing", "Pet", "Shoes", "Seed", "Jewel",
    "Robe", "Hat", "Athame", "Weapon", "Deck", "Ring", "Amulet",
];

pub async fn get_chat(client: &Client) -> String {
    if is_visible_by_path(client, paths::CHAT_WINDOW) {
        let root = match &client.root_window {
            Some(rw) => &rw.window,
            None => return String::new(),
        };
        if let Some(chat_window) = get_window_from_path(root, paths::CHAT_WINDOW) {
            return chat_window.maybe_text().unwrap_or_default();
        }
    }
    String::new()
}

pub fn filter_drops(input_list: Vec<String>) -> Vec<String> {
    let mut drops = Vec::new();

    for raw_i in input_list {
        if raw_i.contains("Art_Chat_System.dds") {
            let matches = DROP_REGEX_1.find(&raw_i).map(|m| m.as_str()).unwrap_or("");
            if !matches.is_empty() {
                let mut drop_type = "";
                if matches.contains(';') {
                    drop_type = DROP_REGEX_2.find(matches).map(|m| m.as_str()).unwrap_or("");
                }

                if DROP_TYPES.iter().any(|&t| t == drop_type) {
                    let raw_drop = DROP_REGEX_3.find(matches).map(|m| m.as_str()).unwrap_or("");
                    let mut drop = DROP_REGEX_4.find(raw_drop).map(|m| m.as_str()).unwrap_or("");
                    let drop_str = drop.replacen(' ', "", 1);
                    drops.push(drop_str);
                }
            } else if raw_i.to_lowercase().contains(':') {
                let mut drop = DROP_REGEX_COLON.find(&raw_i).map(|m| m.as_str()).unwrap_or("");
                let drop_str = drop.replacen(' ', "", 1);
                drops.push(drop_str);
            }
        }
    }

    drops
}

pub fn find_new_stuff(old: &str, new: &str) -> String {
    let mut old_mut = old.to_string();
    let mut found_idx: Option<usize> = None;

    while !old_mut.is_empty() {
        if let Some(idx) = new.find(&old_mut) {
            found_idx = Some(idx);
            break;
        }
        old_mut.remove(0);
    }

    match found_idx {
        Some(idx) => new[idx + old_mut.len()..].to_string(),
        None => new.to_string(),
    }
}
