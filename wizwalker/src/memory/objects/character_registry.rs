use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject};

pub trait CharacterRegistry: MemoryObject {
    fn active_quest_id(&self) -> Result<u64> {
        self.read_value_from_offset(304)
    }

    fn write_active_quest_id(&self, active_quest_id: u64) -> Result<()> {
        self.write_value_to_offset(304, &active_quest_id)
    }

    fn active_goal_id(&self) -> Result<u32> {
        self.read_value_from_offset(336)
    }

    fn write_active_goal_id(&self, active_goal_id: u32) -> Result<()> {
        self.write_value_to_offset(336, &active_goal_id)
    }
}

pub struct DynamicCharacterRegistry {
    inner: DynamicMemoryObject,
}

impl DynamicCharacterRegistry {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::memory_object::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }
}

impl MemoryObject for DynamicCharacterRegistry {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::memory_object::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl CharacterRegistry for DynamicCharacterRegistry {}
