use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, CloseHandle};
use windows::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, SetForegroundWindow, SetWindowTextW,
};
use windows::core::{PCWSTR, PWSTR};

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

pub fn get_foreground_window() -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            None
        } else {
            Some(hwnd.0 as isize)
        }
    }
}

pub fn set_foreground_window(window_handle: isize) -> bool {
    unsafe {
        SetForegroundWindow(HWND(window_handle as *mut _)).as_bool()
    }
}

pub fn get_window_title(handle: isize) -> String {
    let mut buffer: [u16; 100] = [0; 100];
    unsafe {
        let len = GetWindowTextW(HWND(handle as *mut _), &mut buffer);
        String::from_utf16_lossy(&buffer[..len as usize])
    }
}

pub fn set_window_title(handle: isize, window_title: &str) {
    let mut encoded: Vec<u16> = window_title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SetWindowTextW(HWND(handle as *mut _), PCWSTR(encoded.as_ptr()));
    }
}

pub fn get_window_rectangle(handle: isize) -> Rectangle {
    let mut rect = RECT::default();
    unsafe {
        let _ = GetWindowRect(HWND(handle as *mut _), &mut rect);
    }
    Rectangle {
        x1: rect.left,
        y1: rect.top,
        x2: rect.right,
        y2: rect.bottom,
    }
}

pub fn get_pid_from_handle(handle: isize) -> u32 {
    let mut pid = 0;
    unsafe {
        let _ = GetWindowThreadProcessId(HWND(handle as *mut _), Some(&mut pid as *mut u32));
    }
    pid
}

pub fn check_if_process_running(handle: isize) -> bool {
    let mut exit_code = 0;
    unsafe {
        if let Ok(process_handle) = OpenProcess(PROCESS_QUERY_INFORMATION, false, get_pid_from_handle(handle)) {
            let _ = GetExitCodeProcess(process_handle, &mut exit_code);
            let _ = CloseHandle(process_handle);
        }
    }
    exit_code == 259
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let handles = &mut *(lparam.0 as *mut Vec<isize>);
    handles.push(hwnd.0 as isize);
    true.into()
}

pub fn get_windows_from_predicate<F>(mut predicate: F) -> Vec<isize>
where
    F: FnMut(isize) -> bool,
{
    let mut handles: Vec<isize> = Vec::new();
    let lparam = LPARAM(&mut handles as *mut _ as isize);

    unsafe {
        let _ = EnumWindows(Some(enum_windows_callback), lparam);
    }

    handles.into_iter().filter(|&h| predicate(h)).collect()
}

pub fn get_all_wizard_handles() -> Vec<isize> {
    let target_class: Vec<u16> = "Wizard Graphical Client".encode_utf16().chain(std::iter::once(0)).collect();

    get_windows_from_predicate(|handle| {
        let mut buffer: [u16; 100] = [0; 100];
        unsafe {
            let len = GetClassNameW(HWND(handle as *mut _), &mut buffer);
            if len > 0 && buffer[..len as usize] == target_class[..target_class.len()-1] {
                true
            } else {
                false
            }
        }
    })
}

use std::future::Future;
use std::pin::Pin;
use tokio::time::{sleep, timeout, Duration};
use crate::errors::WizWalkerMemoryError;

pub async fn wait_for_value<T, F>(
    mut coro: impl FnMut() -> Pin<Box<dyn Future<Output = Result<T, WizWalkerMemoryError>> + Send>> + Send,
    want: T,
    sleep_time: f32,
) -> Result<T, WizWalkerMemoryError>
where
    T: PartialEq,
{
    loop {
        match coro().await {
            Ok(now) if now == want => return Ok(now),
            _ => sleep(Duration::from_secs_f32(sleep_time)).await,
        }
    }
}

pub async fn maybe_wait_for_value_with_timeout<T>(
    mut coro: impl FnMut() -> Pin<Box<dyn Future<Output = Result<Option<T>, WizWalkerMemoryError>> + Send>> + Send,
    inverse_value: bool,
    timeout_secs: Option<f32>,
) -> Result<Option<T>, WizWalkerMemoryError>
{
    let future = async {
        loop {
            match coro().await {
                Ok(Some(res)) => {
                    if !inverse_value {
                        return Ok(Some(res));
                    }
                }
                Ok(None) => {
                    if inverse_value {
                        return Ok(None);
                    }
                }
                Err(_) => {}
            }
            sleep(Duration::from_millis(500)).await;
        }
    };

    if let Some(t) = timeout_secs {
        match timeout(Duration::from_secs_f32(t), future).await {
            Ok(res) => res,
            Err(_) => Err(WizWalkerMemoryError::Other("Timeout".to_string())),
        }
    } else {
        future.await
    }
}

pub async fn maybe_wait_for_any_value_with_timeout<T>(
    mut coro: impl FnMut() -> Pin<Box<dyn Future<Output = Result<Option<T>, WizWalkerMemoryError>> + Send>> + Send,
    timeout_secs: f32,
) -> Result<Option<T>, WizWalkerMemoryError>
{
    let future = async {
        loop {
            if let Ok(Some(res)) = coro().await {
                return Ok(Some(res));
            }
            sleep(Duration::from_millis(500)).await;
        }
    };

    match timeout(Duration::from_secs_f32(timeout_secs), future).await {
        Ok(res) => res,
        Err(_) => Err(WizWalkerMemoryError::Other("Timeout".to_string())),
    }
}
