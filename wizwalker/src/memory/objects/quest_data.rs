use crate::errors::{Result, WizWalkerError};
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::goal_data::GoalData;
use super::client_tag_list::ClientTagList;
use std::collections::HashMap;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityType {
    NotActivity = 0,
    Spell = 1,
    Crafting = 2,
    Fishing = 3,
    Gardening = 4,
    Pet = 5,
}

pub struct QuestData {
    pub inner: DynamicMemoryObject,
}

impl QuestData {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn goal_data(&self) -> Result<HashMap<i32, GoalData>> {
        Ok(HashMap::new())
    }

    pub fn client_tags(&self) -> Result<Option<ClientTagList>> {
        let addr: u64 = self.inner.read_value_from_offset(0xC0)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(ClientTagList::new(inner)))
    }

    pub fn quest_type(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0xE0)
    }

    pub fn quest_level(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0xE4)
    }

    pub fn permit_quest_helper(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0xF0)
    }

    pub fn write_permit_quest_helper(&self, val: bool) -> Result<()> {
        self.inner.write_value_to_offset(0xF0, &val)
    }

    pub fn mainline(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0xF1)
    }

    pub fn ready_to_turn_in(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0xF2)
    }

    pub fn activity_type(&self) -> Result<ActivityType> {
        let val: i32 = self.inner.read_value_from_offset(0xF4)?;
        match val {
            0 => Ok(ActivityType::NotActivity),
            1 => Ok(ActivityType::Spell),
            2 => Ok(ActivityType::Crafting),
            3 => Ok(ActivityType::Fishing),
            4 => Ok(ActivityType::Gardening),
            5 => Ok(ActivityType::Pet),
            _ => Err(WizWalkerError::ReadingEnumFailed { enum_name: "ActivityType".into(), value: val.to_string() }),
        }
    }

    pub fn pet_only_quest(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0xF9)
    }
}
