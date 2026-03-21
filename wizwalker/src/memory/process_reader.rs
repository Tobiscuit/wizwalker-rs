//! Concrete `MemoryReader` implementation that reads/writes another process's memory
//! via the Win32 API (`ReadProcessMemory`, `WriteProcessMemory`, etc.).
//!
//! This is the foundation that every memory object depends on.
//!
//! # Python equivalent
//! `wizwalker/memory/memory_reader.py` — `MemoryReader` class.
//! Python uses `pymem` which wraps these same Win32 calls, but runs them in
//! `run_in_executor` because of the GIL. In Rust, these are direct ~1μs
//! kernel calls — no executor needed.

use std::ffi::c_void;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, VirtualQueryEx,
    MEMORY_BASIC_INFORMATION,
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE,
    PAGE_PROTECTION_FLAGS,
};
use windows::Win32::System::Threading::GetExitCodeProcess;

use crate::errors::{Result, WizWalkerError};
use super::reader::MemoryReader;

/// A `MemoryReader` backed by a real Windows process handle.
///
/// Created from a `HANDLE` obtained via `OpenProcess` with at least
/// `PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_QUERY_INFORMATION`.
///
/// # Thread Safety
/// `HANDLE` is a raw kernel handle — it is `Send + Sync` safe because
/// `ReadProcessMemory`/`WriteProcessMemory` are thread-safe Win32 calls.
pub struct ProcessMemoryReader {
    process_handle: HANDLE,
}

// SAFETY: Windows `HANDLE` values are opaque kernel references.
// `ReadProcessMemory` and `WriteProcessMemory` are safe to call from any thread.
unsafe impl Send for ProcessMemoryReader {}
unsafe impl Sync for ProcessMemoryReader {}

impl ProcessMemoryReader {
    /// Wrap an existing process handle.
    ///
    /// The caller is responsible for ensuring the handle has the correct access rights
    /// and remains valid for the lifetime of this reader.
    pub fn new(process_handle: HANDLE) -> Self {
        Self { process_handle }
    }

    /// Get the underlying Windows `HANDLE`.
    pub fn handle(&self) -> HANDLE {
        self.process_handle
    }

    /// Scan a single memory page for all occurrences of `pattern`.
    ///
    /// Returns `(next_region_address, Vec<found_addresses>)`.
    fn scan_page(&self, address: usize, pattern: &[u8]) -> Result<(usize, Vec<usize>)> {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        let mbi_size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();

        let result = unsafe {
            VirtualQueryEx(
                self.process_handle,
                Some(address as *const c_void),
                &mut mbi,
                mbi_size,
            )
        };

        if result == 0 {
            // Can't query this region — skip to the next 4KB page.
            return Ok((address + 0x1000, Vec::new()));
        }

        let next_region = mbi.BaseAddress as usize + mbi.RegionSize;

        // Only scan committed pages with readable protection flags.
        if mbi.State != MEM_COMMIT {
            return Ok((next_region, Vec::new()));
        }

        let allowed = [
            PAGE_EXECUTE_READ,
            PAGE_EXECUTE_READWRITE,
            PAGE_READWRITE,
            PAGE_READONLY,
        ];

        if !allowed.contains(&PAGE_PROTECTION_FLAGS(mbi.Protect.0)) {
            return Ok((next_region, Vec::new()));
        }

        // Read the entire page into a local buffer.
        let page_bytes = match self.read_bytes(mbi.BaseAddress as usize, mbi.RegionSize) {
            Ok(bytes) => bytes,
            Err(_) => return Ok((next_region, Vec::new())),
        };

        // Pattern search with wildcard support.
        // Python uses `regex.finditer(pattern, page_bytes, regex.DOTALL)` where
        // `b"."` (0x2E) matches any byte. We replicate this: 0x2E in the pattern
        // is treated as a wildcard that matches any byte in the page.
        const WILDCARD: u8 = 0x2E; // b'.' in Python

        let mut found = Vec::new();
        if pattern.len() <= page_bytes.len() {
            'outer: for i in 0..=(page_bytes.len() - pattern.len()) {
                for j in 0..pattern.len() {
                    if pattern[j] != WILDCARD && pattern[j] != page_bytes[i + j] {
                        continue 'outer;
                    }
                }
                found.push(mbi.BaseAddress as usize + i);
            }
        }

