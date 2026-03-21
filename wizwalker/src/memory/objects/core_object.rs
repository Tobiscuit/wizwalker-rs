use crate::memory::reader::MemoryReaderExt;
use crate::errors::{Result, WizWalkerError};
use crate::types::{Orient, XYZ};
use crate::memory::MemoryReader;
use crate::memory::objects::core_template::CoreTemplate;
use std::sync::Arc;

pub struct CoreObject<R: MemoryReader + 'static> {
    pub reader: Arc<R>,
    pub base_address: u64,
}

impl<R: MemoryReader + 'static> CoreObject<R> {
    pub fn new(reader: Arc<R>, base_address: u64) -> Self {
        Self { reader, base_address }
    }

    pub fn read_base_address(&self) -> Result<u64> {
        Ok(self.base_address)
    }

    pub fn read_value_from_offset<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        self.reader.read_typed::<T>((self.base_address + offset) as usize)
    }

    pub fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        self.reader.write_typed::<T>((self.base_address + offset) as usize, value)
    }

    pub fn global_id_full(&self) -> Result<u64> {
        self.read_value_from_offset::<u64>(72)
    }

    pub fn write_global_id_full(&self, val: u64) -> Result<()> {
        self.write_value_to_offset::<u64>(72, &val)
    }

    pub fn perm_id(&self) -> Result<u64> {
        self.read_value_from_offset::<u64>(80)
    }

    pub fn write_perm_id(&self, val: u64) -> Result<()> {
        self.write_value_to_offset::<u64>(80, &val)
    }

    pub fn object_template(&self) -> Result<Option<CoreTemplate<R>>> {
        let addr = self.read_value_from_offset::<i64>(88)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(CoreTemplate::new(Arc::clone(&self.reader), addr as u64)))
    }

    pub fn template_id_full(&self) -> Result<u64> {
        self.read_value_from_offset::<u64>(96)
    }

    pub fn write_template_id_full(&self, val: u64) -> Result<()> {
        self.write_value_to_offset::<u64>(96, &val)
    }

    fn read_string_from_offset(&self, offset: u64) -> Result<String> {
        let chunk_size = 128;
        let mut string_bytes = Vec::new();
        let mut current_offset = offset;

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

    pub fn debug_name(&self) -> Result<String> {
        self.read_string_from_offset(104)
    }

    pub fn display_key(&self) -> Result<String> {
        self.read_string_from_offset(136)
    }

    pub fn location(&self) -> Result<XYZ> {
        self.read_value_from_offset::<XYZ>(168)
    }

    pub fn write_location(&self, val: &XYZ) -> Result<()> {
        self.write_value_to_offset::<XYZ>(168, val)
    }

    pub fn orientation(&self) -> Result<Orient> {
        self.read_value_from_offset::<Orient>(180)
    }

    pub fn write_orientation(&self, val: &Orient) -> Result<()> {
        self.write_value_to_offset::<Orient>(180, val)
    }

    pub fn speed_multiplier(&self) -> Result<i16> {
        self.read_value_from_offset::<i16>(192)
    }

    pub fn write_speed_multiplier(&self, val: i16) -> Result<()> {
        self.write_value_to_offset::<i16>(192, &val)
    }

    pub fn mobile_id(&self) -> Result<u16> {
        self.read_value_from_offset::<u16>(194)
    }

    pub fn write_mobile_id(&self, val: u16) -> Result<()> {
        self.write_value_to_offset::<u16>(194, &val)
    }

    pub fn scale(&self) -> Result<f32> {
        self.read_value_from_offset::<f32>(196)
    }

    pub fn write_scale(&self, val: f32) -> Result<()> {
        self.write_value_to_offset::<f32>(196, &val)
    }

    pub fn parent(&self) -> Result<Option<CoreObject<R>>> {
        let addr = self.read_value_from_offset::<i64>(208)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(CoreObject::new(Arc::clone(&self.reader), addr as u64)))
    }

    pub fn inactive_behaviors(&self) -> Result<Vec<u64>> {
        // Stub for read_shared_vector(224)
        Ok(vec![])
    }

    pub fn zone_tag_id(&self) -> Result<u32> {
        self.read_value_from_offset::<u32>(344)
    }

    pub fn write_zone_tag_id(&self, val: u32) -> Result<()> {
        self.write_value_to_offset::<u32>(344, &val)
    }

    pub fn search_behavior_by_name(&self, _name: &str) -> Result<Option<u64>> {
        Ok(None)
    }
}
