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

    /// Update the gamebryo camera matrix from pitch/yaw/roll orientation.
    ///
    /// Python equivalent:
    /// ```python
    /// async def update_orientation(self, orientation=None):
    ///     gcam = await self.gamebryo_camera()
    ///     view = await gcam.cam_view()
    ///     mat = await gcam.base_matrix()
    ///     if orientation is None:
    ///         orientation = await self.orientation()
    ///     else:
    ///         await self.write_orientation(orientation)
    ///     mat = utils.make_ypr_matrix(mat, orientation)
    ///     await view.write_view_matrix(mat)
    /// ```
    pub fn update_orientation(&self, orientation: Option<Orient>) -> Result<()> {
        let gcam = self.gamebryo_camera()?.ok_or_else(|| {
            crate::errors::WizWalkerError::Other("Gamebryo camera not found".into())
        })?;
        let view = gcam.cam_view()?.ok_or_else(|| {
            crate::errors::WizWalkerError::Other("Cam view not found".into())
        })?;
        let mat = gcam.base_matrix()?;
        let orient = match orientation {
            Some(o) => {
                self.write_orientation(&o)?;
                o
            }
            None => self.orientation()?,
        };
        let result = make_ypr_matrix(&mat, &orient);
        view.write_view_matrix(&result)?;
        Ok(())
    }
}

// ── 3x3 Matrix Math (ported from Python wizwalker/utils.py) ─────────

/// Multiply two 3x3 matrices stored as flat [f32; 9] arrays.
fn multiply3x3matrices(a: &[f32], b: &[f32]) -> Vec<f32> {
    let mut result = vec![0.0f32; 9];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i * 3 + j] += a[i * 3 + k] * b[k * 3 + j];
            }
        }
    }
    result
}

/// Create a 3x3 pitch rotation matrix.
fn pitch_matrix(pitch: f32) -> Vec<f32> {
    let mut result = vec![0.0f32; 9];
    let s = pitch.sin();
    let c = pitch.cos();
    result[0] = c;
    result[1] = s;
    result[3] = -s;
    result[4] = c;
    result[8] = 1.0;
    result
}

/// Create a 3x3 roll rotation matrix.
fn roll_matrix(roll: f32) -> Vec<f32> {
    let mut result = vec![0.0f32; 9];
    let s = roll.sin();
    let c = roll.cos();
    result[0] = 1.0;
    result[4] = c;
    result[5] = s;
    result[7] = -s;
    result[8] = c;
    result
}

/// Create a 3x3 yaw rotation matrix.
fn yaw_matrix(yaw: f32) -> Vec<f32> {
    let mut result = vec![0.0f32; 9];
    let s = yaw.sin();
    let c = yaw.cos();
    result[0] = c;
    result[2] = -s;
    result[4] = 1.0;
    result[6] = s;
    result[8] = c;
    result
}

/// Build a combined yaw-pitch-roll rotation matrix.
/// Python: `make_ypr_matrix(base, orientation)`
fn make_ypr_matrix(base: &[f32], orient: &Orient) -> Vec<f32> {
    let mat = multiply3x3matrices(base, &yaw_matrix(orient.yaw));
    let mat = multiply3x3matrices(&mat, &pitch_matrix(orient.pitch));
    multiply3x3matrices(&mat, &roll_matrix(orient.roll))
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
