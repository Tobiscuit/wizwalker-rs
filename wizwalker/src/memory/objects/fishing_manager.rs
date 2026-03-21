use crate::errors::Result;
use crate::types::XYZ;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject, MemoryObjectExt};
use super::fish::Fish;

pub struct FishingManager {
    pub inner: DynamicMemoryObject,
}

impl FishingManager {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn fish_list(&self) -> Result<Vec<Fish>> {
        let addrs = self.inner.read_linked_list(0x1C0)?;
        let result = Vec::new();
        // let reader = self.inner.reader();
        for _addr in addrs {
            // let inner = DynamicMemoryObject::new(reader.clone(), addr)?;
            // result.push(Fish::new(inner));
        }
        Ok(result)
    }

    pub fn _bobber_pos(&self) -> Result<XYZ> {
        self.inner.read_value_from_offset(0x390)
    }
}
