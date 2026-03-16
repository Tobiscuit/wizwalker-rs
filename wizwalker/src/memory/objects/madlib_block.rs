use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::madlib_arg::MadlibArg;

pub struct MadlibBlock {
    pub inner: DynamicMemoryObject,
}

impl MadlibBlock {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn entries(&self) -> Result<Vec<MadlibArg>> {
        let addrs = self.inner.read_linked_list::<u64, _>(0x48, |_| Ok(0))?;
        let mut result = Vec::new();
        for _addr in addrs {}
        Ok(result)
    }
}
