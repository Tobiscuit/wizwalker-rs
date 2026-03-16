use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObject, MemoryObjectExt};
use super::behavior_template::DynamicBehaviorTemplate;

pub trait BehaviorInstance: MemoryObject {
    fn behavior_template_name_id(&self) -> Result<u32> {
        self.read_value_from_offset(104)
    }

    fn write_behavior_template_name_id(&self, behavior_template_name_id: u32) -> Result<()> {
        self.write_value_to_offset(104, &behavior_template_name_id)
    }

    fn behavior_template(&self) -> Result<Option<DynamicBehaviorTemplate>> {
        let addr: u64 = self.read_value_from_offset(0x58)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(DynamicBehaviorTemplate::new(self.reader(), addr)?))
    }

    fn behavior_name(&self) -> Result<Option<String>> {
        if let Some(template) = self.behavior_template()? {
            use super::behavior_template::BehaviorTemplate;
            let name = template.behavior_name()?;
            Ok(Some(name))
        } else {
            Ok(None)
        }
    }
}

pub struct DynamicBehaviorInstance {
    inner: DynamicMemoryObject,
}

impl DynamicBehaviorInstance {
    pub fn new(reader: std::sync::Arc<dyn crate::memory::memory_object::MemoryReader>, base_address: u64) -> Result<Self> {
        Ok(Self {
            inner: DynamicMemoryObject::new(reader, base_address)?,
        })
    }
}

impl MemoryObject for DynamicBehaviorInstance {
    fn reader(&self) -> std::sync::Arc<dyn crate::memory::memory_object::MemoryReader> {
        self.inner.reader()
    }

    fn read_base_address(&self) -> Result<u64> {
        self.inner.read_base_address()
    }
}

impl BehaviorInstance for DynamicBehaviorInstance {}
