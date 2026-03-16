use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, DynamicMemoryObject};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }
}

pub struct DynamicWindow {
    pub inner: DynamicMemoryObject,
}

impl DynamicWindow {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn window_rectangle(&self) -> Result<Rectangle> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((base_address + 160) as usize)?;
        let y: i32 = reader.read_typed((base_address + 164) as usize)?;
        let width: i32 = reader.read_typed((base_address + 168) as usize)?;
        let height: i32 = reader.read_typed((base_address + 172) as usize)?;
        Ok(Rectangle::new(x, y, width, height))
    }

    pub async fn write_window_rectangle(&self, window_rectangle: &Rectangle) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 160) as usize, &window_rectangle.x)?;
        reader.write_typed((base_address + 164) as usize, &window_rectangle.y)?;
        reader.write_typed((base_address + 168) as usize, &window_rectangle.width)?;
        reader.write_typed((base_address + 172) as usize, &window_rectangle.height)?;
        Ok(())
    }

    pub async fn target_alpha(&self) -> Result<f32> {
        self.inner.read_value_from_offset(212)
    }

    pub async fn write_target_alpha(&self, target_alpha: f32) -> Result<()> {
        self.inner.write_value_to_offset(212, &target_alpha)
    }

    pub async fn disabled_alpha(&self) -> Result<f32> {
        self.inner.read_value_from_offset(216)
    }

    pub async fn write_disabled_alpha(&self, disabled_alpha: f32) -> Result<()> {
        self.inner.write_value_to_offset(216, &disabled_alpha)
    }

    pub async fn alpha(&self) -> Result<f32> {
        self.inner.read_value_from_offset(208)
    }

    pub async fn write_alpha(&self, alpha: f32) -> Result<()> {
        self.inner.write_value_to_offset(208, &alpha)
    }

    pub async fn offset(&self) -> Result<(i32, i32)> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((base_address + 192) as usize)?;
        let y: i32 = reader.read_typed((base_address + 196) as usize)?;
        Ok((x, y))
    }

    pub async fn write_offset(&self, offset: (i32, i32)) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 192) as usize, &offset.0)?;
        reader.write_typed((base_address + 196) as usize, &offset.1)?;
        Ok(())
    }

    pub async fn scale(&self) -> Result<(f32, f32)> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: f32 = reader.read_typed((base_address + 200) as usize)?;
        let y: f32 = reader.read_typed((base_address + 204) as usize)?;
        Ok((x, y))
    }

    pub async fn write_scale(&self, scale: (f32, f32)) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 200) as usize, &scale.0)?;
        reader.write_typed((base_address + 204) as usize, &scale.1)?;
        Ok(())
    }

    pub async fn parent_offset(&self) -> Result<Rectangle> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((base_address + 176) as usize)?;
        let y: i32 = reader.read_typed((base_address + 180) as usize)?;
        let width: i32 = reader.read_typed((base_address + 184) as usize)?;
        let height: i32 = reader.read_typed((base_address + 188) as usize)?;
        Ok(Rectangle::new(x, y, width, height))
    }

    pub async fn write_parent_offset(&self, parent_offset: &Rectangle) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 176) as usize, &parent_offset.x)?;
        reader.write_typed((base_address + 180) as usize, &parent_offset.y)?;
        reader.write_typed((base_address + 184) as usize, &parent_offset.width)?;
        reader.write_typed((base_address + 188) as usize, &parent_offset.height)?;
        Ok(())
    }

    pub async fn is_control_grayed(&self) -> Result<bool> {
        self.inner.read_value_from_offset(688)
    }
}

pub struct DeckListControlSpellEntry {
    pub inner: DynamicMemoryObject,
}

impl DeckListControlSpellEntry {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn valid_graphical_spell(&self) -> Result<i32> {
        self.inner.read_value_from_offset(0x10)
    }
}

pub struct SpellListControlSpellEntry {
    pub inner: DynamicMemoryObject,
}

impl SpellListControlSpellEntry {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub async fn max_copies(&self) -> Result<u32> {
        self.inner.read_value_from_offset(0x10)
    }

    pub async fn current_copies(&self) -> Result<u32> {
        self.inner.read_value_from_offset(0x14)
    }

    pub async fn window_rectangle(&self) -> Result<Rectangle> {
        let rect_addr: u64 = self.inner.read_value_from_offset(0x18)?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((rect_addr) as usize)?;
        let y: i32 = reader.read_typed((rect_addr + 4) as usize)?;
        let width: i32 = reader.read_typed((rect_addr + 8) as usize)?;
        let height: i32 = reader.read_typed((rect_addr + 12) as usize)?;
        Ok(Rectangle::new(x, y, width, height))
    }
}

pub struct DynamicDeckListControl {
    pub window: DynamicWindow,
}

impl DynamicDeckListControl {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { window: DynamicWindow::new(inner) }
    }

    pub async fn card_size_horizontal(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2A4)
    }

    pub async fn card_size_vertical(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2A8)
    }

    pub async fn card_spacing(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2AC)
    }

    pub async fn card_spacing_vertical_adjust(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2B0)
    }
}

pub struct DynamicSpellListControl {
    pub window: DynamicWindow,
}

impl DynamicSpellListControl {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { window: DynamicWindow::new(inner) }
    }

    pub async fn card_size_horizontal(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2C4)
    }

    pub async fn card_size_vertical(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2C8)
    }

    pub async fn start_index(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2C8)
    }

    pub async fn write_start_index(&self, start_index: u32) -> Result<()> {
        self.window.inner.write_value_to_offset(0x2C8, &start_index)
    }
}

pub struct DynamicGraphicalSpellWindow {
    pub window: DynamicWindow,
}

impl DynamicGraphicalSpellWindow {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { window: DynamicWindow::new(inner) }
    }
}

pub struct CurrentRootWindow {
    pub window: DynamicWindow,
}

impl CurrentRootWindow {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { window: DynamicWindow::new(inner) }
    }
}
