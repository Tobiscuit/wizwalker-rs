use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

pub struct ClientTagList {
    pub inner: DynamicMemoryObject,
}

impl ClientTagList {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn client_tags(&self) -> Result<Vec<String>> {
        let addrs = self.inner.read_linked_list::<u64, _>(0x48, |_| Ok(0))?;
        let mut res = Vec::new();
        // Since the read_linked_list returns empty currently
        for addr in addrs {
            // let string = self.inner.read_string(addr)?;
            // res.push(string);
        }
        Ok(res)
    }
}
