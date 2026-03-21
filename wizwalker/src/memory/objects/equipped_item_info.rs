use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject};

pub struct EquippedItemInfo {
    inner: DynamicMemoryObject,
}

impl EquippedItemInfo {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }

    pub fn item_id(&self) -> Result<u32> {
        self.read_value_from_offset(72)
    }

    pub fn write_item_id(&self, val: u32) -> Result<()> {
        self.write_value_to_offset(72, &val)
    }
}

impl MemoryObject for EquippedItemInfo {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}
