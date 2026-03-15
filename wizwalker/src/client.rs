use std::sync::Arc;
use windows::Win32::Foundation::HWND;
use crate::errors::Result;
use crate::types::DynamicWindow;

pub struct HookHandler;
impl HookHandler {
    pub fn check_if_hook_active(&self) -> bool { false }
    pub async fn activate_mouseless_cursor_hook(&self) -> Result<()> { Ok(()) }
    pub async fn deactivate_mouseless_cursor_hook(&self) -> Result<()> { Ok(()) }
    pub async fn write_mouse_position(&self, _x: i32, _y: i32) -> Result<()> { Ok(()) }
}

pub struct RootWindow;
impl RootWindow {
    pub async fn get_windows_with_name(&self, _name: &str) -> Result<Vec<DynamicWindow>> { Ok(vec![]) }
}

pub struct Client {
    pub window_handle: HWND,
    pub hook_handler: HookHandler,
    pub root_window: RootWindow,
}
