use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use crate::memory::reader::MemoryReaderExt;

pub struct DynamicCamView {
    pub inner: DynamicMemoryObject,
}

impl DynamicCamView {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    /// Read the 3x3 view matrix (9 floats at offset 80).
    /// Python: `await self.read_vector(80, 9)`
    pub fn view_matrix(&self) -> Result<Vec<f32>> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let mut matrix = Vec::with_capacity(9);
        for i in 0..9u64 {
            let val: f32 = reader.read_typed((base_address + 80 + i * 4) as usize)?;
            matrix.push(val);
        }
        Ok(matrix)
    }

    /// Write the 3x3 view matrix (9 floats at offset 80).
    /// Python: `await self.write_vector(80, tuple(values), 9)`
    pub fn write_view_matrix(&self, vals: &[f32]) -> Result<()> {
        if vals.len() != 9 {
            return Err(crate::errors::WizWalkerError::Other(
                "view_matrix requires 9 values".into(),
            ));
        }
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        for i in 0..9u64 {
            reader.write_typed((base_address + 80 + i * 4) as usize, &vals[i as usize])?;
        }
        Ok(())
    }

    pub fn cull_near(&self) -> Result<f32> {
        self.inner.read_value_from_offset(304)
    }

    pub fn write_cull_near(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(304, &value)
    }

    pub fn cull_far(&self) -> Result<f32> {
        self.inner.read_value_from_offset(308)
    }

    pub fn write_cull_far(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(308, &value)
    }

    pub fn base_cull_near(&self) -> Result<f32> {
        self.inner.read_value_from_offset(316)
    }

    pub fn write_base_cull_near(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(316, &value)
    }

    pub fn base_cull_far(&self) -> Result<f32> {
        self.inner.read_value_from_offset(320)
    }

    pub fn write_base_cull_far(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(320, &value)
    }

    pub fn viewport_left(&self) -> Result<f32> {
        self.inner.read_value_from_offset(288)
    }

    pub fn write_viewport_left(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(288, &value)
    }

    pub fn viewport_right(&self) -> Result<f32> {
        self.inner.read_value_from_offset(292)
    }

    pub fn write_viewport_right(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(292, &value)
    }

    pub fn viewport_top(&self) -> Result<f32> {
        self.inner.read_value_from_offset(296)
    }

    pub fn write_viewport_top(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(296, &value)
    }

    pub fn viewport_bottom(&self) -> Result<f32> {
        self.inner.read_value_from_offset(300)
    }

    pub fn write_viewport_bottom(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(300, &value)
    }

    pub fn screenport_left(&self) -> Result<f32> {
        self.inner.read_value_from_offset(324)
    }

    pub fn write_screenport_left(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(324, &value)
    }

    pub fn screenport_right(&self) -> Result<f32> {
        self.inner.read_value_from_offset(328)
    }

    pub fn write_screenport_right(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(328, &value)
    }

    pub fn screenport_top(&self) -> Result<f32> {
        self.inner.read_value_from_offset(332)
    }

    pub fn write_screenport_top(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(332, &value)
    }

    pub fn screenport_bottom(&self) -> Result<f32> {
        self.inner.read_value_from_offset(336)
    }

    pub fn write_screenport_bottom(&self, value: f32) -> Result<()> {
        self.inner.write_value_to_offset(336, &value)
    }
}
