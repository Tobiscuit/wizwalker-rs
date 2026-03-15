use crate::errors::{Result, WizWalkerError};
use std::sync::Arc;

/// Trait for reading/writing game process memory.
/// Implemented by the concrete MemoryReader in memory/reader.rs
pub trait MemoryReader: Send + Sync {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>>;
    fn write_bytes(&self, address: usize, data: &[u8]) -> Result<()>;
    fn read_typed<T: Copy>(&self, address: usize) -> Result<T> where Self: Sized;
    fn write_typed<T: Copy>(&self, address: usize, value: &T) -> Result<()> where Self: Sized;
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
        let size = std::mem::size_of::<T>();
        let bytes = self.reader().read_bytes((base_address + offset) as usize, size)?;
        unsafe { Ok(std::ptr::read_unaligned(bytes.as_ptr() as *const T)) }
    }

    fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        let base_address = self.read_base_address()?;
        let size = std::mem::size_of::<T>();
        let mut bytes = vec![0u8; size];
        unsafe { std::ptr::write_unaligned(bytes.as_mut_ptr() as *mut T, *value) };
        self.reader().write_bytes((base_address + offset) as usize, &bytes)
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


use std::collections::HashSet;
use std::hash::Hash;
use crate::types::{Color, XYZ, Orient};

const MAX_STRING: u32 = 4096;

pub trait MemoryObjectExt: MemoryObject {
    fn read_string_from_offset(&self, offset: u64) -> Result<String> {
        let base_address = self.read_base_address()?;
        let address = base_address + offset;

        let string_len: u32 = self.read_value_from_offset(offset + 16)?;

        if string_len == 0 || string_len > MAX_STRING {
            return Ok(String::new());
        }

        let string_address = if string_len >= 16 {
            self.read_value_from_offset::<u64>(offset)?
        } else {
            address
        };

        let bytes = self.reader().read_bytes(string_address as usize, string_len as usize)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    fn write_string_to_offset(&self, offset: u64, string: &str) -> Result<()> {
        let base_address = self.read_base_address()?;
        let address = base_address + offset;

        let encoded = string.as_bytes();
        let string_len = encoded.len() as u32;
        let current_string_len: u32 = self.read_value_from_offset(offset + 16)?;

        if string_len >= 15 && string_len > current_string_len {
            let pointer_address = self.reader().allocate((string_len + 1) as usize)?;
            let mut data = encoded.to_vec();
            data.push(0);
            self.reader().write_bytes(pointer_address, &data)?;
            self.write_value_to_offset::<u64>(offset, &(pointer_address as u64))?;
            self.write_value_to_offset::<u32>(offset + 16, &string_len)?;
            self.write_value_to_offset::<u32>(offset + 20, &string_len)?;
        } else {
            let dest_addr = if current_string_len >= 16 {
                self.read_value_from_offset::<u64>(offset)?
            } else {
                address
            };
            let mut data = encoded.to_vec();
            data.push(0);
            self.reader().write_bytes(dest_addr as usize, &data)?;
            self.write_value_to_offset::<u32>(offset + 16, &string_len)?;
            if current_string_len < 16 {
                self.write_value_to_offset::<u32>(offset + 20, &15)?;
            }
        }

        Ok(())
    }

    fn read_dynamic_vector<T: Copy + Default>(&self, offset: u64) -> Result<Vec<T>> {
        let start_address: u64 = self.read_value_from_offset(offset)?;
        let end_address: u64 = self.read_value_from_offset(offset + 8)?;

        let size_per_type = std::mem::size_of::<T>();
        if start_address >= end_address {
            return Ok(Vec::new());
        }

        let bytes_len = end_address - start_address;
        let count = bytes_len / size_per_type as u64;

        if count == 0 {
            return Ok(Vec::new());
        }

        let bytes = self.reader().read_bytes(start_address as usize, bytes_len as usize)?;

        let mut vec = Vec::with_capacity(count as usize);
        unsafe {
            let ptr = bytes.as_ptr() as *const T;
            for i in 0..count {
                vec.push(std::ptr::read_unaligned(ptr.add(i as usize)));
            }
        }
        Ok(vec)
    }

    fn read_shared_linked_list(&self, offset: u64) -> Result<Vec<u64>> {
        let list_addr: u64 = self.read_value_from_offset(offset)?;
        if list_addr == 0 {
            return Ok(Vec::new());
        }
        let bytes = self.reader().read_bytes(list_addr as usize, 8)?;
        let mut next_node_addr = u64::from_ne_bytes(bytes.try_into().unwrap());

        let list_size: u32 = self.read_value_from_offset(offset + 8)?;

        let mut addrs = Vec::new();
        for _ in 0..list_size {
            let addr_bytes = self.reader().read_bytes((next_node_addr + 16) as usize, 8)?;
            let addr = u64::from_ne_bytes(addr_bytes.try_into().unwrap());
            addrs.push(addr);

            let next_bytes = self.reader().read_bytes(next_node_addr as usize, 8)?;
            next_node_addr = u64::from_ne_bytes(next_bytes.try_into().unwrap());
        }
        Ok(addrs)
    }

    fn read_hashset_basic<T: Copy + Eq + Hash + Default>(&self, offset: u64) -> Result<HashSet<T>> {
        let mut result = HashSet::new();
        let root_addr: u64 = self.read_value_from_offset(offset)?;
        let child_bytes = self.reader().read_bytes((root_addr + 8) as usize, 8)?;
        let root_child_addr = u64::from_ne_bytes(child_bytes.try_into().unwrap());

        let mut stack = vec![root_child_addr];
        let size_per_type = std::mem::size_of::<T>();

        while let Some(node_addr) = stack.pop() {
            let is_leaf_bytes = self.reader().read_bytes((node_addr + 0x19) as usize, 1)?;
            let is_leaf = is_leaf_bytes[0] != 0;
            if is_leaf {
                continue;
            }

            let left_bytes = self.reader().read_bytes(node_addr as usize, 8)?;
            let left = u64::from_ne_bytes(left_bytes.try_into().unwrap());

            let right_bytes = self.reader().read_bytes((node_addr + 0x10) as usize, 8)?;
            let right = u64::from_ne_bytes(right_bytes.try_into().unwrap());

            stack.push(left);
            stack.push(right);

            let val_bytes = self.reader().read_bytes((node_addr + 0x1C) as usize, size_per_type)?;
            unsafe {
                let ptr = val_bytes.as_ptr() as *const T;
                result.insert(std::ptr::read_unaligned(ptr));
            }
        }
        Ok(result)
    }

    fn read_enum<T: TryFrom<i32>>(&self, offset: u64) -> Result<T> {
        let val: i32 = self.read_value_from_offset(offset)?;
        T::try_from(val).map_err(|_| WizWalkerError::Other(format!("Failed to read enum with value {}", val)))
    }

    fn write_enum<T>(&self, offset: u64, value: T) -> Result<()>
    where T: Into<i32> {
        self.write_value_to_offset(offset, &value.into())
    }

    fn read_color(&self, offset: u64) -> Result<Color> {
        let bytes = self.reader().read_bytes((self.read_base_address()? + offset) as usize, 3)?;
        Ok(Color { r: bytes[0], g: bytes[1], b: bytes[2], a: 255 })
    }

    fn write_color(&self, offset: u64, val: &Color) -> Result<()> {
        let bytes = vec![val.r, val.g, val.b];
        self.reader().write_bytes((self.read_base_address()? + offset) as usize, &bytes)?;
        Ok(())
    }
}

impl<T: MemoryObject + ?Sized> MemoryObjectExt for T {}
