use crate::errors::{Result, WizWalkerError};
use crate::memory::reader::{MemoryReader, MemoryReaderExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct InstanceFinder {
    reader: Arc<dyn MemoryReader>,
    class_name: String,

    all_jmp_instructions: Mutex<Option<Vec<usize>>>,
    all_type_name_functions: Mutex<Option<Vec<usize>>>,
    type_name_function_map: Mutex<Option<HashMap<String, Vec<usize>>>>,
    jmp_functions: Mutex<Option<Vec<usize>>>,
}

impl InstanceFinder {
    pub const GET_TYPE_NAME_PATTERN: &'static [u8] = b"\x48\x89\x5C\x24\x10\x57\x48\x83\xEC\x20\xE8....\xBF\x02\x00\x00\x00\x48\x8B\xD8\x8B\xC7\xF0\x0F\xB1\x3D....\x74\x54\x48\x89\x74\x24\x30\xBE\x01\x00\x00\x00\x0F\x1F\x00\x33\xC0";
    pub const EXE_NAME: &'static str = "WizardGraphicalClient.exe";

    pub fn new(reader: Arc<dyn MemoryReader>, class_name: String) -> Self {
        Self {
            reader,
            class_name,
            all_jmp_instructions: Mutex::new(None),
            all_type_name_functions: Mutex::new(None),
            type_name_function_map: Mutex::new(None),
            jmp_functions: Mutex::new(None),
        }
    }

    pub async fn read_null_terminated_string(&self, address: usize, max_size: usize) -> Result<String> {
        let bytes = self.reader.read_bytes(address, max_size)?;
        if let Some(end) = bytes.iter().position(|&b| b == 0) {
            if end == 0 {
                return Ok(String::new());
            }
            Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
        } else {
            Err(WizWalkerError::Other("Null terminator not found".into()))
        }
    }

    pub async fn scan_for_pointer(&self, address: usize) -> Result<Vec<usize>> {
        let pattern = address.to_le_bytes();
        self.reader.pattern_scan(&pattern, None, true)
    }

    pub async fn get_all_jmp_instructions(&self) -> Result<Vec<usize>> {
        let mut cache = self.all_jmp_instructions.lock().await;
        if let Some(jmps) = &*cache {
            return Ok(jmps.clone());
        }

        let jmps = self.reader.pattern_scan(b"\xE9", Some(Self::EXE_NAME), true)?;
        *cache = Some(jmps.clone());
        Ok(jmps)
    }

    pub async fn get_all_type_name_functions(&self) -> Result<Vec<usize>> {
        let mut cache = self.all_type_name_functions.lock().await;
        if let Some(funcs) = &*cache {
            return Ok(funcs.clone());
        }

        let funcs = self.reader.pattern_scan(Self::GET_TYPE_NAME_PATTERN, Some(Self::EXE_NAME), true)?;
        *cache = Some(funcs.clone());
        Ok(funcs)
    }

    pub async fn get_type_name_function_map(&self) -> Result<HashMap<String, Vec<usize>>> {
        let mut cache = self.type_name_function_map.lock().await;
        if let Some(map) = &*cache {
            return Ok(map.clone());
        }

        let mut func_name_map: HashMap<String, Vec<usize>> = HashMap::new();
        let funcs = self.get_all_type_name_functions().await?;

        for func in funcs {
            let lea_instruction = func + 63;
            let lea_target = func + 66;
            let rip_offset: i32 = self.reader.read_typed(lea_target)?;

            let type_name_addr = (lea_instruction as isize + rip_offset as isize + 7) as usize;
            let type_name = self.read_null_terminated_string(type_name_addr, 60).await?;

            func_name_map.entry(type_name).or_insert_with(Vec::new).push(func);
        }

        *cache = Some(func_name_map.clone());
        Ok(func_name_map)
    }

    pub async fn get_type_name_functions(&self) -> Result<Vec<usize>> {
        let function_map = self.get_type_name_function_map().await?;
        Ok(function_map.get(&self.class_name).cloned().unwrap_or_default())
    }

    pub async fn get_jmp_functions(&self) -> Result<Vec<usize>> {
        let mut cache = self.jmp_functions.lock().await;
        if let Some(jmps) = &*cache {
            return Ok(jmps.clone());
        }

        let all_jmps = self.get_all_jmp_instructions().await?;
        let type_name_funcs = self.get_type_name_functions().await?;

        let mut jmp_funcs = Vec::new();
        for jmp in all_jmps {
            if jmp_funcs.len() == type_name_funcs.len() {
                break;
            }

            let offset: i32 = self.reader.read_typed(jmp + 1)?;

            for &poss in &type_name_funcs {
                if (offset as isize + 5) == (poss as isize - jmp as isize) {
                    jmp_funcs.push(jmp);
                }
            }
        }

        *cache = Some(jmp_funcs.clone());
        Ok(jmp_funcs)
    }

    pub async fn get_instances(&self) -> Result<Vec<usize>> {
        let mut instances = Vec::new();

        for jmp_function in self.get_jmp_functions().await? {
            let vtable_function_pointers = self.scan_for_pointer(jmp_function).await?;
            for vtable_function in vtable_function_pointers {
                let mut vtable_pointers = self.scan_for_pointer(vtable_function).await?;
                instances.append(&mut vtable_pointers);
            }
        }

        for type_name_function in self.get_type_name_functions().await? {
            let vtable_function_pointers = self.scan_for_pointer(type_name_function).await?;
            for vtable_function in vtable_function_pointers {
                let mut vtable_pointers = self.scan_for_pointer(vtable_function).await?;
                instances.append(&mut vtable_pointers);
            }
        }

        Ok(instances)
    }
}
