pub trait MemoryHook: Send + Sync {
    fn hook(&mut self);
    fn unhook(&mut self);
    fn bytes_to_write(&self) -> Vec<u8>;
    fn pattern(&self) -> &[u8];
    fn pattern_offset(&self) -> usize;
    fn hook_size(&self) -> usize;
}

pub struct PlayerHook;
impl MemoryHook for PlayerHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0xF2, 0x0F, 0x10, 0x40, 0x58, 0xF2]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 5 }
}

pub struct PlayerStatHook;
impl MemoryHook for PlayerStatHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0x2B, 0xD8, 0xB8, 0x2E, 0x2E, 0x2E, 0x2E, 0x0F, 0x49, 0xC3, 0x48, 0x83, 0xC4, 0x20, 0x5B, 0xC3]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 7 }
}

pub struct QuestHook;
impl MemoryHook for QuestHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0xF3, 0x41, 0x0F, 0x10, 0x2E, 0xFC, 0x0C, 0x00, 0x00, 0xF3, 0x0F, 0x11]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 5 }
}

pub struct ClientHook;
impl MemoryHook for ClientHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0x18, 0x48, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x48, 0x8B, 0x7C, 0x24, 0x38, 0x48, 0x85, 0xFF, 0x74, 0x29, 0x8B, 0xC6, 0xF0, 0x0F, 0xC1, 0x47, 0x08, 0x83, 0xF8, 0x01, 0x75, 0x1D, 0x48, 0x8B, 0x07, 0x48, 0x8B, 0xCF, 0xFF, 0x50, 0x08, 0xF0, 0x0F, 0xC1, 0x77, 0x0C]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 7 }
}

pub struct RootWindowHook;
impl MemoryHook for RootWindowHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x48, 0x8B, 0x01, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0xFF, 0x50, 0x70, 0x84]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 7 }
}

pub struct RenderContextHook;
impl MemoryHook for RenderContextHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0x2E, 0xF3, 0x41, 0x0F, 0x10, 0x28, 0xF3, 0x0F, 0x10, 0x56, 0x04, 0x48, 0x63, 0xC1]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 9 }
}

pub struct MouselessCursorMoveHook;
impl MemoryHook for MouselessCursorMoveHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 0 }
}

pub struct MovementTeleportHook;
impl MemoryHook for MovementTeleportHook {
    fn hook(&mut self) {}
    fn unhook(&mut self) {}
    fn bytes_to_write(&self) -> Vec<u8> { vec![] }
    fn pattern(&self) -> &[u8] {
        &[0x57, 0x48, 0x83, 0xEC, 0x2E, 0x48, 0x8B, 0x99, 0x2E, 0x2E, 0x2E, 0x2E, 0x48, 0x85, 0xDB, 0x74, 0x2E, 0x4C, 0x8B, 0x43, 0x2E, 0x48, 0x8B, 0x5B, 0x2E, 0x48, 0x85, 0xDB, 0x74, 0x2E, 0xF0, 0xFF, 0x43, 0x2E, 0x4D, 0x85, 0xC0, 0x74, 0x2E, 0xF2, 0x0F, 0x10, 0x02, 0xF2, 0x41, 0x0F, 0x11, 0x40, 0x2E, 0x8B, 0x42, 0x2E, 0x41, 0x89, 0x40, 0x2E, 0x41, 0xC6, 0x80, 0x2E]
    }
    fn pattern_offset(&self) -> usize { 0 }
    fn hook_size(&self) -> usize { 5 }
}
