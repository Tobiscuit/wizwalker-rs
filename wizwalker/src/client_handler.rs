use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, GetWindowTextW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

use crate::client::Client;
use crate::errors::WizError;

pub struct ClientHandler {
    pub clients: Vec<Arc<Mutex<Client>>>,
    managed_handles: Vec<HWND>,
}

impl ClientHandler {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            managed_handles: Vec::new(),
        }
    }

    pub fn install_location(&self) -> PathBuf {
        // TODO: utils::get_wiz_install()
        PathBuf::from("C:\\ProgramData\\KingsIsle Entertainment\\Wizard101")
    }

    pub fn start_wiz_client() {
        // TODO: utils::start_instance()
    }

    pub async fn get_foreground_client(&self) -> Option<Arc<Mutex<Client>>> {
        for client in &self.clients {
            if client.lock().await.is_foreground() {
                return Some(client.clone());
            }
        }
        None
    }

    pub async fn get_new_clients(&mut self) -> Result<Vec<Arc<Mutex<Client>>>, WizError> {
        let mut new_clients = Vec::new();

        unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let mut title: [u16; 256] = [0; 256];
            let len = unsafe { GetWindowTextW(hwnd, &mut title) };
            if len > 0 {
                let title_str = String::from_utf16_lossy(&title[..len as usize]);
                if title_str.contains("Wizard101") {
                    let handles_ptr = lparam.0 as *mut Vec<HWND>;
                    let handles = unsafe { &mut *handles_ptr };
                    handles.push(hwnd);
                }
            }
            BOOL(1)
        }

        let mut hwnds: Vec<HWND> = Vec::new();
        unsafe {
            let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut hwnds as *mut _ as isize));
        }

        for hwnd in hwnds {
            if !self.managed_handles.contains(&hwnd) {
                self.managed_handles.push(hwnd);

                let mut process_id = 0;
                unsafe {
                    GetWindowThreadProcessId(hwnd, Some(&mut process_id));
                }

                if process_id != 0 {
                    let process_handle = unsafe {
                        OpenProcess(PROCESS_ALL_ACCESS, false.into(), process_id)
                    }.map_err(|e| WizError::WindowsError(e))?;

                    let client = Client::new(hwnd, process_handle, process_id);
                    let client_arc = Arc::new(Mutex::new(client));
                    self.clients.push(client_arc.clone());
                    new_clients.push(client_arc);
                }
            }
        }

        Ok(new_clients)
    }

    pub async fn remove_dead_clients(&mut self) -> Vec<Arc<Mutex<Client>>> {
        let mut dead_clients = Vec::new();
        let mut alive_clients = Vec::new();

        for client_arc in self.clients.drain(..) {
            let is_running = client_arc.lock().await.is_running();
            if is_running {
                alive_clients.push(client_arc);
            } else {
                dead_clients.push(client_arc);
            }
        }

        self.clients = alive_clients;
        // Also remove dead handles from managed_handles
        self.managed_handles.clear();
        for client in &self.clients {
            self.managed_handles.push(client.lock().await.window_handle);
        }

        dead_clients
    }

    pub async fn get_ordered_clients(&self) -> Vec<Arc<Mutex<Client>>> {
        // TODO: Get client's ordered by their position on the screen
        // return utils.order_clients(self.clients)
        self.clients.clone()
    }

    pub async fn activate_all_client_hooks(&self, _wait_for_ready: bool) {
        // Activate hooks for all clients
    }

    pub async fn activate_all_client_mouseless(&self) {
        // Activates mouseless hook for all clients
    }

    pub async fn close(&mut self) {
        for client in &self.clients {
            client.lock().await.close().await;
        }
        self.clients.clear();
        self.managed_handles.clear();
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        // Drop logic if necessary, though true async drop is tricky in Rust.
        // The Python client handler has `async def close(self)` and `__aexit__` which calls it.
    }
}
