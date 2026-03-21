use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

pub struct DynamicCamView {
    pub inner: DynamicMemoryObject,
}

impl DynamicCamView {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
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
