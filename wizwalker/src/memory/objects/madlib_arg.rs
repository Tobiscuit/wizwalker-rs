use crate::memory::memory_object::DynamicMemoryObject;

pub struct MadlibArg {
    pub inner: DynamicMemoryObject,
}

impl MadlibArg {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }
}
