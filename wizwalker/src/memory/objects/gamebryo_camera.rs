use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};
use super::cam_view::DynamicCamView;

pub struct DynamicGamebryoCamera {
    pub inner: DynamicMemoryObject,
}

impl DynamicGamebryoCamera {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn base_matrix(&self) -> Result<Vec<f32>> {
        let mut matrix = Vec::with_capacity(9);
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        for i in 0..9 {
            let val: f32 = reader.read_typed((base_address + 164 + i * 4) as usize)?;
            matrix.push(val);
        }
        Ok(matrix)
    }

    pub async fn write_base_matrix(&self, vals: &[f32]) -> Result<()> {
        if vals.len() != 9 {
            return Err(crate::errors::WizWalkerError::Other("base_matrix requires 9 values".into()));
        }
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        for i in 0..9 {
            reader.write_typed((base_address + 164 + i * 4) as usize, &vals[i as usize])?;
        }
        Ok(())
    }

    pub async fn cam_view(&self) -> Result<Option<DynamicCamView>> {
        let addr: u64 = self.inner.read_value_from_offset(200)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicCamView::new(inner)))
    }
}
