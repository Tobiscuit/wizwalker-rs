use crate::errors::Result;
use crate::types::XYZ;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::fish::Fish;

pub struct FishingManager {
    pub inner: DynamicMemoryObject,
}

impl FishingManager {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn fish_list(&self) -> Result<Vec<Fish>> {
        let addrs = self.inner.read_linked_list::<u64, _>(0x1C0, |_| Ok(0))?;
        let mut result = Vec::new();
        // let reader = self.inner.reader();
        for _addr in addrs {
            // let inner = DynamicMemoryObject::new(reader.clone(), addr)?;
            // result.push(Fish::new(inner));
        }
        Ok(result)
    }

    pub async fn _bobber_pos(&self) -> Result<XYZ> {
        self.inner.read_value_from_offset(0x390)
    }
}
