use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObjectExt};
use super::madlib_arg::MadlibArg;

pub struct MadlibBlock {
    pub inner: DynamicMemoryObject,
}

impl MadlibBlock {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn entries(&self) -> Result<Vec<MadlibArg>> {
        let addrs = self.inner.read_linked_list(0x48)?;
        let result = Vec::new();
        for _addr in addrs {}
        Ok(result)
    }
}
