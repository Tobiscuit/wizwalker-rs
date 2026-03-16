use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::scene_manager::DynamicSceneManager;

pub struct DynamicGamebryoPresenter {
    pub inner: DynamicMemoryObject,
}

impl DynamicGamebryoPresenter {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn default_background_color_red(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x48)
    }

    pub async fn write_default_background_color_red(&self, red: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x48, &red)
    }

    pub async fn default_background_color_green(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x4C)
    }

    pub async fn write_default_background_color_green(&self, green: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x4C, &green)
    }

    pub async fn default_background_color_blue(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x50)
    }

    pub async fn write_default_background_color_blue(&self, blue: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x50, &blue)
    }

    pub async fn scene_manager(&self) -> Result<Option<DynamicSceneManager>> {
        let addr: u64 = self.inner.read_value_from_offset(0x68)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicSceneManager::new(inner)))
    }

    pub async fn shadow_detail(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x8C)
    }

    pub async fn write_shadow_detail(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x8C, &value)
    }

    pub async fn master_scene_root(&self) -> Result<u64> {
        self.inner.read_value_from_offset(0x90)
    }

    pub async fn master_collision_scene(&self) -> Result<u64> {
        self.inner.read_value_from_offset(0xA8)
    }

    pub async fn nametag_flags(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x190)
    }

    pub async fn write_nametag_flags(&self, flags: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x190, &flags)
    }
}
