use crate::memory::reader::MemoryReaderExt;
use crate::errors::{Result, WizWalkerError};
use crate::memory::MemoryReader;
use std::sync::Arc;

pub struct ClientZone<R: MemoryReader + 'static> {
    pub reader: Arc<R>,
    pub base_address: u64,
}

impl<R: MemoryReader + 'static> ClientZone<R> {
    pub fn new(reader: Arc<R>, base_address: u64) -> Self {
        Self { reader, base_address }
    }

    pub fn read_base_address(&self) -> Result<u64> {
        Ok(self.base_address)
    }

    fn read_value_from_offset<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        self.reader.read_typed::<T>((self.base_address + offset) as usize)
    }

    fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        self.reader.write_typed::<T>((self.base_address + offset) as usize, value)
    }

    pub fn zone_id(&self) -> Result<i64> {
        self.read_value_from_offset::<i64>(72)
    }

    pub fn write_zone_id(&self, zone_id: i64) -> Result<()> {
        self.write_value_to_offset::<i64>(72, &zone_id)
    }

    pub fn zone_name(&self) -> Result<String> {
        let chunk_size = 128;
        let mut string_bytes = Vec::new();
        let mut current_offset = 88;

        'outer: loop {
            let chunk = self.reader.read_bytes((self.base_address + current_offset) as usize, chunk_size)?;
            for &byte in chunk.iter() {
                if byte == 0 {
                    break 'outer;
                }
                string_bytes.push(byte);
            }
            current_offset += chunk_size as u64;

            if string_bytes.len() > 1024 {
                return Err(WizWalkerError::Other("String too long".into()));
            }
        }

        Ok(String::from_utf8(string_bytes).map_err(|_| WizWalkerError::Other("Invalid UTF-8".into()))?)
    }

    pub fn write_zone_name(&self, _zone_name: &str) -> Result<()> {
        Err(WizWalkerError::Other("write_zone_name not fully implemented".into()))
    }
}
