use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject};

pub struct EquipmentSet {
    inner: DynamicMemoryObject,
}

impl EquipmentSet {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }

    pub fn equipment_set_name(&self) -> Result<u32> {
        self.read_value_from_offset(88)
    }

    pub fn write_equipment_set_name(&self, val: u32) -> Result<()> {
        self.write_value_to_offset(88, &val)
    }

    pub fn is_equipped(&self) -> Result<bool> {
        self.read_value_from_offset(128)
    }

    pub fn write_is_equipped(&self, val: bool) -> Result<()> {
        self.write_value_to_offset(128, &val)
    }
}

impl MemoryObject for EquipmentSet {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}
