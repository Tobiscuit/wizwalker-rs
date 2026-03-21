use crate::errors::Result;
use crate::types::{XYZ, Orient};
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::gamebryo_camera::DynamicGamebryoCamera;

pub struct DynamicCameraController {
    pub inner: DynamicMemoryObject,
}

impl DynamicCameraController {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn position(&self) -> Result<XYZ> {
        self.inner.read_value_from_offset(108)
    }

    pub fn write_position(&self, position: &XYZ) -> Result<()> {
        self.inner.write_value_to_offset(108, position)
    }

    pub fn orientation(&self) -> Result<Orient> {
        self.inner.read_value_from_offset(120)
    }

    pub fn write_orientation(&self, orientation: &Orient) -> Result<()> {
        self.inner.write_value_to_offset(120, orientation)
    }

    pub fn pitch(&self) -> Result<f32> {
        self.inner.read_value_from_offset(120)
    }

    pub fn write_pitch(&self, pitch: f32) -> Result<()> {
        self.inner.write_value_to_offset(120, &pitch)
    }

    pub fn roll(&self) -> Result<f32> {
        self.inner.read_value_from_offset(124)
    }

    pub fn write_roll(&self, roll: f32) -> Result<()> {
        self.inner.write_value_to_offset(124, &roll)
    }

    pub fn yaw(&self) -> Result<f32> {
        self.inner.read_value_from_offset(128)
    }

    pub fn write_yaw(&self, yaw: f32) -> Result<()> {
        self.inner.write_value_to_offset(128, &yaw)
    }

    pub fn gamebryo_camera(&self) -> Result<Option<DynamicGamebryoCamera>> {
        let addr: u64 = self.inner.read_value_from_offset(136)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicGamebryoCamera::new(inner)))
    }
}

pub struct DynamicFreeCameraController {
    pub inner: DynamicMemoryObject,
}

impl DynamicFreeCameraController {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }
}

pub struct DynamicElasticCameraController {
    pub inner: DynamicMemoryObject,
}

impl DynamicElasticCameraController {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn check_collisions(&self) -> Result<bool> {
        self.inner.read_value_from_offset(608)
    }

    pub fn write_check_collisions(&self, check_collisions: bool) -> Result<()> {
        self.inner.write_value_to_offset(608, &check_collisions)
    }

    pub fn distance(&self) -> Result<f32> {
        self.inner.read_value_from_offset(300)
    }

    pub fn write_distance(&self, distance: f32) -> Result<()> {
        self.inner.write_value_to_offset(300, &distance)
    }

    pub fn distance_target(&self) -> Result<f32> {
        self.inner.read_value_from_offset(304)
    }

    pub fn write_distance_target(&self, distance_target: f32) -> Result<()> {
        self.inner.write_value_to_offset(304, &distance_target)
    }

    pub fn zoom_resolution(&self) -> Result<f32> {
        self.inner.read_value_from_offset(324)
    }

    pub fn write_zoom_resolution(&self, zoom_resolution: f32) -> Result<()> {
        self.inner.write_value_to_offset(324, &zoom_resolution)
    }

    pub fn max_distance(&self) -> Result<f32> {
        self.inner.read_value_from_offset(328)
    }

    pub fn write_max_distance(&self, max_distance: f32) -> Result<()> {
        self.inner.write_value_to_offset(328, &max_distance)
    }

    pub fn min_distance(&self) -> Result<f32> {
        self.inner.read_value_from_offset(332)
    }

    pub fn write_min_distance(&self, min_distance: f32) -> Result<()> {
        self.inner.write_value_to_offset(332, &min_distance)
    }
}
