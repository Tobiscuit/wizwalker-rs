//! Automation module — Deimos automation features ported to Rust.
//!
//! This module contains the faithful 1:1 port of Deimos-Wizard101's
//! automation features, including:
//! - Window path navigation (utils.rs)
//! - UI window path constants (paths.rs)
//! - Auto dialogue (dialogue.rs)
//! - Auto sigil (sigil.rs)
//! - Anti-AFK (anti_afk.rs)
//! - SprintyClient entity helpers (sprinty_client.rs)
//! - Auto pet training (auto_pet.rs)
//! - Auto questing (questing.rs)
//! - Teleport math & navmap pathfinding (teleport_math.rs)
//! - Camera animation utilities (camera_utils.rs)
//! - Collision world parser (collision.rs)
//! - Combat object/school helpers (combat_objects.rs)
//! - Combat math engine (combat_math.rs)
//! - Combat stat utilities (combat_utils.rs)
//! - Combat state cache (combat_cache.rs)
//! - Stat viewer display (stat_viewer.rs)
//! - Deck token encoder/decoder (deck_encoder.rs)

pub mod paths;
pub mod utils;
pub mod dialogue;
pub mod sigil;
pub mod anti_afk;
pub mod sprinty_client;
pub mod auto_pet;
pub mod questing;
pub mod teleport_math;
pub mod camera_utils;
pub mod collision;
pub mod combat_objects;
pub mod combat_math;
pub mod combat_utils;
pub mod combat_cache;
pub mod stat_viewer;
pub mod deck_encoder;
