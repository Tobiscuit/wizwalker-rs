use crate::errors::Result;
use crate::types::XYZ;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

pub struct TeleportHelper {
    pub inner: DynamicMemoryObject,
}

impl TeleportHelper {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn position(&self) -> Result<XYZ> {
        self.inner.read_value_from_offset(0)
    }

    pub fn write_position(&self, xyz: &XYZ) -> Result<()> {
        self.inner.write_value_to_offset(0, xyz)
    }

    pub fn should_update(&self) -> Result<bool> {
        self.inner.read_value_from_offset(12)
    }

    pub fn write_should_update(&self, should_update: bool) -> Result<()> {
        self.inner.write_value_to_offset(12, &should_update)
    }

    pub fn target_object_address(&self) -> Result<u64> {
        self.inner.read_value_from_offset(13)
    }

    pub fn write_target_object_address(&self, target_object_address: u64) -> Result<()> {
        self.inner.write_value_to_offset(13, &target_object_address)
    }
}
