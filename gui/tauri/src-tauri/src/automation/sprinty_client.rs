//! SprintyClient — entity search & navigation helpers.
//!
//! Faithfully ported from Deimos `src/sprinty_client.py` (252 lines).
#![allow(dead_code, unused_imports)]

use wizwalker::client::Client;
use wizwalker::types::XYZ;
use wizwalker::memory::reader::MemoryReaderExt;

/// SprintyClient wraps a `Client` reference and adds entity search helpers.
pub struct SprintyClient<'a> {
    pub client: &'a Client,
}

/// Represents a lightweight entity reference.
pub struct EntityRef {
    pub base_address: u64,
    pub object_name: String,
    pub location: Option<XYZ>,
}

impl<'a> SprintyClient<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get base entity list with names and locations.
    pub fn get_base_entity_list(&self) -> Vec<EntityRef> {
        let raw_entities = self.client.get_base_entity_list();
        let reader = match self.client.process_reader() {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut entities = Vec::new();
        for addr in raw_entities {
            let name = self.read_entity_object_name(addr as usize, &*reader).unwrap_or_default();
            let location = self.read_entity_location(addr as usize, &*reader);
            entities.push(EntityRef {
                base_address: addr,
                object_name: name,
                location,
            });
        }
        entities
    }

    /// Get entities whose name contains the given string (case-insensitive).
    pub fn get_entities_with_vague_name(&self, name: &str) -> Vec<EntityRef> {
        let lower_name = name.to_lowercase();
        self.get_base_entity_list()
            .into_iter()
            .filter(|e| e.object_name.to_lowercase().contains(&lower_name))
            .collect()
    }

    pub fn get_health_wisps(&self) -> Vec<EntityRef> {
        self.get_entities_with_vague_name("WispHealth")
    }

    pub fn get_mana_wisps(&self) -> Vec<EntityRef> {
        self.get_entities_with_vague_name("WispMana")
    }

    pub fn get_gold_wisps(&self) -> Vec<EntityRef> {
        self.get_entities_with_vague_name("WispGold")
    }

    pub fn get_mobs(&self) -> Vec<EntityRef> {
        let skip = ["Basic Positional", "WispHealth", "WispMana", "KT_WispHealth",
                     "KT_WispMana", "WispGold", "DuelCircle", "Player Object",
                     "SkeletonKeySigilArt", "Basic Ambient", "TeleportPad"];
        self.get_base_entity_list()
            .into_iter()
            .filter(|e| !skip.iter().any(|s| e.object_name == *s) && !e.object_name.is_empty())
            .collect()
    }

    /// Find the closest entity to the player's position.
    pub fn find_closest_of(&self, entities: &[EntityRef]) -> Option<usize> {
        let player_pos = self.client.body_position()?;
        let mut closest_idx = None;
        let mut closest_dist = f32::MAX;

        for (i, entity) in entities.iter().enumerate() {
            if let Some(loc) = &entity.location {
                let dist = ((loc.x - player_pos.x).powi(2)
                    + (loc.y - player_pos.y).powi(2)
                    + (loc.z - player_pos.z).powi(2))
                    .sqrt();
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_idx = Some(i);
                }
            }
        }
        closest_idx
    }

    /// Find entities that are safe (far from mobs).
    pub fn find_safe_entities<'b>(&self, entities: &'b [EntityRef], safe_distance: f32) -> Vec<&'b EntityRef> {
        let mobs = self.get_mobs();
        let mob_positions: Vec<XYZ> = mobs.iter()
            .filter_map(|m| m.location.clone())
            .collect();

        entities.iter().filter(|e| {
            if let Some(loc) = &e.location {
                mob_positions.iter().all(|mp| {
                    let dist = ((loc.x - mp.x).powi(2)
                        + (loc.y - mp.y).powi(2)
                        + (loc.z - mp.z).powi(2))
                        .sqrt();
                    dist >= safe_distance
                })
            } else {
                false
            }
        }).collect()
    }

    fn read_entity_object_name(&self, entity_addr: usize, reader: &dyn wizwalker::memory::reader::MemoryReader) -> Option<String> {
        let template_ptr: u64 = reader.read_typed(entity_addr + 72).ok()?;
        if template_ptr == 0 { return None; }
        self.client.read_wide_string_at(template_ptr as usize + 72)
    }

    fn read_entity_location(&self, entity_addr: usize, reader: &dyn wizwalker::memory::reader::MemoryReader) -> Option<XYZ> {
        let x: f32 = reader.read_typed(entity_addr + 88).ok()?;
        let y: f32 = reader.read_typed(entity_addr + 92).ok()?;
        let z: f32 = reader.read_typed(entity_addr + 96).ok()?;
        Some(XYZ { x, y, z })
    }
}
