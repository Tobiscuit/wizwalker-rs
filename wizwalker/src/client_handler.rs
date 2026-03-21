use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetClassNameW, GetWindowThreadProcessId};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

use crate::client::Client;
use crate::errors::WizError;

pub struct ClientHandler {
    pub clients: Vec<Arc<Mutex<Client>>>,
    managed_handles: Vec<HWND>,
}

// SAFETY: HWND is a raw kernel handle that is valid across threads within
// the same process. Client is already Send, Arc<Mutex<Client>> is Send+Sync.
unsafe impl Send for ClientHandler {}
unsafe impl Sync for ClientHandler {}

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

    /// Return the client that currently has foreground focus, if any.
    ///
    /// # Python equivalent
    /// `ClientHandler.get_foreground_client()` — sync `def` in Python.
    pub fn get_foreground_client(&self) -> Option<Arc<Mutex<Client>>> {
        for client in &self.clients {
            if client.blocking_lock().is_foreground() {
                return Some(client.clone());
            }
        }
        None
    }

    /// Scan for new Wizard101 game instances not yet managed.
    ///
    /// Uses EnumWindows to find all windows with "Wizard101" in the title,
    /// creates Client objects for new ones, and returns them.
    ///
    /// # Python equivalent
    /// `ClientHandler.get_new_clients()` — sync `def` in Python.
    pub fn get_new_clients(&mut self) -> Result<Vec<Arc<Mutex<Client>>>, WizError> {
        let mut new_clients = Vec::new();

        // Match by window CLASS name, not title — exactly like Python's
        // get_all_wizard_handles() which uses GetClassNameW + "Wizard Graphical Client".
        // This correctly excludes the launcher/patcher window.
        unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            const TARGET_CLASS: &str = "Wizard Graphical Client";
            let mut class_name: [u16; 64] = [0; 64];
            let len = unsafe { GetClassNameW(hwnd, &mut class_name) };
            if len > 0 {
                let class_str = String::from_utf16_lossy(&class_name[..len as usize]);
                if class_str == TARGET_CLASS {
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
                    }.map_err(|e| WizError::Other(format!("OpenProcess failed: {}", e)))?;

                    let mut client = Client::from_handles(hwnd, process_handle, process_id);

                    // Initialize the memory reader and memory objects.
                    // If open() fails (e.g., process died between EnumWindows and here),
                    // we still track the client — it will be cleaned up by remove_dead_clients().
                    if let Err(e) = client.open() {
                        eprintln!("Warning: Failed to open client (pid {}): {}", process_id, e);
                    }

                    let client_arc = Arc::new(Mutex::new(client));
                    self.clients.push(client_arc.clone());
                    new_clients.push(client_arc);
                }
            }
        }

        Ok(new_clients)
    }

    /// Remove and return clients that are no longer running.
    ///
    /// # Python equivalent
    /// `ClientHandler.remove_dead_clients()` — sync `def` in Python.
    pub fn remove_dead_clients(&mut self) -> Vec<Arc<Mutex<Client>>> {
        let mut dead_clients = Vec::new();
        let mut alive_clients = Vec::new();

        for client_arc in self.clients.drain(..) {
            let is_running = client_arc.blocking_lock().is_running();
            if is_running {
                alive_clients.push(client_arc);
            } else {
                dead_clients.push(client_arc);
            }
        }

        self.clients = alive_clients;
        // Rebuild managed handles from alive clients.
        self.managed_handles.clear();
        for client in &self.clients {
            self.managed_handles.push(client.blocking_lock().window_handle);
        }

        dead_clients
    }

    /// Get clients ordered by their screen position.
    ///
    /// # Python equivalent
    /// `ClientHandler.get_ordered_clients()` — sync `def` in Python.
    pub fn get_ordered_clients(&self) -> Vec<Arc<Mutex<Client>>> {
        // TODO: order by screen position (utils.order_clients)
        self.clients.clone()
    }

    /// Activate hooks for all managed clients.
    ///
    /// # Python equivalent
    /// `ClientHandler.activate_all_client_hooks()` — async in Python
    /// (used asyncio.create_task for parallelism), but the underlying
    /// HookHandler.activate_all_hooks() is sync in Rust (WriteProcessMemory).
    pub fn activate_all_client_hooks(&self) {
        for client in &self.clients {
            let mut c = client.blocking_lock();
            if let Err(e) = c.activate_hooks() {
                eprintln!("Warning: Failed to activate hooks for pid {}: {}", c.process_id, e);
            }
        }
    }

    /// Activate mouseless cursor hook for all managed clients.
    ///
    /// # Python equivalent
    /// `ClientHandler.activate_all_client_mouseless()` — async in Python,
    /// sync in Rust (WriteProcessMemory).
    pub fn activate_all_client_mouseless(&self) {
        for client in &self.clients {
            let mut c = client.blocking_lock();
            // TODO: c.activate_mouseless() when implemented
            let _ = &mut c; // suppress unused warning
        }
    }

    /// Close all clients — unhook, release handles.
    ///
    /// # Python equivalent
    /// `ClientHandler.close()` — async in Python, sync in Rust.
    pub fn close(&mut self) {
        for client in &self.clients {
            client.blocking_lock().close();
        }
        self.clients.clear();
        self.managed_handles.clear();
    }
}

impl Drop for ClientHandler {
    fn drop(&mut self) {
        self.close();
    }
}
