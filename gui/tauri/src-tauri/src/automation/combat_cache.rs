//! Combat cache — dot-path get/set/filter for snapshotting combat state.
//!
//! Faithfully ported from `deimos-reference/src/combat_cache.py`.
//!
//! Uses `serde_json::Value` as a dynamic cache value type so we can store
//! arbitrary nested data (dicts, lists, scalars) like Python's `Dict[str, Any]`.
#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use serde_json::Value;

/// A cache is a JSON object mapping string keys to arbitrary values.
pub type Cache = serde_json::Map<String, Value>;

/// Retrieve a value from a cache using a dot-separated path.
///
/// Supports numeric indexes for arrays: `"get_participant.hanging_effects.0"`.
///
/// Python: `cache_get(cache, path)` — combat_cache.py:5
pub fn cache_get<'a>(cache: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = cache;
    for part in path.split('.') {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Retrieve multiple values from a cache using multiple dot-separated paths.
///
/// Python: `cache_get_multi(cache, paths)` — combat_cache.py:21
pub fn cache_get_multi<'a>(cache: &'a Value, paths: &[&str]) -> Vec<Option<&'a Value>> {
    paths.iter().map(|p| cache_get(cache, p)).collect()
}

/// Modify a value in a cache by dot-separated path.
///
/// Python: `cache_modify(cache, new_value, path_str)` — combat_cache.py:47
pub fn cache_modify(cache: &mut Value, new_value: Value, path: &str) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return false;
    }

    let mut current = cache;
    for part in &parts[..parts.len() - 1] {
        current = match current {
            Value::Object(map) => {
                if !map.contains_key(*part) {
                    return false;
                }
                map.get_mut(*part).unwrap()
            }
            Value::Array(arr) => {
                let idx: usize = match part.parse() {
                    Ok(i) => i,
                    Err(_) => return false,
                };
                match arr.get_mut(idx) {
                    Some(v) => v,
                    None => return false,
                }
            }
            _ => return false,
        };
    }

    let last = parts.last().unwrap();
    match current {
        Value::Object(map) => {
            map.insert(last.to_string(), new_value);
            true
        }
        Value::Array(arr) => {
            if let Ok(idx) = last.parse::<usize>() {
                if idx < arr.len() {
                    arr[idx] = new_value;
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Remove an entry from a cache by dot-separated path.
///
/// Python: `cache_remove(cache, path_str)` — combat_cache.py:26
pub fn cache_remove(cache: &mut Value, path: &str) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return false;
    }

    let mut current = cache;
    for part in &parts[..parts.len() - 1] {
        current = match current {
            Value::Object(map) => match map.get_mut(*part) {
                Some(v) => v,
                None => return false,
            },
            Value::Array(arr) => {
                let idx: usize = match part.parse() {
                    Ok(i) => i,
                    Err(_) => return false,
                };
                match arr.get_mut(idx) {
                    Some(v) => v,
                    None => return false,
                }
            }
            _ => return false,
        };
    }

    let last = parts.last().unwrap();
    match current {
        Value::Object(map) => {
            map.remove(*last).is_some()
        }
        Value::Array(arr) => {
            if let Ok(idx) = last.parse::<usize>() {
                if idx < arr.len() {
                    arr.remove(idx);
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Filter caches by matching dot-path → value pairs.
///
/// Returns (matching_caches, matching_indexes).
/// When `exclusive` is true, returns only NON-matching caches instead.
///
/// Python: `filter_caches(caches, match, exclusive, either_or)` — combat_cache.py:68
pub fn filter_caches(
    caches: &[Value],
    match_map: &HashMap<String, Value>,
    exclusive: bool,
    either_or: bool,
) -> (Vec<Value>, Vec<usize>) {
    let mut matches = Vec::new();
    let mut match_indices = Vec::new();

    for (i, cache) in caches.iter().enumerate() {
        let results: Vec<bool> = match_map
            .iter()
            .map(|(path, expected)| {
                let actual = cache_get(cache, path);
                let matched = actual.map_or(false, |v| v == expected);
                // Flip logic if exclusive
                if exclusive { !matched } else { matched }
            })
            .collect();

        let passes = if either_or {
            results.iter().any(|r| *r)
        } else {
            results.iter().all(|r| *r)
        };

        if passes {
            matches.push(cache.clone());
            match_indices.push(i);
        }
    }

    (matches, match_indices)
}
