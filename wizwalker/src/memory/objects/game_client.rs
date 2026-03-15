use crate::errors::Result;
use crate::memory::memory_object::MemoryReader;
use crate::memory::objects::client_object::ClientObject;
use crate::memory::objects::core_object::CoreObject;
use crate::memory::objects::enums::AccountPermissions;
use std::sync::Arc;

pub struct GameClient<R: MemoryReader + 'static> {
    pub reader: Arc<R>,
    pub base_address: u64,
}

impl<R: MemoryReader + 'static> GameClient<R> {
    pub fn new(reader: Arc<R>, base_address: u64) -> Self {
        Self { reader, base_address }
    }

    pub fn read_base_address(&self) -> Result<u64> {
        Ok(self.base_address)
    }

    fn read_value_from_offset<T: Copy + Default>(&self, offset: u64) -> Result<T> {
        self.reader.read_typed::<T>((self.base_address + offset) as usize)
    }

    fn write_value_to_offset<T: Copy>(&self, offset: u64, value: &T) -> Result<()> {
        self.reader.write_typed::<T>((self.base_address + offset) as usize, value)
    }

    async fn pattern_scan_offset_cached(&self, pattern: &[u8], offset_idx: usize, default: u64) -> Result<u64> {
        let addrs = self.reader.pattern_scan(pattern, Some("WizardGraphicalClient.exe"), true)?;
        if !addrs.is_empty() {
            let addr = addrs[0];
            let offset_bytes = self.reader.read_bytes(addr + offset_idx, 4)?;
            let mut offset_array = [0u8; 4];
            offset_array.copy_from_slice(&offset_bytes[..4]);
            let offset = i32::from_ne_bytes(offset_array);
            Ok((addr as i64 + offset_idx as i64 + 4 + offset as i64) as u64 - self.base_address)
        } else {
            Ok(default)
        }
    }

    pub async fn elastic_camera_controller(&self) -> Result<Option<u64>> {
        let offset = self.pattern_scan_offset_cached(
            b"\x48\x8B\x93\xD8\x1F\x02\x00\x41\xFF\xD1\x32\xC0\xEB\x05\x41\xFF\xD1\xB0\x01\x88\x83\x20\x20\x02\x00\x48\x8B\x07\x33\xD2\x48\x8B\xCF",
            3,
            0x22260
        ).await?;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr))
    }

    pub async fn free_camera_controller(&self) -> Result<Option<u64>> {
        let offset = self.pattern_scan_offset_cached(
            b"\x48\x8B\x93\x00\x00\x00\x00\x48\x8B\x03\x4C\x8B\x88\x00\x00\x00\x00\x41\xB8\x01\x00\x00\x00\x48\x8B\xCB\x48\x3B\xFA\x75",
            3,
            0x22270
        ).await?;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr))
    }

    pub async fn selected_camera_controller(&self) -> Result<Option<u64>> {
        let offset = self.pattern_scan_offset_cached(
            b"\x48\x89\x87\x08\x20\x02\x00\x48\x8D\x8F\x10\x20\x02\x00\x48\x8D\x54\x24\x40\xE8\x00\x00\x00\x00\x90\x48\x8B\x4C\x24",
            3,
            0x22290
        ).await?;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr))
    }

    pub async fn write_selected_camera_controller(&self, selected_camera_controller: u64) -> Result<()> {
        let offset = self.pattern_scan_offset_cached(
            b"\x48\x89\x87\x08\x20\x02\x00\x48\x8D\x8F\x10\x20\x02\x00\x48\x8D\x54\x24\x40\xE8\x00\x00\x00\x00\x90\x48\x8B\x4C\x24",
            3,
            0x22290
        ).await?;
        self.write_value_to_offset::<u64>(offset, &selected_camera_controller)
    }

    pub async fn is_freecam(&self) -> Result<bool> {
        let offset = self.pattern_scan_offset_cached(
            b"\x0F\xB6\x88\x20\x20\x02\x00\x88\x8B\x6A\x02\x00\x00\x84\xC9\x0F\x85\x00\x00\x00\x00\x48\x8D\x55\xE0\x48\x8B\xCB\xE8",
            3,
            0x222A8
        ).await?;
        self.read_value_from_offset::<bool>(offset)
    }

    pub async fn write_is_freecam(&self, is_freecam: bool) -> Result<()> {
        let offset = self.pattern_scan_offset_cached(
            b"\x0F\xB6\x88\x20\x20\x02\x00\x88\x8B\x6A\x02\x00\x00\x84\xC9\x0F\x85\x00\x00\x00\x00\x48\x8D\x55\xE0\x48\x8B\xCB\xE8",
            3,
            0x222A8
        ).await?;
        self.write_value_to_offset::<bool>(offset, &is_freecam)
    }

    pub async fn root_client_object(&self) -> Result<Option<ClientObject<R>>> {
        let offset = self.pattern_scan_offset_cached(
            b"\x48\x8D\x93\xA0\x12\x02\x00\xFF\x90\xB8\x01\x00\x00\x90\x48\x8B\x7C\x24\x30\x48\x85\xFF\x74\x2E\xBE\xFF\xFF\xFF\xFF\x8B\xC6\xF0\x0F\xC1\x47\x08",
            3,
            0x21318
        ).await?;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(ClientObject::new(Arc::clone(&self.reader), addr)))
    }

    pub async fn frames_per_second(&self) -> Result<f32> {
        let offset = self.pattern_scan_offset_cached(
            b"\xF3\x0F\x11\x8B\xFC\x19\x02\x00\xC7\x05\x00\x00\x00\x00\x00\x00\x00\x00\xF2\x0F\x11\x00\x00\x00\x00\x00\x48\x8B\x8B\x00\x13\x02\x00\x48\x85\xC9\x74\x09",
            4,
            0x219FC
        ).await?;
        self.read_value_from_offset::<f32>(offset)
    }

    pub async fn shutdown_signal(&self) -> Result<i32> {
        let offset = self.pattern_scan_offset_cached(
            b"\x38\x9F\xB8\x11\x02\x00\x74\xBE\xE8\x00\x00\x00\x00\x83\xF8\x64\x0F\x8F\x00\x00\x00\x00\xB9\x0F\x00\x00\x00",
            2,
            0x211B8
        ).await?;
        self.read_value_from_offset::<i32>(offset)
    }

    pub async fn write_shutdown_signal(&self, shutdown_signal: i32) -> Result<()> {
        let offset = self.pattern_scan_offset_cached(
            b"\x38\x9F\xB8\x11\x02\x00\x74\xBE\xE8\x00\x00\x00\x00\x83\xF8\x64\x0F\x8F\x00\x00\x00\x00\xB9\x0F\x00\x00\x00",
            2,
            0x211B8
        ).await?;
        self.write_value_to_offset::<i32>(offset, &shutdown_signal)
    }

    pub async fn character_registry(&self) -> Result<Option<u64>> {
        let offset = 0x224A8;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr))
    }

    pub async fn account_permissions(&self) -> Result<AccountPermissions> {
        let offset = self.pattern_scan_offset_cached(
            b"\x41\x89\x86\x3C\x1D\x02\x00\x4D\x8B\x06\x8B\xD0\x49\x8B\xCE\x41\xFF\x90\x10\x04\x00\x00\x49\x8B\x06\x49\x8B\xCE\xFF\x90\x58\x01\x00\x00",
            3,
            0x21D3C
        ).await?;
        let value = self.read_value_from_offset::<i32>(offset)?;
        Ok(AccountPermissions::from_bits_truncate(value))
    }

    pub async fn write_account_permissions(&self, account_permissions: AccountPermissions) -> Result<()> {
        let offset = self.pattern_scan_offset_cached(
            b"\x41\x89\x86\x3C\x1D\x02\x00\x4D\x8B\x06\x8B\xD0\x49\x8B\xCE\x41\xFF\x90\x10\x04\x00\x00\x49\x8B\x06\x49\x8B\xCE\xFF\x90\x58\x01\x00\x00",
            3,
            0x21D3C
        ).await?;
        self.write_value_to_offset::<i32>(offset, &account_permissions.bits())
    }

    pub async fn has_membership(&self) -> Result<bool> {
        let offset = self.pattern_scan_offset_cached(
            b"\x83\xBB\x40\x1D\x02\x00\x00\x75\x04\xB2\x01\xEB\x02\x33\xD2\x48\x8B\x00\x00\x00\x00\x00\xE8",
            2,
            0x21D40
        ).await?;
        self.read_value_from_offset::<bool>(offset)
    }

    pub async fn write_has_membership(&self, has_membership: bool) -> Result<()> {
        let offset = self.pattern_scan_offset_cached(
            b"\x83\xBB\x40\x1D\x02\x00\x00\x75\x04\xB2\x01\xEB\x02\x33\xD2\x48\x8B\x00\x00\x00\x00\x00\xE8",
            2,
            0x21D40
        ).await?;
        self.write_value_to_offset::<bool>(offset, &has_membership)
    }

    pub async fn gamebryo_presenter(&self) -> Result<Option<u64>> {
        let offset = self.pattern_scan_offset_cached(
            b"\x00\x00\x00\x00\x00\x00\x00\x48\x8B\x01\xFF\x50\x40\x84\xC0\x75\x00\xE8",
            3,
            0x21FB8
        ).await?;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr))
    }

    pub async fn fishing_manager(&self) -> Result<Option<u64>> {
        let offset = 0x23140;
        let addr = self.read_value_from_offset::<u64>(offset)?;
        if addr == 0 {
            return Ok(None);
        }
        Ok(Some(addr))
    }
}

