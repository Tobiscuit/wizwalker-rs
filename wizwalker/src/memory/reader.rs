use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use std::ffi::c_void;

pub struct MemoryReader {
    pub process_handle: HANDLE,
}

impl MemoryReader {
    pub async fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>, std::io::Error> {
        let handle = self.process_handle;
        tokio::task::spawn_blocking(move || {
            let mut buf = vec![0u8; size];
            let mut read = 0;
            let success = unsafe {
                ReadProcessMemory(
                    handle,
                    address as *const c_void,
                    buf.as_mut_ptr() as *mut c_void,
                    size,
                    Some(&mut read as *mut usize),
                )
            };
            if success.is_ok() {
                buf.truncate(read);
                Ok(buf)
            } else {
                Err(std::io::Error::last_os_error())
            }
        }).await.unwrap()
    }

    pub async fn write_bytes(&self, address: usize, data: Vec<u8>) -> Result<usize, std::io::Error> {
        let handle = self.process_handle;
        tokio::task::spawn_blocking(move || {
            let mut written = 0;
            let success = unsafe {
                WriteProcessMemory(
                    handle,
                    address as *const c_void,
                    data.as_ptr() as *const c_void,
                    data.len(),
                    Some(&mut written as *mut usize),
                )
            };
            if success.is_ok() {
                Ok(written)
            } else {
                Err(std::io::Error::last_os_error())
            }
        }).await.unwrap()
    }

