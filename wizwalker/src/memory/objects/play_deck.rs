use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject};

pub trait PlayDeck: MemoryObject {
}

pub trait PlaySpellData: MemoryObject {
    fn template_id(&self) -> u32 {
        self.read_value_from_offset(72).unwrap_or(0)
    }

    fn enchantment(&self) -> u32 {
        self.read_value_from_offset(76).unwrap_or(0)
    }
}

pub struct DynamicPlayDeck {
    pub inner: DynamicMemoryObject,
}

impl DynamicPlayDeck {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }
}

impl MemoryObject for DynamicPlayDeck {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }
    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl PlayDeck for DynamicPlayDeck {}
