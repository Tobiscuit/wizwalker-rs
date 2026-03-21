use crate::errors::Result;
use crate::types::{Orient, XYZ};
use crate::memory::MemoryReader;
use crate::memory::reader::MemoryReaderExt;
use std::sync::Arc;

pub struct ActorBody<R: MemoryReader + 'static> {
    pub reader: Arc<R>,
    pub base_address: u64,
}

impl<R: MemoryReader + 'static> ActorBody<R> {
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

    pub fn position(&self) -> Result<XYZ> {
        self.read_value_from_offset::<XYZ>(88)
    }

    pub fn write_position(&self, position: &XYZ) -> Result<()> {
        self.write_value_to_offset::<XYZ>(88, position)
    }

    pub fn orientation(&self) -> Result<Orient> {
        self.read_value_from_offset::<Orient>(100)
    }

    pub fn write_orientation(&self, orient: &Orient) -> Result<()> {
        self.write_value_to_offset::<Orient>(100, orient)
    }

    pub fn pitch(&self) -> Result<f32> {
        self.read_value_from_offset::<f32>(100)
    }

    pub fn write_pitch(&self, pitch: f32) -> Result<()> {
        self.write_value_to_offset::<f32>(100, &pitch)
    }

    pub fn roll(&self) -> Result<f32> {
        self.read_value_from_offset::<f32>(104)
    }

    pub fn write_roll(&self, roll: f32) -> Result<()> {
        self.write_value_to_offset::<f32>(104, &roll)
    }

    pub fn yaw(&self) -> Result<f32> {
        self.read_value_from_offset::<f32>(108)
    }

    pub fn write_yaw(&self, yaw: f32) -> Result<()> {
        self.write_value_to_offset::<f32>(108, &yaw)
    }

    pub fn scale(&self) -> Result<f32> {
        self.read_value_from_offset::<f32>(112)
    }

    pub fn write_scale(&self, scale: f32) -> Result<()> {
        self.write_value_to_offset::<f32>(112, &scale)
    }

    pub fn height(&self) -> Result<f32> {
        self.read_value_from_offset::<f32>(132)
    }

    pub fn write_height(&self, height: f32) -> Result<()> {
        self.write_value_to_offset::<f32>(132, &height)
    }

    pub fn model_update_scheduled(&self) -> Result<bool> {
        self.read_value_from_offset::<bool>(136)
    }

    pub fn write_model_update_scheduled(&self, state: bool) -> Result<()> {
        self.write_value_to_offset::<bool>(136, &state)
    }
}