    pub async fn read_typed<T: Copy>(&self, address: usize) -> Result<T, std::io::Error> {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_bytes(address, size).await?;

        if bytes.len() != size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Failed to read the entire typed value",
            ));
        }

        let mut value: std::mem::MaybeUninit<T> = std::mem::MaybeUninit::uninit();
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                value.as_mut_ptr() as *mut u8,
                size,
            );
            Ok(value.assume_init())
        }
    }

    pub async fn write_typed<T: Copy>(&self, address: usize, value: T) -> Result<usize, std::io::Error> {
        let size = std::mem::size_of::<T>();
        let mut bytes = vec![0u8; size];
        unsafe {
            std::ptr::copy_nonoverlapping(
                &value as *const T as *const u8,
                bytes.as_mut_ptr(),
                size,
            );
        }
        self.write_bytes(address, bytes).await
    }

    pub async fn read_null_terminated_string(&self, address: usize, max_len: usize) -> Result<String, std::io::Error> {
        let bytes = self.read_bytes(address, max_len).await?;
        let null_index = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let string_bytes = &bytes[..null_index];
        String::from_utf8(string_bytes.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub async fn read_wide_string(&self, address: usize, max_len: usize) -> Result<String, std::io::Error> {
        let bytes_to_read = max_len * 2;
        let bytes = self.read_bytes(address, bytes_to_read).await?;

        let mut u16_buffer = vec![0u16; max_len];
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                u16_buffer.as_mut_ptr() as *mut u8,
                std::cmp::min(bytes.len(), u16_buffer.len() * 2),
            );
        }

        let null_index = u16_buffer.iter().position(|&w| w == 0).unwrap_or(u16_buffer.len());
        let string_words = &u16_buffer[..null_index];

        String::from_utf16(string_words)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub async fn allocate(&self, size: usize) -> Result<usize, std::io::Error> {
        use windows::Win32::System::Memory::{VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
        let handle = self.process_handle;
        tokio::task::spawn_blocking(move || {
            let addr = unsafe {
                VirtualAllocEx(
                    handle,
                    None,
                    size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_EXECUTE_READWRITE,
                )
            };
            if addr.is_null() {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(addr as usize)
            }
        }).await.unwrap()
    }

    pub async fn free(&self, address: usize) -> Result<(), std::io::Error> {
        use windows::Win32::System::Memory::{VirtualFreeEx, MEM_RELEASE};
        let handle = self.process_handle;
        tokio::task::spawn_blocking(move || {
            let success = unsafe {
                VirtualFreeEx(
                    handle,
                    address as *mut c_void,
                    0,
                    MEM_RELEASE,
                )
            };
            if success.is_ok() {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        }).await.unwrap()
    }

    pub async fn pattern_scan(
        &self,
        pattern: &[u8],
        module: Option<&str>,
    ) -> Result<usize, std::io::Error> {
        let handle = self.process_handle;
        let pattern_vec = pattern.to_vec();
        let module_name = module.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            use windows::Win32::System::ProcessStatus::{EnumProcessModulesEx, GetModuleInformation, MODULEINFO, LIST_MODULES_ALL};
            use windows::Win32::Foundation::HMODULE;
            use windows::Win32::System::ProcessStatus::GetModuleBaseNameA;
            use windows::Win32::System::Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_READWRITE, PAGE_READONLY};

            let mut start_address = 0;
            let mut end_address = 0x7FFFFFFF0000;

            if let Some(mod_name) = module_name {
                let mut modules = [HMODULE::default(); 1024];
                let mut needed = 0;

                let ok = unsafe {
                    EnumProcessModulesEx(
                        handle,
                        modules.as_mut_ptr(),
                        (modules.len() * std::mem::size_of::<HMODULE>()) as u32,
                        &mut needed,
                        LIST_MODULES_ALL,
                    )
                };

                if ok.is_err() {
                    return Err(std::io::Error::last_os_error());
                }

                let num_modules = (needed as usize) / std::mem::size_of::<HMODULE>();
                let mut found_module = false;
                for i in 0..num_modules {
                    let mut name_buf = [0u8; 256];
                    let len = unsafe {
                        GetModuleBaseNameA(
                            handle,
                            modules[i],
                            &mut name_buf,
                        )
                    };

                    if len > 0 {
                        let name = String::from_utf8_lossy(&name_buf[..len as usize]);
                        if name == mod_name {
                            let mut mod_info = MODULEINFO::default();
                            let info_ok = unsafe {
                                GetModuleInformation(
                                    handle,
                                    modules[i],
                                    &mut mod_info,
                                    std::mem::size_of::<MODULEINFO>() as u32,
                                )
                            };
                            if info_ok.is_ok() {
                                start_address = mod_info.lpBaseOfDll as usize;
                                end_address = start_address + mod_info.SizeOfImage as usize;
                                found_module = true;
                                break;
                            }
                        }
                    }
                }
                if !found_module {
                    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Module not found"));
                }
            }

            let mut current_address = start_address;
            while current_address < end_address {
                let mut mbi = MEMORY_BASIC_INFORMATION::default();
                let result = unsafe {
                    VirtualQueryEx(
                        handle,
                        Some(current_address as *const c_void),
                        &mut mbi,
                        std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                    )
                };

                if result == 0 {
                    break;
                }

                let region_size = mbi.RegionSize;
                let next_region = current_address + region_size;

                let allowed_protections = PAGE_EXECUTE_READ.0 | PAGE_EXECUTE_READWRITE.0 | PAGE_READWRITE.0 | PAGE_READONLY.0;

                let is_commit = mbi.State == MEM_COMMIT;
                let is_protected = (mbi.Protect.0 & allowed_protections) != 0;

                if is_commit && is_protected {
                    let mut buf = vec![0u8; region_size];
                    let mut read = 0;
                    unsafe {
                        let _ = ReadProcessMemory(
                            handle,
                            current_address as *const c_void,
                            buf.as_mut_ptr() as *mut c_void,
                            region_size,
                            Some(&mut read as *mut usize),
                        );
                    }

                    if read > 0 {
                        buf.truncate(read);
                        for i in 0..=buf.len().saturating_sub(pattern_vec.len()) {
                            let mut matches = true;
                            for j in 0..pattern_vec.len() {
                                if pattern_vec[j] != 0x2E && pattern_vec[j] != buf[i + j] {
                                    matches = false;
                                    break;
                                }
                            }
                            if matches {
                                return Ok(current_address + i);
                            }
                        }
                    }
                }
                current_address = next_region;
            }

            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Pattern not found"))
        }).await.unwrap()
    }
}