pub struct CurrentGameClient<R: MemoryReader + 'static> {
    pub reader: Arc<R>,
    base_address_cache: std::sync::Mutex<Option<u64>>,
}

impl<R: MemoryReader + 'static> CurrentGameClient<R> {
    pub fn new(reader: Arc<R>) -> Result<Self> {
        Ok(Self {
            reader,
            base_address_cache: std::sync::Mutex::new(None),
        })
    }

    pub fn read_base_address(&self) -> Result<u64> {
        let mut cache = self.base_address_cache.lock().unwrap();
        if let Some(addr) = *cache {
            return Ok(addr);
        }

        let reader = Arc::clone(&self.reader);
        let addrs = reader.pattern_scan(
            b"\x48\x8b\x00\x00\x00\x00\x00\x48\x8b\x00\x80\xb8\x00\x00\x00\x00\x00\x74\x00\x4c\x8b",
            Some("WizardGraphicalClient.exe"),
            true
        )?;

        if addrs.is_empty() {
            return Err(crate::errors::WizWalkerError::Other("Could not find CurrentGameClient pattern".into()));
        }

        let addr = addrs[0];
        let offset_bytes = reader.read_bytes(addr + 3, 4)?;
        let mut offset_array = [0u8; 4];
        offset_array.copy_from_slice(&offset_bytes[..4]);
        let offset = i32::from_ne_bytes(offset_array);

        let base_address_bytes = reader.read_bytes((addr as i64 + 7 + offset as i64) as usize, 8)?;
        let mut base_address_array = [0u8; 8];
        base_address_array.copy_from_slice(&base_address_bytes[..8]);
        let base_address = u64::from_ne_bytes(base_address_array);

        *cache = Some(base_address);
        Ok(base_address)
    }

    pub async fn get_client(&self) -> Result<GameClient<R>> {
        let base_address = self.read_base_address()?;
        Ok(GameClient::new(Arc::clone(&self.reader), base_address))
    }
}
