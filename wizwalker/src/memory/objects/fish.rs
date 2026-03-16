use crate::errors::{Result, WizWalkerError};
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::fish_template::FishTemplate;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishStatusCode {
    Neutral = 0,
    Scared = 1,
    Unknown2 = 2,
    Unknown3 = 3,
    Escaped = 4,
    Unknown5 = 5,
    Unknown6 = 6,
}

pub struct Fish {
    pub inner: DynamicMemoryObject,
}

impl Fish {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn status_code(&self) -> Result<FishStatusCode> {
        let val: i32 = self.inner.read_value_from_offset(0xB8)?;
        match val {
            0 => Ok(FishStatusCode::Neutral),
            1 => Ok(FishStatusCode::Scared),
            2 => Ok(FishStatusCode::Unknown2),
            3 => Ok(FishStatusCode::Unknown3),
            4 => Ok(FishStatusCode::Escaped),
            5 => Ok(FishStatusCode::Unknown5),
            6 => Ok(FishStatusCode::Unknown6),
            _ => Err(WizWalkerError::ReadingEnumFailed { enum_name: "FishStatusCode".into(), value: val.to_string() }),
        }
    }

    pub async fn write_status_code(&self, val: FishStatusCode) -> Result<()> {
        let enum_val = val as i32;
        self.inner.write_value_to_offset(0xB8, &enum_val)
    }

    pub async fn template(&self) -> Result<FishTemplate> {
        let addr: u64 = self.inner.read_value_from_offset(0xD8)?;
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(FishTemplate::new(inner))
    }

    pub async fn bobber_submerge_ease(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0xE0)
    }

    pub async fn write_bobber_submerge_ease(&self, val: f32) -> Result<()> {
        self.inner.write_value_to_offset(0xE0, &val)
    }

    pub async fn fish_id(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0xE4)
    }

    pub async fn template_id(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0xE8)
    }

    pub async fn size(&self) -> Result<f32> {
        self.inner.read_value_from_offset(0xEC)
    }

    pub async fn is_chest(&self) -> Result<bool> {
        self.inner.read_value_from_offset(0xF0)
    }
}
