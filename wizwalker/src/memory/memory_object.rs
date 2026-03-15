use crate::errors::{Result, WizWalkerError};
use std::sync::Arc;

/// Trait for reading/writing game process memory.
/// Implemented by the concrete MemoryReader in memory/reader.rs
pub trait MemoryReader: Send + Sync {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>>;
    fn write_bytes(&self, address: usize, data: &[u8]) -> Result<()>;
    fn read_typed<T: Copy>(&self, address: usize) -> Result<T>;
    fn write_typed<T: Copy>(&self, address: usize, value: &T) -> Result<()>;
    fn pattern_scan(&self, pattern: &[u8], module: Option<&str>, first_only: bool) -> Result<Vec<usize>>;
    fn allocate(&self, size: usize) -> Result<usize>;
    fn free(&self, address: usize) -> Result<()>;
}

/// Base trait for all game memory objects.
/// Each concrete memory object wraps a MemoryReader + base_address.
pub trait MemoryObject {
    fn reader(&self) -> Arc<dyn MemoryReader>;
    fn read_base_address(&self) -> Result<u64>;

    fn read_value_from_offset<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        let base_address = self.read_base_address()?;
        self.reader().read_typed::<T>((base_address + offset) as usize)
    }

    fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        let base_address = self.read_base_address()?;
        self.reader().write_typed::<T>((base_address + offset) as usize, value)
    }

    fn read_linked_list<T, F>(&self, head_offset: u64, _parser: F) -> Result<Vec<T>>
    where
        F: Fn(u64) -> Result<T>,
    {
        // Implemented by objects that read linked list game data
        let _ = head_offset;
        Ok(vec![])
    }
}

/// Concrete generic MemoryObject. Used when you just need base+reader without a specific type.
pub struct DynamicMemoryObject {
    reader: Arc<dyn MemoryReader>,
    base_address: u64,
}

impl DynamicMemoryObject {
    pub fn new(reader: Arc<dyn MemoryReader>, base_address: u64) -> Result<Self> {
        if base_address == 0 {
            return Err(WizWalkerError::Other(
                "Dynamic object passed 0 base address.".into(),
            ));
        }
        Ok(Self { reader, base_address })
    }
}

impl MemoryObject for DynamicMemoryObject {
    fn reader(&self) -> Arc<dyn MemoryReader> {
        Arc::clone(&self.reader)
    }

    fn read_base_address(&self) -> Result<u64> {
        Ok(self.base_address)
    }
}
