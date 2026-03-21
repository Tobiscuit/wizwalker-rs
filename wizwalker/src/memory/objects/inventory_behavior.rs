use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};
use super::behavior_instance::BehaviorInstance;

pub struct InventoryBehaviorBase {
    inner: DynamicMemoryObject,
}

impl InventoryBehaviorBase {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }

    pub fn item_list(&self) -> Result<Vec<DynamicMemoryObject>> {
        let addrs = self.read_shared_linked_list(112)?;
        let mut result = Vec::new();
        for addr in addrs {
            if addr != 0 {
                result.push(DynamicMemoryObject::new(self.reader(), addr)?);
            }
        }
        Ok(result)
    }
}

impl MemoryObject for InventoryBehaviorBase {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorInstance for InventoryBehaviorBase {}

pub struct ClientInventoryBehavior {
    inner: InventoryBehaviorBase,
}

impl ClientInventoryBehavior {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: InventoryBehaviorBase::new(reader, base_address)?,
        })
    }
}

impl std::ops::Deref for ClientInventoryBehavior {
    type Target = InventoryBehaviorBase;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl MemoryObject for ClientInventoryBehavior {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorInstance for ClientInventoryBehavior {}

pub struct ClientWizInventoryBehavior {
    inner: ClientInventoryBehavior,
}

impl ClientWizInventoryBehavior {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: ClientInventoryBehavior::new(reader, base_address)?,
        })
    }

    pub fn num_items_allowed(&self) -> Result<i32> {
        self.read_value_from_offset(160)
    }

    pub fn write_num_items_allowed(&self, val: i32) -> Result<()> {
        self.write_value_to_offset(160, &val)
    }

    pub fn num_jewels_allowed(&self) -> Result<i32> {
        self.read_value_from_offset(164)
    }

    pub fn write_num_jewels_allowed(&self, val: i32) -> Result<()> {
        self.write_value_to_offset(164, &val)
    }

    pub fn num_ce_emotes_allowed(&self) -> Result<i32> {
        self.read_value_from_offset(168)
    }

    pub fn write_num_ce_emotes_allowed(&self, val: i32) -> Result<()> {
        self.write_value_to_offset(168, &val)
    }

    pub fn num_ce_teleports_allowed(&self) -> Result<i32> {
        self.read_value_from_offset(172)
    }

    pub fn write_num_ce_teleports_allowed(&self, val: i32) -> Result<()> {
        self.write_value_to_offset(172, &val)
    }
}

impl std::ops::Deref for ClientWizInventoryBehavior {
    type Target = ClientInventoryBehavior;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl MemoryObject for ClientWizInventoryBehavior {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorInstance for ClientWizInventoryBehavior {}
