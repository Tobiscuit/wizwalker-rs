use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    PostMessageW, SendMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN,
    WM_RBUTTONUP,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use crate::client::Client;
use crate::errors::Result;
use crate::types::DynamicWindow;

#[derive(Clone)]
pub struct MouseHandler {
    client: Arc<Client>,
    click_lock: Arc<Mutex<()>>,
    click_predelay: Duration,
    ref_lock: Arc<Mutex<()>>,
    ref_count: Arc<Mutex<usize>>,
}

impl MouseHandler {
    pub fn new(client: Arc<Client>) -> Self {
        // SetProcessDpiAwareness(2) should be called by the user or client at startup
        Self {
            client,
            click_lock: Arc::new(Mutex::new(())),
            click_predelay: Duration::from_secs_f64(0.02),
            ref_lock: Arc::new(Mutex::new(())),
            ref_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn activate_mouseless(&self) -> Result<()> {
        let count = *self.ref_count.lock().await;
        if count > 0 {
            // "You can't mix managed mouseless with unmanaged mouseless"
            return Err(crate::errors::Error::Runtime(
                "You can't mix managed mouseless with unmanaged mouseless".to_string(),
            ));
        }
        self.client.hook_handler.activate_mouseless_cursor_hook().await
    }

    pub async fn deactivate_mouseless(&self) -> Result<()> {
        let count = *self.ref_count.lock().await;
        if count > 0 {
            return Err(crate::errors::Error::Runtime(
                "You can't mix managed mouseless with unmanaged mouseless".to_string(),
            ));
        }
        self.client.hook_handler.deactivate_mouseless_cursor_hook().await
    }

    pub async fn enter_managed_mouseless(&self) -> Result<()> {
        let _guard = self.ref_lock.lock().await;
        let mut count = self.ref_count.lock().await;
        if *count == 0 && !self.client.hook_handler.check_if_hook_active() {
            self.client.hook_handler.activate_mouseless_cursor_hook().await?;
        }
        *count += 1;
        Ok(())
    }

    pub async fn exit_managed_mouseless(&self) -> Result<()> {
        let _guard = self.ref_lock.lock().await;
        let mut count = self.ref_count.lock().await;
        *count -= 1;
        if *count == 0 && self.client.hook_handler.check_if_hook_active() {
            self.client.hook_handler.deactivate_mouseless_cursor_hook().await?;
        }
        Ok(())
    }

    pub async fn set_mouse_position_to_window(&self, window: &DynamicWindow) -> Result<()> {
        let scaled_rect = window.scale_to_client().await?;
        let center = scaled_rect.center();
        self.set_mouse_position(center.0, center.1, true, false).await
    }

    pub async fn click_window(&self, window: &DynamicWindow) -> Result<()> {
        let scaled_rect = window.scale_to_client().await?;
        let center = scaled_rect.center();
        self.click(center.0, center.1, false, 0.0, false).await
    }

    pub async fn click_window_with_name(&self, name: &str) -> Result<()> {
        let possible_windows = self.client.root_window.get_windows_with_name(name).await?;
        if possible_windows.is_empty() {
            return Err(crate::errors::Error::Value(format!("Window with name {} not found.", name)));
        } else if possible_windows.len() > 1 {
            return Err(crate::errors::Error::Value(format!("Multiple windows with name {}.", name)));
        }
        self.click_window(&possible_windows[0]).await
    }

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

        let handle = self.client.window_handle;
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

        // move mouse outside of client area
        self.set_mouse_position(-100, -100, true, false).await?;

        Ok(())
    }

    pub async fn set_mouse_position(
        &self,
        mut x: i32,
        mut y: i32,
        convert_from_client: bool,
        use_post: bool,
    ) -> Result<()> {
        let handle = self.client.window_handle;

        if convert_from_client {
            let mut point = POINT { x, y };
            unsafe {
                if !ClientToScreen(handle, &mut point).as_bool() {
                    return Err(crate::errors::Error::Runtime("Client to screen conversion failed".to_string()));
                }
            }
            x = point.x;
            y = point.y;
        }

        self.client.hook_handler.write_mouse_position(x, y).await?;

        unsafe {
            if use_post {
                let _ = PostMessageW(Some(handle), WM_MOUSEMOVE, WPARAM(0), LPARAM(0));
            } else {
                SendMessageW(handle, WM_MOUSEMOVE, Some(WPARAM(0)), Some(LPARAM(0)));
            }
        }

        Ok(())
    }
}
