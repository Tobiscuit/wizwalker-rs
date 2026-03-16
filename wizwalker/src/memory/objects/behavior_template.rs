use crate::errors::{Result, WizWalkerError};
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};
use crate::types::Color;

pub trait BehaviorTemplate: MemoryObject {
    fn behavior_name(&self) -> Result<String> {
        self.read_string_from_offset(72)
    }

    fn write_behavior_name(&self, behavior_name: &str) -> Result<()> {
        self.write_string_to_offset(72, behavior_name)
    }
}

pub struct DynamicBehaviorTemplate {
    inner: DynamicMemoryObject,
}

impl DynamicBehaviorTemplate {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::memory_object::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }
}

impl MemoryObject for DynamicBehaviorTemplate {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::memory_object::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorTemplate for DynamicBehaviorTemplate {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum NpcBehaviorTemplateTitleType {
    Easy = 0,
    Normal = 1,
    Elite = 2,
    Boss = 3,
    Minion = 4,
}

impl TryFrom<i32> for NpcBehaviorTemplateTitleType {
    type Error = ();
    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(NpcBehaviorTemplateTitleType::Easy),
            1 => Ok(NpcBehaviorTemplateTitleType::Normal),
            2 => Ok(NpcBehaviorTemplateTitleType::Elite),
            3 => Ok(NpcBehaviorTemplateTitleType::Boss),
            4 => Ok(NpcBehaviorTemplateTitleType::Minion),
            _ => Err(()),
        }
    }
}

impl Into<i32> for NpcBehaviorTemplateTitleType {
    fn into(self) -> i32 {
        self as i32
    }
}

pub struct NPCBehaviorTemplate {
    inner: DynamicMemoryObject,
}

impl NPCBehaviorTemplate {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::memory_object::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }

    pub fn starting_health(&self) -> Result<i32> {
        self.read_value_from_offset(120)
    }

    pub fn hide_current_hp(&self) -> Result<bool> {
        self.read_value_from_offset(124)
    }

    pub fn level(&self) -> Result<i32> {
        self.read_value_from_offset(128)
    }

    pub fn intelligence(&self) -> Result<f32> {
        self.read_value_from_offset(132)
    }

    pub fn selfish_factor(&self) -> Result<bool> {
        self.read_value_from_offset(136)
    }

    pub fn aggressive_factor(&self) -> Result<i32> {
        self.read_value_from_offset(140)
    }

    pub fn boss_mob(&self) -> Result<bool> {
        self.read_value_from_offset(144)
    }

    pub fn turn_towards_player(&self) -> Result<bool> {
        self.read_value_from_offset(145)
    }

    pub fn mob_title(&self) -> Result<NpcBehaviorTemplateTitleType> {
        self.read_enum(148)
    }

    pub fn name_color(&self) -> Result<Color> {
        self.read_color(152)
    }

    pub fn write_name_color(&self, val: &Color) -> Result<()> {
        self.write_color(152, val)
    }

    pub fn school_of_focus(&self) -> Result<String> {
        self.read_string_from_offset(160)
    }

    pub fn secondary_school_of_focus(&self) -> Result<String> {
        self.read_string_from_offset(200)
    }

    pub fn cylinder_scale_value(&self) -> Result<f32> {
        self.read_value_from_offset(268)
    }

    pub fn max_shadow_pips(&self) -> Result<i32> {
        self.read_value_from_offset(272)
    }
}

impl MemoryObject for NPCBehaviorTemplate {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::memory_object::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorTemplate for NPCBehaviorTemplate {}
