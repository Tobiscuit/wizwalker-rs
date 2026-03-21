use crate::errors::Result;
use crate::types::XYZ;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

pub struct CurrentQuestPosition {
    pub inner: DynamicMemoryObject,
}

impl CurrentQuestPosition {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn position(&self) -> Result<XYZ> {
        self.inner.read_value_from_offset(0)
    }

    pub fn write_position(&self, position: &XYZ) -> Result<()> {
        self.inner.write_value_to_offset(0, position)
    }
}
