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

    pub fn default_background_color_red(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x48)
    }

    pub fn write_default_background_color_red(&self, red: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x48, &red)
    }

    pub fn default_background_color_green(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x4C)
    }

    pub fn write_default_background_color_green(&self, green: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x4C, &green)
    }

    pub fn default_background_color_blue(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x50)
    }

    pub fn write_default_background_color_blue(&self, blue: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x50, &blue)
    }

    pub fn scene_manager(&self) -> Result<Option<DynamicSceneManager>> {
        let addr: u64 = self.inner.read_value_from_offset(0x68)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicSceneManager::new(inner)))
    }

    pub fn shadow_detail(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x8C)
    }

    pub fn write_shadow_detail(&self, value: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x8C, &value)
    }

    pub fn master_scene_root(&self) -> Result<u64> {
        self.inner.read_value_from_offset(0x90)
    }

    pub fn master_collision_scene(&self) -> Result<u64> {
        self.inner.read_value_from_offset(0xA8)
    }

    pub fn nametag_flags(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x190)
    }

    pub fn write_nametag_flags(&self, flags: i32) -> Result<()> {
        self.inner.write_value_to_offset(0x190, &flags)
    }
}
