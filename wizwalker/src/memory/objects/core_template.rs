use crate::errors::Result;
use crate::memory::memory_object::MemoryReader;
use std::sync::Arc;

pub struct CoreTemplate<R: MemoryReader + 'static> {
    pub reader: Arc<R>,
    pub base_address: u64,
}

impl<R: MemoryReader + 'static> CoreTemplate<R> {
    pub fn new(reader: Arc<R>, base_address: u64) -> Self {
        Self { reader, base_address }
    }

    pub fn read_base_address(&self) -> Result<u64> {
        Ok(self.base_address)
    }

    pub async fn behaviors(&self) -> Result<Vec<u64>> {
        // read_dynamic_vector(72) stub
        // Since we don't have DynamicBehaviorTemplate, returning addresses for now
        Ok(vec![])
    }
}
