use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

pub struct FishTemplate {
    pub inner: DynamicMemoryObject,
}

impl FishTemplate {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn rank(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0xA0)
    }

    pub async fn size_min(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0xC8)
    }

    pub async fn size_max(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0xCC)
    }

    pub async fn is_sentinel(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0xD0)
    }

    pub async fn bobber_submerge_ease(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x108)
    }

    pub async fn write_bobber_submerge_ease(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x108, &val)
    }
}
