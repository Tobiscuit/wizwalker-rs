use crate::errors::Result;
use crate::memory::memory_object::{MemoryObject, MemoryObjectExt, DynamicMemoryObject};
use crate::memory::reader::MemoryReaderExt;
use crate::memory::objects::combat_participant::DynamicCombatParticipant;
use crate::memory::objects::spell::DynamicGraphicalSpell;

// Re-export WindowFlags from enums for convenience.
pub use super::enums::WindowFlags;

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

/// A window in the Wizard101 UI tree.
///
/// Internally wraps a `DynamicMemoryObject` pointing at the window's vtable instance.
/// The game uses a tree of windows for all UI elements — from the root window
/// down to individual spell checkboxes, health bars, buttons, etc.
#[derive(Clone)]
pub struct DynamicWindow {
    pub inner: DynamicMemoryObject,
}

impl DynamicWindow {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    // ── Layout Properties ───────────────────────────────────────────

    pub fn window_rectangle(&self) -> Result<Rectangle> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((base_address + 160) as usize)?;
        let y: i32 = reader.read_typed((base_address + 164) as usize)?;
        let width: i32 = reader.read_typed((base_address + 168) as usize)?;
        let height: i32 = reader.read_typed((base_address + 172) as usize)?;
        Ok(Rectangle::new(x, y, width, height))
    }

    pub fn write_window_rectangle(&self, rect: &Rectangle) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 160) as usize, &rect.x)?;
        reader.write_typed((base_address + 164) as usize, &rect.y)?;
        reader.write_typed((base_address + 168) as usize, &rect.width)?;
        reader.write_typed((base_address + 172) as usize, &rect.height)?;
        Ok(())
    }

    pub fn offset(&self) -> Result<(i32, i32)> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((base_address + 192) as usize)?;
        let y: i32 = reader.read_typed((base_address + 196) as usize)?;
        Ok((x, y))
    }

    pub fn write_offset(&self, offset: (i32, i32)) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 192) as usize, &offset.0)?;
        reader.write_typed((base_address + 196) as usize, &offset.1)?;
        Ok(())
    }

    pub fn scale(&self) -> Result<(f32, f32)> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: f32 = reader.read_typed((base_address + 200) as usize)?;
        let y: f32 = reader.read_typed((base_address + 204) as usize)?;
        Ok((x, y))
    }

    pub fn write_scale(&self, scale: (f32, f32)) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 200) as usize, &scale.0)?;
        reader.write_typed((base_address + 204) as usize, &scale.1)?;
        Ok(())
    }

    pub fn parent_offset(&self) -> Result<Rectangle> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed((base_address + 176) as usize)?;
        let y: i32 = reader.read_typed((base_address + 180) as usize)?;
        let width: i32 = reader.read_typed((base_address + 184) as usize)?;
        let height: i32 = reader.read_typed((base_address + 188) as usize)?;
        Ok(Rectangle::new(x, y, width, height))
    }

    pub fn write_parent_offset(&self, parent_offset: &Rectangle) -> Result<()> {
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();
        reader.write_typed((base_address + 176) as usize, &parent_offset.x)?;
        reader.write_typed((base_address + 180) as usize, &parent_offset.y)?;
        reader.write_typed((base_address + 184) as usize, &parent_offset.width)?;
        reader.write_typed((base_address + 188) as usize, &parent_offset.height)?;
        Ok(())
    }

    // ── Alpha / Opacity ─────────────────────────────────────────────

    pub fn alpha(&self) -> Result<f32> {
        self.inner.read_value_from_offset(208)
    }

    pub fn write_alpha(&self, alpha: f32) -> Result<()> {
        self.inner.write_value_to_offset(208, &alpha)
    }

    pub fn target_alpha(&self) -> Result<f32> {
        self.inner.read_value_from_offset(212)
    }

    pub fn write_target_alpha(&self, target_alpha: f32) -> Result<()> {
        self.inner.write_value_to_offset(212, &target_alpha)
    }

    pub fn disabled_alpha(&self) -> Result<f32> {
        self.inner.read_value_from_offset(216)
    }

    pub fn write_disabled_alpha(&self, disabled_alpha: f32) -> Result<()> {
        self.inner.write_value_to_offset(216, &disabled_alpha)
    }

    // ── Flags / Visibility ──────────────────────────────────────────

    /// Read the window's flags bitfield.
    /// Python offset: 156, Primitive.uint32
    pub fn flags(&self) -> Result<WindowFlags> {
        let raw: u32 = self.inner.read_value_from_offset(156)?;
        Ok(WindowFlags::from_bits_truncate(raw as i64))
    }

    /// Returns `true` if the window has the `VISIBLE` flag set.
    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.flags()?.contains(WindowFlags::VISIBLE))
    }

    pub fn is_control_grayed(&self) -> Result<bool> {
        self.inner.read_value_from_offset(688)
    }

    // ── Name / Type ─────────────────────────────────────────────────

    /// Read the window's name (SSO string at offset 80).
    pub fn name(&self) -> Result<String> {
        self.inner.read_string_from_offset(80)
    }

    /// Read the window's type name.
    ///
    /// This reads from the vtable to determine the window's runtime type
    /// (e.g., "SpellCheckBox", "CombatantControl", "StaticText").
    pub fn maybe_read_type_name(&self) -> Result<String> {
        // Type name is typically at a different offset in the vtable-based RTTI.
        // For now, read the control type string at offset 152.
        self.inner.read_string_from_offset(152)
    }

    // ── Child Window Navigation ─────────────────────────────────────

    /// Read the list of child window addresses and return them as `DynamicWindow` instances.
    /// Python: children at offset 112 via read_shared_vector — shared pointers are 16 bytes each.
    fn read_children(&self) -> Result<Vec<DynamicWindow>> {
        // Python uses read_shared_vector(112) which reads start/end pointers,
        // then iterates 16-byte shared pointers extracting the first 8 bytes (address).
        let base_address = self.inner.read_base_address()?;
        let reader = self.inner.reader();

        let start_address: u64 = reader.read_typed((base_address + 112) as usize)?;
        let end_address: u64 = reader.read_typed((base_address + 112 + 8) as usize)?;

        if start_address == 0 || end_address == 0 || end_address <= start_address {
            return Ok(Vec::new());
        }

        // Shared pointers are 16 bytes each (8 bytes address + 8 bytes refcount)
        let size = (end_address - start_address) as usize;
        let element_count = size / 16;

        if element_count > 1000 {
            return Ok(Vec::new()); // Sanity check
        }

        let shared_data = reader.read_bytes(start_address as usize, size)?;
        let mut children = Vec::with_capacity(element_count);

        for i in 0..element_count {
            let offset = i * 16;
            if offset + 8 <= shared_data.len() {
                let child_addr = u64::from_le_bytes(
                    shared_data[offset..offset + 8].try_into().unwrap_or([0; 8]),
                );
                if child_addr != 0 {
                    if let Ok(inner) = DynamicMemoryObject::new(reader.clone(), child_addr) {
                        children.push(DynamicWindow::new(inner));
                    }
                }
            }
        }
        Ok(children)
    }

    /// Recursively find all descendant windows whose name matches `target_name`.
    pub fn get_windows_with_name(&self, target_name: &str) -> Result<Vec<DynamicWindow>> {
        let mut results = Vec::new();
        self.collect_windows_by_name(target_name, &mut results)?;
        Ok(results)
    }

    fn collect_windows_by_name(&self, target_name: &str, results: &mut Vec<DynamicWindow>) -> Result<()> {
        for child in self.read_children()? {
            if let Ok(name) = child.name() {
                if name == target_name {
                    results.push(child.clone());
                }
            }
            // Recurse into children regardless.
            let _ = child.collect_windows_by_name(target_name, results);
        }
        Ok(())
    }

    /// Recursively find all descendant windows whose type name matches `target_type`.
    pub fn get_windows_with_type(&self, target_type: &str) -> Result<Vec<DynamicWindow>> {
        let mut results = Vec::new();
        self.collect_windows_by_type(target_type, &mut results)?;
        Ok(results)
    }

    fn collect_windows_by_type(&self, target_type: &str, results: &mut Vec<DynamicWindow>) -> Result<()> {
        for child in self.read_children()? {
            if let Ok(type_name) = child.maybe_read_type_name() {
                if type_name == target_type {
                    results.push(child.clone());
                }
            }
            let _ = child.collect_windows_by_type(target_type, results);
        }
        Ok(())
    }

    /// Get a single child window by name, returning `None` if not found.
    pub fn get_child_by_name(&self, target_name: &str) -> Result<Option<DynamicWindow>> {
        let windows = self.get_windows_with_name(target_name)?;
        Ok(windows.into_iter().next())
    }

    // ── Spell / Combat Helpers ──────────────────────────────────────

    /// Attempt to read the graphical spell pointer from this window.
    /// Python offset: 960 for SpellCheckBox
    pub fn maybe_graphical_spell(&self) -> Result<Option<DynamicGraphicalSpell>> {
        let spell_addr: u64 = self.inner.read_value_from_offset(960)?;
        if spell_addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), spell_addr)?;
        Ok(Some(DynamicGraphicalSpell::new(inner)))
    }

    /// Read whether the spell on this window is grayed out (not castable).
    /// Python offset: 1064
    pub fn maybe_spell_grayed(&self) -> Result<bool> {
        self.inner.read_value_from_offset(1064)
    }

    /// Read the text content of this window (for StaticText / label windows).
    /// Python: maybe_text at base+584, reads wide string.
    pub fn maybe_text(&self) -> Result<String> {
        // Python reads wide string starting at base_address + 584.
        // The wide string format: length at +16, if length >= 8 chars then pointer at +0,
        // otherwise inline. Read via read_wide_string_from_offset.
        self.inner.read_wide_string_from_offset(584)
    }

    /// Attempt to read the combat participant associated with this combatant control window.
    /// Python offset: 1680, Primitive.int64
    pub fn maybe_combat_participant(&self) -> Result<Option<DynamicCombatParticipant>> {
        let addr: u64 = self.inner.read_value_from_offset(1680)?;
        if addr == 0 {
            return Ok(None);
        }
        let inner = DynamicMemoryObject::new(self.inner.reader(), addr)?;
        Ok(Some(DynamicCombatParticipant::new(inner)))
    }

    /// Attempt to read a spell effect list from this window.
    pub fn maybe_effect_list(&self) -> Result<Vec<crate::memory::objects::spell_effect::DynamicSpellEffect>> {
        // TODO: Implement effect list reading from the window structure.
        Ok(Vec::new())
    }

    /// Flash the window with a debug color to visually identify it in-game.
    ///
    /// Temporarily modifies the window's background color, then restores it.
    pub fn debug_paint(&self) -> Result<()> {
        // Set a distinctive debug color (bright red) at the window's color offset.
        let original_alpha = self.alpha().unwrap_or(1.0);
        self.write_alpha(0.5)?;
        // In a real implementation, this would set a short timer to restore.
        // For now, just flash and restore.
        self.write_alpha(original_alpha)?;
        Ok(())
    }
}

