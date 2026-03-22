use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SendMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN,
    WM_RBUTTONUP,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Foundation::POINT;
use crate::errors::Result;
use crate::memory::objects::window::DynamicWindow;

/// Handles mouse input to the Wizard101 game window.
///
/// Uses `SendMessage`/`PostMessage` to simulate mouse clicks at coordinates
/// within the game window. Coordinates can be in client-space (relative to window)
/// or screen-space.
#[derive(Clone)]
pub struct MouseHandler {
    window_handle: HWND,
    click_lock: Arc<Mutex<()>>,
    click_predelay: Duration,
}

impl MouseHandler {
    pub fn new(window_handle: HWND) -> Self {
        Self {
            window_handle,
            click_lock: Arc::new(Mutex::new(())),
            click_predelay: Duration::from_secs_f64(0.02),
        }
    }

    /// Move the mouse cursor to the center of a window.
    pub async fn set_mouse_position_to_window(&self, window: &DynamicWindow) -> Result<()> {
        let rect = window.window_rectangle()?;
        let center_x = rect.x + rect.width / 2;
        let center_y = rect.y + rect.height / 2;
        self.set_mouse_position(center_x, center_y, true, false).await
    }

    /// Click on a window. If `right_click` is `true`, sends a right-click instead.
    pub async fn use_potion_if_needed(&self, _health_percent: i32, _mana_percent: i32) -> Result<()> {
        // BUG: (from Python original) this is a complex check and UI interaction in utils.py
        Ok(())
    }

    pub async fn click_window(&self, window: &DynamicWindow, right_click: bool) -> Result<()> {
        let rect = window.window_rectangle()?;
        let center_x = rect.x + rect.width / 2;
        let center_y = rect.y + rect.height / 2;
        self.click(center_x, center_y, right_click, 0.0, false).await
    }

    /// Click on a window found by name. If `right_click` is `true`, sends a right-click.
    pub async fn click_window_with_name(&self, _name: &str, _right_click: bool) -> Result<()> {
        // TODO: Navigate the window tree to find the window by name, then click.
        // This requires access to the root window, which should be passed to MouseHandler
        // or the caller should find the window and call click_window directly.
        Err(crate::errors::WizWalkerError::Other(
            "click_window_with_name: root window not available in MouseHandler".to_string(),
        ))
    }

    /// Low-level click at screen coordinates.
    pub async fn click(
        &self,
        x: i32,
        y: i32,
        right_click: bool,
        sleep_duration: f64,
        use_post: bool,
    ) -> Result<()> {
        let (button_down, button_up) = if right_click {
            (WM_RBUTTONDOWN, WM_RBUTTONUP)
        } else {
            (WM_LBUTTONDOWN, WM_LBUTTONUP)
        };

        let handle = self.window_handle;
        let _guard = self.click_lock.lock().await;

        self.set_mouse_position(x, y, true, false).await?;
        sleep(self.click_predelay).await;

        unsafe {
            if use_post {
                let _ = PostMessageW(Some(handle), button_down, WPARAM(1), LPARAM(0));
            } else {
                SendMessageW(handle, button_down, Some(WPARAM(1)), Some(LPARAM(0)));
            }
        }

        if sleep_duration > 0.0 {
            sleep(Duration::from_secs_f64(sleep_duration)).await;
        }

        unsafe {
            if use_post {
                let _ = PostMessageW(Some(handle), button_up, WPARAM(0), LPARAM(0));
            } else {
                SendMessageW(handle, button_up, Some(WPARAM(0)), Some(LPARAM(0)));
            }
        }

        // Move mouse outside of client area.
        self.set_mouse_position(-100, -100, true, false).await?;

        Ok(())
    }

    /// Set the mouse position, optionally converting from client coordinates.
    pub async fn set_mouse_position(
        &self,
        mut x: i32,
        mut y: i32,
        convert_from_client: bool,
        use_post: bool,
    ) -> Result<()> {
        let handle = self.window_handle;

        if convert_from_client {
            let mut point = POINT { x, y };
            unsafe {
                if !ClientToScreen(handle, &mut point).as_bool() {
                    return Err(crate::errors::WizWalkerError::Other(
                        "Client to screen conversion failed".to_string(),
                    ));
                }
            }
            x = point.x;
            y = point.y;
        }

        // The mouse position for the game needs to be packed as LPARAM (y << 16 | x).
        let lparam = LPARAM(((y as i32) << 16 | (x as i32 & 0xFFFF)) as isize);

        unsafe {
            if use_post {
                let _ = PostMessageW(Some(handle), WM_MOUSEMOVE, WPARAM(0), lparam);
            } else {
                SendMessageW(handle, WM_MOUSEMOVE, Some(WPARAM(0)), Some(lparam));
            }
        }

        Ok(())
    }
}
