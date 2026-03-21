use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};
use super::behavior_instance::BehaviorInstance;
use super::equipped_slot_info::EquippedSlotInfo;
use super::equipped_item_info::EquippedItemInfo;
use super::equipment_set::EquipmentSet;

pub struct ClientEquipmentBehavior {
    inner: DynamicMemoryObject,
}

impl ClientEquipmentBehavior {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }

    pub fn item_list(&self) -> Result<Vec<DynamicMemoryObject>> {
        let addrs = self.read_shared_linked_list(120)?;
        let mut result = Vec::new();
        for addr in addrs {
            if addr != 0 {
                result.push(DynamicMemoryObject::new(self.reader(), addr)?);
            }
        }
        Ok(result)
    }

    pub fn slot_list(&self) -> Result<Vec<EquippedSlotInfo>> {
        let addrs = self.read_shared_linked_list(136)?;
        let mut result = Vec::new();
        for addr in addrs {
            if addr != 0 {
                result.push(EquippedSlotInfo::new(self.reader(), addr)?);
            }
        }
        Ok(result)
    }

    pub fn public_item_list(&self) -> Result<Vec<EquippedItemInfo>> {
        let addrs = self.read_shared_linked_list(152)?;
        let mut result = Vec::new();
        for addr in addrs {
            if addr != 0 {
                result.push(EquippedItemInfo::new(self.reader(), addr)?);
            }
        }
        Ok(result)
    }
}

impl MemoryObject for ClientEquipmentBehavior {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorInstance for ClientEquipmentBehavior {}

pub struct ClientWizEquipmentBehavior {
    inner: ClientEquipmentBehavior,
}

impl ClientWizEquipmentBehavior {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::reader::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: ClientEquipmentBehavior::new(reader, base_address)?,
        })
    }

    pub fn equipment_sets(&self) -> Result<Vec<EquipmentSet>> {
        let addrs = self.read_shared_linked_list(232)?;
        let mut result = Vec::new();
        for addr in addrs {
            if addr != 0 {
                result.push(EquipmentSet::new(self.reader(), addr)?);
            }
        }
        Ok(result)
    }
}

impl std::ops::Deref for ClientWizEquipmentBehavior {
    type Target = ClientEquipmentBehavior;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl MemoryObject for ClientWizEquipmentBehavior {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::reader::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorInstance for ClientWizEquipmentBehavior {}
