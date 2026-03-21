use crate::errors::{Result, WizWalkerError};
use crate::memory::reader::MemoryReader;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use crate::types::{Color, XYZ, Orient};

/// Base trait for all game memory objects.
///
/// Each concrete memory object wraps an `Arc<dyn MemoryReader>` and a base address.
/// Game-specific traits (e.g., `GameStats`, `CombatParticipant`) extend this
/// to provide typed accessors at known offsets.
pub trait MemoryObject {
    /// Returns the shared memory reader for this object.
    fn reader(&self) -> Arc<dyn MemoryReader>;

    /// Returns the base address of this object in the target process.
    fn read_base_address(&self) -> Result<u64>;

    /// Read a raw value of type `T` at `self.base_address + offset`.
    fn read_value_from_offset<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        let base_address = self.read_base_address()?;
        let address = (base_address + offset) as usize;
        let size = std::mem::size_of::<T>();
        let bytes = self.reader().read_bytes(address, size)?;
        unsafe { Ok(std::ptr::read_unaligned(bytes.as_ptr() as *const T)) }
    }

    /// Write a raw value of type `T` at `self.base_address + offset`.
    fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        let base_address = self.read_base_address()?;
        let address = (base_address + offset) as usize;
        let size = std::mem::size_of::<T>();
        let mut bytes = vec![0u8; size];
        unsafe { std::ptr::write_unaligned(bytes.as_mut_ptr() as *mut T, *value) };
        self.reader().write_bytes(address, &bytes)
    }
}

/// Concrete generic memory object used when you need base + reader without a specific type.
#[derive(Clone)]
pub struct DynamicMemoryObject {
    reader: Arc<dyn MemoryReader>,
    base_address: u64,
}