        Ok((next_region, found))
    }

    /// Scan the entire process for a byte pattern.
    ///
    /// If `return_multiple` is false, stops after the first match.
    fn scan_all(&self, pattern: &[u8], return_multiple: bool) -> Result<Vec<usize>> {
        let mut address: usize = 0;
        let mut found = Vec::new();

        // Scan up to the user-mode address limit (x64).
        while address < 0x7FFF_FFFF_0000 {
            let (next, page_found) = self.scan_page(address, pattern)?;
            found.extend(page_found);

            if !return_multiple && !found.is_empty() {
                break;
            }

            address = next;
        }

        Ok(found)
    }

    /// Scan within a specific module's address range.
    pub(crate) fn scan_module(
        &self,
        base_address: usize,
        module_size: usize,
        pattern: &[u8],
    ) -> Result<Vec<usize>> {
        let max_address = base_address + module_size;
        let mut address = base_address;
        let mut found = Vec::new();

        while address < max_address {
            let (next, page_found) = self.scan_page(address, pattern)?;
            found.extend(page_found);
            address = next;
        }

        Ok(found)
    }

    /// Find a loaded module by name and return `(base_address, size)`.
    ///
    /// Uses `EnumProcessModulesEx` + `GetModuleBaseNameW` to iterate modules.
    pub fn find_module(&self, module_name: &str) -> Result<(usize, usize)> {
        use windows::Win32::System::ProcessStatus::{
            EnumProcessModulesEx, GetModuleBaseNameW, GetModuleInformation,
            LIST_MODULES_ALL, MODULEINFO,
        };
        use windows::Win32::Foundation::HMODULE;

        let mut modules = [HMODULE::default(); 1024];
        let mut bytes_needed: u32 = 0;

        unsafe {
            EnumProcessModulesEx(
                self.process_handle,
                modules.as_mut_ptr(),
                (modules.len() * std::mem::size_of::<HMODULE>()) as u32,
                &mut bytes_needed,
                LIST_MODULES_ALL,
            )?;
        }

        let module_count = bytes_needed as usize / std::mem::size_of::<HMODULE>();

        for module in &modules[..module_count] {
            let mut name_buf = [0u16; 256];
            let name_len = unsafe {
                GetModuleBaseNameW(self.process_handle, Some(*module), &mut name_buf)
            };

            if name_len == 0 {
                continue;
            }

            let name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
            if name.eq_ignore_ascii_case(module_name) {
                let mut info = MODULEINFO::default();
                unsafe {
                    GetModuleInformation(
                        self.process_handle,
                        *module,
                        &mut info,
                        std::mem::size_of::<MODULEINFO>() as u32,
                    )?;
                }

                return Ok((info.lpBaseOfDll as usize, info.SizeOfImage as usize));
            }
        }

        Err(WizWalkerError::Other(format!(
            "Module '{}' not found in target process",
            module_name
        )))
    }
}

impl MemoryReader for ProcessMemoryReader {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        // Validate address range (mirrors Python's check).
        if address == 0 || address > 0x7FFF_FFFF_FFFF_FFFF {
            return Err(WizWalkerError::AddressOutOfRange(address));
        }

        let mut buffer = vec![0u8; size];
        let mut bytes_read: usize = 0;

        let success = unsafe {
            ReadProcessMemory(
                self.process_handle,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                size,
                Some(&mut bytes_read),
            )
        };

        match success {
            Ok(()) => Ok(buffer),
            Err(_) => {
                // Python: if read fails, check if process is still running.
                if !self.is_running() {
                    Err(WizWalkerError::ClientClosed)
                } else {
                    Err(WizWalkerError::MemoryRead { address })
                }
            }
        }
    }

    fn write_bytes(&self, address: usize, data: &[u8]) -> Result<()> {
        let mut bytes_written: usize = 0;

        let success = unsafe {
            WriteProcessMemory(
                self.process_handle,
                address as *const c_void,
                data.as_ptr() as *const c_void,
                data.len(),
                Some(&mut bytes_written),
            )
        };

        match success {
            Ok(()) => Ok(()),
            Err(_) => {
                if !self.is_running() {
                    Err(WizWalkerError::ClientClosed)
                } else {
                    Err(WizWalkerError::MemoryWrite { address })
                }
            }
        }
    }

    fn pattern_scan(
        &self,
        pattern: &[u8],
        module: Option<&str>,
        return_multiple: bool,
    ) -> Result<Vec<usize>> {
        let found = if let Some(module_name) = module {
            let (base, size) = self.find_module(module_name)?;
            self.scan_module(base, size, pattern)?
        } else {
            self.scan_all(pattern, return_multiple)?
        };

        if found.is_empty() {
            return Err(WizWalkerError::PatternNotFound(
                format!("{:02X?}", pattern),
            ));
        }

        if !return_multiple && found.len() > 1 {
            return Err(WizWalkerError::Other(format!(
                "Pattern scan returned {} results (expected 1)",
                found.len()
            )));
        }

        Ok(found)
    }

    fn allocate(&self, size: usize) -> Result<usize> {
        let addr = unsafe {
            VirtualAllocEx(
                self.process_handle,
                None,
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };

        if addr.is_null() {
            Err(WizWalkerError::Other(format!(
                "VirtualAllocEx failed for {} bytes",
                size
            )))
        } else {
            Ok(addr as usize)
        }
    }

    fn free(&self, address: usize) -> Result<()> {
        let success = unsafe {
            VirtualFreeEx(
                self.process_handle,
                address as *mut c_void,
                0,
                MEM_RELEASE,
            )
        };

        match success {
            Ok(()) => Ok(()),
            Err(e) => Err(WizWalkerError::Other(format!(
                "VirtualFreeEx failed at {:#X}: {}",
                address, e
            ))),
        }
    }

    fn is_running(&self) -> bool {
        let mut exit_code: u32 = 0;
        let result = unsafe { GetExitCodeProcess(self.process_handle, &mut exit_code) };

        // STILL_ACTIVE = 259
        result.is_ok() && exit_code == 259
    }

    fn process_handle(&self) -> isize {
        self.process_handle.0 as isize
    }
}
