use crate::errors::{Result, WizWalkerError};
use crate::memory::reader::{MemoryReader, MemoryReaderExt};
use std::sync::Arc;

pub trait MemoryObject {
    fn reader(&self) -> Arc<dyn MemoryReader>;
    fn read_base_address(&self) -> Result<u64>;

    fn read_value_from_offset<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        let base_address = self.read_base_address()?;
        self.reader().read_typed::<T>(base_address + offset)
    }

    fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        let base_address = self.read_base_address()?;
        self.reader().write_typed::<T>(base_address + offset, value)
    }

    fn pattern_scan_offset(&self, pattern: &[u8], instruction_length: u64, static_backup: Option<u64>) -> Result<u64> {
        let res = self.reader().pattern_scan(pattern, Some("WizardGraphicalClient.exe"), false);
        match res {
            Ok(addrs) => {
                let addr = addrs[0];
                let val: u32 = self.reader().read_typed(addr + instruction_length)?;
                Ok(val as u64)
            }
            Err(e) => {
                if let Some(backup) = static_backup {
                    Ok(backup)
                } else {
                    Err(e)
                }
            }
        }
    }
}

pub struct DynamicMemoryObject {
    reader: Arc<dyn MemoryReader>,
    base_address: u64,
}

impl DynamicMemoryObject {
    pub fn new(reader: Arc<dyn MemoryReader>, base_address: u64) -> Result<Self> {
        if base_address == 0 {
            return Err(WizWalkerError::Other("Dynamic object passed 0 base address.".into()));
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

use crate::types::{Color, Orient, XYZ};

pub trait MemoryObjectExt: MemoryObject {
    fn read_null_terminated_string(&self, address: u64, max_size: usize) -> Result<String> {
        let bytes = self.reader().read_bytes(address, max_size)?;
        if let Some(end) = bytes.iter().position(|&b| b == 0) {
            if end == 0 {
                return Ok(String::new());
            }
            Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
        } else {
            Err(WizWalkerError::Other("Null terminator not found".into()))
        }
    }

    fn read_string(&self, address: u64) -> Result<String> {
        let mut capacity: usize = self.reader().read_typed(address + 0x18)?;
        if capacity < 16 {
            capacity = 15;
        }

        if capacity < 16 {
            let bytes = self.reader().read_bytes(address, 16)?;
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
        } else {
            let ptr: u64 = self.reader().read_typed(address)?;
            let len: usize = self.reader().read_typed(address + 0x10)?;
            let bytes = self.reader().read_bytes(ptr, len)?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }
    }

    fn read_vector<T: Copy + Default>(&self, offset: u64, size: usize) -> Result<Vec<T>> {
        let mut res = Vec::new();
        let base_address = self.read_base_address()?;
        let type_size = std::mem::size_of::<T>() as u64;
        let mut addr = base_address + offset;

        for _ in 0..size {
            let val = self.reader().read_typed::<T>(addr)?;
            res.push(val);
            addr += type_size;
        }
        Ok(res)
    }

    fn write_vector<T: Copy>(&self, offset: u64, values: &[T]) -> Result<()> {
        let base_address = self.read_base_address()?;
        let type_size = std::mem::size_of::<T>() as u64;
        let mut addr = base_address + offset;

        for val in values {
            self.reader().write_typed(addr, val)?;
            addr += type_size;
        }
        Ok(())
    }

    fn read_color(&self, offset: u64) -> Result<Color> {
        let vec: Vec<u8> = self.read_vector(offset, 3)?;
        Ok(Color::new(vec[0], vec[1], vec[2]))
    }

    fn write_color(&self, offset: u64, color: Color) -> Result<()> {
        self.write_vector(offset, &[color.r, color.g, color.b])
    }

    fn read_xyz(&self, offset: u64) -> Result<XYZ> {
        let vec: Vec<f32> = self.read_vector(offset, 3)?;
        Ok(XYZ::new(vec[0], vec[1], vec[2]))
    }

    fn write_xyz(&self, offset: u64, xyz: XYZ) -> Result<()> {
        self.write_vector(offset, &[xyz.x, xyz.y, xyz.z])
    }

    fn read_orient(&self, offset: u64) -> Result<Orient> {
        let vec: Vec<f32> = self.read_vector(offset, 3)?;
        Ok(Orient::new(vec[0], vec[1], vec[2]))
    }

    fn write_orient(&self, offset: u64, orient: Orient) -> Result<()> {
        self.write_vector(offset, &[orient.pitch, orient.roll, orient.yaw])
    }

    fn read_enum<T: From<i32>>(&self, offset: u64) -> Result<T> {
        let val: i32 = self.read_value_from_offset(offset)?;
        Ok(T::from(val))
    }

    fn write_enum<T: Into<i32> + Copy>(&self, offset: u64, enum_val: T) -> Result<()> {
        let val: i32 = enum_val.into();
        self.write_value_to_offset(offset, &val)
    }

    fn read_dynamic_vector<T: Copy + Default>(&self, offset: u64) -> Result<Vec<T>> {
        let start_address: u64 = self.read_value_from_offset(offset)?;
        let end_address: u64 = self.read_value_from_offset(offset + 8)?;

        let size_per_type = std::mem::size_of::<T>() as u64;
        let count = end_address.saturating_sub(start_address) / size_per_type;

        if count == 0 {
            return Ok(Vec::new());
        }

        let mut current_address = start_address;
        let mut vals = Vec::new();
        for _ in 0..count {
            let val = self.reader().read_typed::<T>(current_address)?;
            vals.push(val);
            current_address += size_per_type;
        }
        Ok(vals)
    }

    fn read_shared_vector(&self, offset: u64, max_size: usize) -> Result<Vec<u64>> {
        let start_address: u64 = self.read_value_from_offset(offset)?;
        let end_address: u64 = self.read_value_from_offset(offset + 8)?;

        let diff = end_address.saturating_sub(start_address);
        let element_number = diff / 16;

        if diff == 0 || element_number == 0 {
            return Ok(Vec::new());
        }
        if element_number > max_size as u64 {
            return Err(WizWalkerError::Other(format!("Size was {} and max was {}", element_number, max_size)));
        }

        let mut pointers = Vec::new();
        let mut current_addr = start_address;
        for _ in 0..element_number {
            let val: u64 = self.reader().read_typed(current_addr)?;
            pointers.push(val);
            current_addr += 16;
        }

        Ok(pointers)
    }

    fn read_inlined_vector<T, F>(&self, offset: u64, object_size: u64, mapper: F) -> Result<Vec<T>>
    where
        F: Fn(Arc<dyn MemoryReader>, u64) -> Result<T>,
    {
        let start: u64 = self.read_value_from_offset(offset)?;
        let end: u64 = self.read_value_from_offset(offset + 8)?;

        let total_size = end.saturating_sub(start) / object_size;
        let mut current_addr = start;
        let mut res = Vec::new();
        for _ in 0..total_size {
            res.push(mapper(self.reader(), current_addr)?);
            current_addr += object_size;
        }
        Ok(res)
    }
}

impl<T: MemoryObject> MemoryObjectExt for T {}

pub trait ComplexStructures: MemoryObject {
    fn read_linked_list(&self, offset: u64) -> Result<Vec<u64>> {
        let list_addr: u64 = self.read_value_from_offset(offset)?;
        let list_size: i32 = self.read_value_from_offset(offset + 8)?;

        if list_size < 1 {
            return Ok(Vec::new());
        }

        let mut addrs = Vec::new();
        let mut list_node: u64 = self.reader().read_typed(list_addr)?;

        addrs.push(list_node + 16);
        for _ in 0..(list_size - 1) {
            list_node = self.reader().read_typed(list_node)?;
            addrs.push(list_node + 16);
        }

        Ok(addrs)
    }

    fn read_shared_linked_list(&self, offset: u64) -> Result<Vec<u64>> {
        let list_addr: u64 = self.read_value_from_offset(offset)?;
        let list_size: i32 = self.read_value_from_offset(offset + 8)?;

        if list_size < 1 {
            return Ok(Vec::new());
        }

        let mut addrs = Vec::new();
        let mut next_node_addr: u64 = self.reader().read_typed(list_addr)?;

        for _ in 0..list_size {
            let addr: u64 = self.reader().read_typed(next_node_addr + 16)?;
            addrs.push(addr);
            next_node_addr = self.reader().read_typed(next_node_addr)?;
        }

        Ok(addrs)
    }

    fn read_std_map<K, V, F>(&self, offset: u64, mapper: F) -> Result<Vec<(K, V)>>
    where
        K: From<u64>,
        F: Fn(Arc<dyn MemoryReader>, u64) -> Result<V> + Copy,
    {
        let root: u64 = self.read_value_from_offset(offset)?;
        let first_node: u64 = self.reader().read_typed(root + 0x8)?;

        if first_node == root {
            return Ok(Vec::new());
        }

        let mut res = Vec::new();
        let mut stack = vec![first_node];

        while let Some(node) = stack.pop() {
            let is_leaf: u8 = self.reader().read_typed(node + 0x19)?;
            if is_leaf == 0 {
                let key: u64 = self.reader().read_typed(node + 0x20)?;
                let data: u64 = self.reader().read_typed(node + 0x28)?;
                res.push((K::from(key), mapper(self.reader(), data)?));

                let left_node: u64 = self.reader().read_typed(node)?;
                if left_node != 0 {
                    stack.push(left_node);
                }

                let right_node: u64 = self.reader().read_typed(node + 0x10)?;
                if right_node != 0 {
                    stack.push(right_node);
                }
            }
        }

        Ok(res)
    }

    fn read_hashset_basic<T: Copy + Default + std::hash::Hash + Eq>(&self, offset: u64) -> Result<std::collections::HashSet<T>> {
        let mut result = std::collections::HashSet::new();
        let root_addr: u64 = self.read_value_from_offset(offset)?;
        let root = DynamicMemoryObject::new(self.reader(), root_addr)?;

        let initial_node_addr: u64 = root.read_value_from_offset(8)?;
        let mut stack = vec![DynamicMemoryObject::new(self.reader(), initial_node_addr)?];

        while let Some(node) = stack.pop() {
            let is_leaf: u8 = node.read_value_from_offset(0x19)?;
            if is_leaf != 0 {
                continue;
            }

            let n1: u64 = node.read_value_from_offset(0x00)?;
            stack.push(DynamicMemoryObject::new(self.reader(), n1)?);

            let n2: u64 = node.read_value_from_offset(0x10)?;
            stack.push(DynamicMemoryObject::new(self.reader(), n2)?);

            let val: T = node.read_value_from_offset(0x1C)?;
            result.insert(val);
        }
        Ok(result)
    }
}

impl<T: MemoryObject> ComplexStructures for T {}

pub trait PropertyClass: MemoryObjectExt {
    fn maybe_read_type_name(&self) -> String {
        self.read_type_name().unwrap_or_default()
    }

    fn read_type_name(&self) -> Result<String> {
        let vtable: u64 = self.read_value_from_offset(0)?;
        let get_class_name: u64 = self.reader().read_typed(vtable)?;

        let maybe_jmp = self.reader().read_bytes(get_class_name, 5)?;
        let mut actual_get_class_name = get_class_name;
        if maybe_jmp[0] == 233 { // 0xE9 jmp
            let offset: i32 = self.reader().read_typed(get_class_name + 1)?;
            actual_get_class_name = (get_class_name as i64 + offset as i64 + 5) as u64;
        }

        let lea_instruction = actual_get_class_name + 63;
        let lea_target = actual_get_class_name + 66;
        let rip_offset: i32 = self.reader().read_typed(lea_target)?;

        let type_name_addr = (lea_instruction as i64 + rip_offset as i64 + 7) as u64;
        self.read_null_terminated_string(type_name_addr, 60)
    }
}

impl PropertyClass for DynamicMemoryObject {}

pub trait Pointers: MemoryObject {
    fn read_pointer<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        let ptr_addr: u64 = self.read_value_from_offset(offset)?;
        self.reader().read_typed::<T>(ptr_addr)
    }
}

impl<M: MemoryObject> Pointers for M {}
