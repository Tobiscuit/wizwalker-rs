//! Automation module — Deimos automation features ported to Rust.
//!
//! This module contains the faithful 1:1 port of Deimos-Wizard101's
//! automation features, including:
//! - Window path navigation (utils.rs)
//! - UI window path constants (paths.rs)
//! - Auto combat (combat.rs)
//! - Auto dialogue (dialogue.rs)
//! - Auto sigil (sigil.rs)
//! - Anti-AFK (anti_afk.rs)

pub mod paths;
pub mod utils;
pub mod dialogue;
pub mod sigil;
pub mod anti_afk;
