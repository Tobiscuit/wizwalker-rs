use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogMode {
    None = 0,
    Linear = 1,
    Exp = 2,
    Exp2 = 3,
    Filter = 4,
}

pub struct DynamicSceneManager {
    pub inner: DynamicMemoryObject,
}

impl DynamicSceneManager {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn fog_mode(&self) -> Result<FogMode> {
        let val: i32 = self.inner.read_value_from_offset(0x180)?;
        match val {
            0 => Ok(FogMode::None),
            1 => Ok(FogMode::Linear),
            2 => Ok(FogMode::Exp),
            3 => Ok(FogMode::Exp2),
            4 => Ok(FogMode::Filter),
            _ => Err(crate::errors::WizWalkerError::ReadingEnumFailed { enum_name: "FogMode".into(), value: val.to_string() }),
        }
    }

    pub async fn write_fog_mode(&self, mode: FogMode) -> Result<()> {
        let val = mode as i32;
        self.inner.write_value_to_offset(0x180, &val)
    }

    pub async fn fog_density(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x184)
    }

    pub async fn write_fog_density(&self, density: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x184, &density)
    }

    pub async fn fog_density_target(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x188)
    }

    pub async fn write_fog_density_target(&self, target: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x188, &target)
    }

    pub async fn fog_start_density(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x18C)
    }

    pub async fn write_fog_start_density(&self, start: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x18C, &start)
    }

    pub async fn fog_color_red(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x190)
    }

    pub async fn write_fog_color_red(&self, red: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x190, &red)
    }

    pub async fn fog_color_green(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x194)
    }

    pub async fn write_fog_color_green(&self, green: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x194, &green)
    }

    pub async fn fog_color_blue(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0x198)
    }

    pub async fn write_fog_color_blue(&self, blue: f32) -> Result<()> {
        self.inner.write_value_to_offset(0x198, &blue)
    }
}