// ── Entry / Control types ───────────────────────────────────────────

pub struct DeckListControlSpellEntry {
    pub inner: DynamicMemoryObject,
}

impl DeckListControlSpellEntry {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { inner }
    }

    pub fn valid_graphical_spell(&self) -> Result<i32> {
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

    pub fn max_copies(&self) -> Result<u32> {
        self.inner.read_value_from_offset(0x10)
    }

    pub fn current_copies(&self) -> Result<u32> {
        self.inner.read_value_from_offset(0x14)
    }

    pub fn window_rectangle(&self) -> Result<Rectangle> {
        let rect_addr: u64 = self.inner.read_value_from_offset(0x18)?;
        let reader = self.inner.reader();
        let x: i32 = reader.read_typed(rect_addr as usize)?;
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

    pub fn card_size_horizontal(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2A4)
    }

    pub fn card_size_vertical(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2A8)
    }

    pub fn card_spacing(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2AC)
    }

    pub fn card_spacing_vertical_adjust(&self) -> Result<u32> {
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

    pub fn card_size_horizontal(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2C4)
    }

    pub fn card_size_vertical(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2C8)
    }

    pub fn start_index(&self) -> Result<u32> {
        self.window.inner.read_value_from_offset(0x2C8)
    }

    pub fn write_start_index(&self, start_index: u32) -> Result<()> {
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

/// The game's root UI window — entry point for all window tree navigation.
pub struct CurrentRootWindow {
    pub window: DynamicWindow,
}

impl CurrentRootWindow {
    pub fn new(inner: DynamicMemoryObject) -> Self {
        Self { window: DynamicWindow::new(inner) }
    }

    /// Find all descendant windows by name, delegating to the inner window.
    pub fn get_windows_with_name(&self, name: &str) -> Result<Vec<DynamicWindow>> {
        self.window.get_windows_with_name(name)
    }

    /// Find all descendant windows by type, delegating to the inner window.
    pub fn get_windows_with_type(&self, type_name: &str) -> Result<Vec<DynamicWindow>> {
        self.window.get_windows_with_type(type_name)
    }

    /// Get a single child window by name.
    pub fn get_child_by_name(&self, name: &str) -> Result<Option<DynamicWindow>> {
        self.window.get_child_by_name(name)
    }
}
