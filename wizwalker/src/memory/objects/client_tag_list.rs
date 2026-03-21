use crate::errors::Result;
use crate::memory::memory_object::{DynamicMemoryObject, MemoryObjectExt};

pub struct ClientTagList {
    pub inner: DynamicMemoryObject,
}

impl ClientTagList {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn client_tags(&self) -> Result<Vec<String>> {
        let addrs = self.inner.read_linked_list(0x48)?;
        let res = Vec::new();
        // Since the read_linked_list returns empty currently
        for _addr in addrs {
            // let string = self.inner.read_string(addr)?;
            // res.push(string);
        }
        Ok(res)
    }
}
