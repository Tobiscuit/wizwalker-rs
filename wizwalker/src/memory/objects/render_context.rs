use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

pub struct RenderContext {
    pub inner: DynamicMemoryObject,
}

impl RenderContext {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn ui_scale(&self) -> Result<f32> {
        self.inner.read_value_from_offset(152)
    }
}

pub struct CurrentRenderContext {
    pub inner: DynamicMemoryObject,
}

impl CurrentRenderContext {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }
}