impl DynamicMemoryObject {
    pub fn new(reader: Arc<dyn MemoryReader>, base_address: u64) -> Result<Self> {
        if base_address == 0 {
            return Err(WizWalkerError::AddressOutOfRange(0));
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

const MAX_STRING_LENGTH: u32 = 4096;

/// Extension trait providing complex memory-reading helpers for all `MemoryObject` types.
///
/// Handles game-engine-specific structures like SSO strings, dynamic vectors,
/// shared linked lists, hash sets, enums, and color values.
pub trait MemoryObjectExt: MemoryObject {
    /// Read a C++ SSO (Small String Optimization) string at the given offset.
    ///
    /// Layout: if `string_len >= 16` the first 8 bytes are a pointer to heap data,
    /// otherwise the string is stored inline. Length at `offset + 16`, capacity at `offset + 20`.
    fn read_string_from_offset(&self, offset: u64) -> Result<String> {
        let base_address = self.read_base_address()?;
        let address = base_address + offset;

        let string_len: u32 = self.read_value_from_offset(offset + 16)?;

        if string_len == 0 || string_len > MAX_STRING_LENGTH {
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

    /// Read a C++ `std::wstring` (UTF-16) from the given offset.
    ///
    /// Python equivalent: `MemoryObject.read_wide_string_from_offset(offset)`
    /// Layout: length (in wchar_t units) at `offset + 16`.
    /// If `string_len >= 8` wide chars, the first 8 bytes are a pointer to heap data;
    /// otherwise the string is stored inline.
    fn read_wide_string_from_offset(&self, offset: u64) -> Result<String> {
        let base_address = self.read_base_address()?;
        let address = base_address + offset;

        let string_len: i32 = self.read_value_from_offset(offset + 16)?;

        if string_len <= 0 {
            return Ok(String::new());
        }

        // Wide chars are 2 bytes each
        let byte_len = (string_len as usize) * 2;

        // Wide strings >= 8 wchars (16 bytes) are stored as pointers
        let string_address = if byte_len >= 16 {
            self.read_value_from_offset::<u64>(offset)?
        } else {
            address
        };

        let bytes = self.reader().read_bytes(string_address as usize, byte_len)?;

        // Decode UTF-16LE
        let u16_vec: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(String::from_utf16_lossy(&u16_vec))
    }

    /// Write a C++ SSO string at the given offset.
    fn write_string_to_offset(&self, offset: u64, value: &str) -> Result<()> {
        let base_address = self.read_base_address()?;
        let address = base_address + offset;

        let encoded = value.as_bytes();
        let new_len = encoded.len() as u32;
        let current_len: u32 = self.read_value_from_offset(offset + 16)?;

        if new_len >= 15 && new_len > current_len {
            // Allocate new heap buffer for longer strings
            let buffer_address = self.reader().allocate((new_len + 1) as usize)?;
            let mut data = encoded.to_vec();
            data.push(0); // null terminator
            self.reader().write_bytes(buffer_address, &data)?;
            self.write_value_to_offset::<u64>(offset, &(buffer_address as u64))?;
            self.write_value_to_offset::<u32>(offset + 16, &new_len)?;
            self.write_value_to_offset::<u32>(offset + 20, &new_len)?;
        } else {
            let dest_addr = if current_len >= 16 {
                self.read_value_from_offset::<u64>(offset)?
            } else {
                address
            };
            let mut data = encoded.to_vec();
            data.push(0);
            self.reader().write_bytes(dest_addr as usize, &data)?;
            self.write_value_to_offset::<u32>(offset + 16, &new_len)?;
            if current_len < 16 {
                self.write_value_to_offset::<u32>(offset + 20, &15)?;
            }
        }

        Ok(())
    }

    /// Read a `std::vector<T>` (start/end pointer pair) from the given offset.
    fn read_dynamic_vector<T: Copy + Default>(&self, offset: u64) -> Result<Vec<T>> {
        let start_address: u64 = self.read_value_from_offset(offset)?;
        let end_address: u64 = self.read_value_from_offset(offset + 8)?;

        let element_size = std::mem::size_of::<T>();
        if start_address >= end_address {
            return Ok(Vec::new());
        }

        let byte_count = (end_address - start_address) as usize;
        let element_count = byte_count / element_size;

        if element_count == 0 {
            return Ok(Vec::new());
        }

        let bytes = self.reader().read_bytes(start_address as usize, byte_count)?;

        let mut elements = Vec::with_capacity(element_count);
        unsafe {
            let ptr = bytes.as_ptr() as *const T;
            for i in 0..element_count {
                elements.push(std::ptr::read_unaligned(ptr.add(i)));
            }
        }
        Ok(elements)
    }

    /// Read a shared pointer linked list. Returns the data addresses from each node.
    fn read_shared_linked_list(&self, offset: u64) -> Result<Vec<u64>> {
        let list_addr: u64 = self.read_value_from_offset(offset)?;
        if list_addr == 0 {
            return Ok(Vec::new());
        }

        let bytes = self.reader().read_bytes(list_addr as usize, 8)?;
        let mut next_node_addr = u64::from_ne_bytes(bytes.try_into().unwrap());

        let list_size: u32 = self.read_value_from_offset(offset + 8)?;

        let mut addresses = Vec::with_capacity(list_size as usize);
        for _ in 0..list_size {
            let addr_bytes = self.reader().read_bytes((next_node_addr + 16) as usize, 8)?;
            let data_addr = u64::from_ne_bytes(addr_bytes.try_into().unwrap());
            addresses.push(data_addr);

            let next_bytes = self.reader().read_bytes(next_node_addr as usize, 8)?;
            next_node_addr = u64::from_ne_bytes(next_bytes.try_into().unwrap());
        }
        Ok(addresses)
    }

    /// Alias for `read_shared_linked_list` — used by some object files.
    fn read_linked_list(&self, offset: u64) -> Result<Vec<u64>> {
        self.read_shared_linked_list(offset)
    }

    /// Read a `std::set<T>` (red-black tree) from the given offset.
    fn read_hashset_basic<T: Copy + Eq + Hash + Default>(&self, offset: u64) -> Result<HashSet<T>> {
        let mut result = HashSet::new();
        let root_addr: u64 = self.read_value_from_offset(offset)?;
        let child_bytes = self.reader().read_bytes((root_addr + 8) as usize, 8)?;
        let root_child_addr = u64::from_ne_bytes(child_bytes.try_into().unwrap());

        let mut stack = vec![root_child_addr];
        let element_size = std::mem::size_of::<T>();

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

            let val_bytes = self.reader().read_bytes((node_addr + 0x1C) as usize, element_size)?;
            unsafe {
                let ptr = val_bytes.as_ptr() as *const T;
                result.insert(std::ptr::read_unaligned(ptr));
            }
        }
        Ok(result)
    }

    /// Read a game enum stored as an `i32` at the given offset.
    fn read_enum<T: TryFrom<i32>>(&self, offset: u64) -> Result<T> {
        let raw_value: i32 = self.read_value_from_offset(offset)?;
        T::try_from(raw_value).map_err(|_| {
            WizWalkerError::ReadingEnumFailed {
                enum_name: std::any::type_name::<T>().to_string(),
                value: raw_value.to_string(),
            }
        })
    }

    /// Write a game enum (converted to `i32`) at the given offset.
    fn write_enum<T: Into<i32>>(&self, offset: u64, value: T) -> Result<()> {
        self.write_value_to_offset(offset, &value.into())
    }

    /// Read an RGB color (3 bytes) at the given offset.
    fn read_color(&self, offset: u64) -> Result<Color> {
        let address = (self.read_base_address()? + offset) as usize;
        let bytes = self.reader().read_bytes(address, 3)?;
        Ok(Color { r: bytes[0], g: bytes[1], b: bytes[2], a: 255 })
    }

    /// Write an RGB color (3 bytes) at the given offset.
    fn write_color(&self, offset: u64, color: &Color) -> Result<()> {
        let address = (self.read_base_address()? + offset) as usize;
        let bytes = [color.r, color.g, color.b];
        self.reader().write_bytes(address, &bytes)
    }

    /// Read an XYZ position (3× f32, 12 bytes) at the given offset.
    fn read_xyz(&self, offset: u64) -> Result<XYZ> {
        Ok(XYZ {
            x: self.read_value_from_offset(offset)?,
            y: self.read_value_from_offset(offset + 4)?,
            z: self.read_value_from_offset(offset + 8)?,
        })
    }

    /// Write an XYZ position at the given offset.
    fn write_xyz(&self, offset: u64, position: &XYZ) -> Result<()> {
        self.write_value_to_offset(offset, &position.x)?;
        self.write_value_to_offset(offset + 4, &position.y)?;
        self.write_value_to_offset(offset + 8, &position.z)
    }

    /// Read an Orient (pitch, roll, yaw — 3× f32) at the given offset.
    fn read_orient(&self, offset: u64) -> Result<Orient> {
        Ok(Orient {
            pitch: self.read_value_from_offset(offset)?,
            roll: self.read_value_from_offset(offset + 4)?,
            yaw: self.read_value_from_offset(offset + 8)?,
        })
    }

    /// Write an Orient at the given offset.
    fn write_orient(&self, offset: u64, orient: &Orient) -> Result<()> {
        self.write_value_to_offset(offset, &orient.pitch)?;
        self.write_value_to_offset(offset + 4, &orient.roll)?;
        self.write_value_to_offset(offset + 8, &orient.yaw)
    }
}

/// Blanket implementation: every `MemoryObject` automatically gets `MemoryObjectExt`.
impl<T: MemoryObject + ?Sized> MemoryObjectExt for T {}
