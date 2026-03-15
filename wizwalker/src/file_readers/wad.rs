use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WadFileInfo {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub is_zip: bool,
    pub crc: u32,
    pub unzipped_size: u32,
}

#[derive(Debug)]
pub struct Wad {
    pub name: String,
    pub file_path: PathBuf,
    file_list: Vec<WadFileInfo>,
    open: bool,
    refreshed_once: bool,
}

impl Wad {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{:?} not found.", path),
            ));
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            name,
            file_path: path.to_path_buf(),
            file_list: Vec::new(),
            open: false,
            refreshed_once: false,
        })
    }

    pub fn from_game_data(name: &str) -> io::Result<Self> {
        let mut name_with_ext = name.to_string();
        if !name_with_ext.ends_with(".wad") {
            name_with_ext.push_str(".wad");
        }

        // We assume `crate::utils::get_wiz_install` returns PathBuf
        let mut file_path = crate::utils::get_wiz_install()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        file_path.push("Data");
        file_path.push("GameData");
        file_path.push(name_with_ext);

        Self::new(file_path)
    }

    pub fn size(&mut self) -> io::Result<u32> {
        if !self.open {
            self.open_wad()?;
        }
        Ok(self.file_list.iter().map(|f| f.size).sum())
    }

    pub fn names(&mut self) -> io::Result<Vec<String>> {
        if !self.open {
            self.open_wad()?;
        }
        Ok(self.file_list.iter().map(|f| f.name.clone()).collect())
    }

    pub fn open_wad(&mut self) -> io::Result<()> {
        if self.open {
            return Ok(());
        }
        self.refresh_journal()?;
        self.open = true;
        Ok(())
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    fn refresh_journal(&mut self) -> io::Result<()> {
        if self.refreshed_once {
            return Ok(());
        }

        let mut fp = File::open(&self.file_path)?;

        // KIWAD id string (5 bytes)
        fp.seek(SeekFrom::Start(5))?;

        let version = fp.read_u32::<LittleEndian>()?;
        let file_num = fp.read_u32::<LittleEndian>()?;

        if version >= 2 {
            // skip 1 byte
            fp.seek(SeekFrom::Current(1))?;
        }

        for _ in 0..file_num {
            let offset = fp.read_u32::<LittleEndian>()?;
            let size = fp.read_u32::<LittleEndian>()?;
            let zsize = fp.read_u32::<LittleEndian>()?;

            let mut is_zip_byte = [0u8; 1];
            fp.read_exact(&mut is_zip_byte)?;
            let is_zip = is_zip_byte[0] != 0;

            let crc = fp.read_u32::<LittleEndian>()?;
            let name_length = fp.read_u32::<LittleEndian>()?;

            let mut name_bytes = vec![0u8; name_length as usize];
            fp.read_exact(&mut name_bytes)?;

            // Python does: `decode("utf-8")[:-1]` (removing the null terminator)
            let name = String::from_utf8(name_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .trim_end_matches('\0')
                .to_string();

            self.file_list.push(WadFileInfo {
                name,
                offset,
                size,
                is_zip,
                crc,
                unzipped_size: zsize,
            });
        }

        self.refreshed_once = true;
        Ok(())
    }

    pub fn get_file_info(&mut self, name: &str) -> io::Result<WadFileInfo> {
        if !self.open {
            self.open_wad()?;
        }

        self.file_list
            .iter()
            .find(|f| f.name == name)
            .cloned()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("File {} not found.", name))
            })
    }

    pub fn get_file(&mut self, name: &str) -> io::Result<Vec<u8>> {
        let target_file = self.get_file_info(name)?;

        let mut fp = File::open(&self.file_path)?;
        fp.seek(SeekFrom::Start(target_file.offset as u64))?;

        let mut raw_data = vec![0u8; target_file.size as usize];
        fp.read_exact(&mut raw_data)?;

        if target_file.is_zip {
            let mut decoder = ZlibDecoder::new(Cursor::new(raw_data));
            let mut decompressed_data = Vec::new();
            match decoder.read_to_end(&mut decompressed_data) {
                Ok(_) => Ok(decompressed_data),
                Err(_) => {
                    // Fallback to raw data if decompression fails, as in the Python script
                    fp.seek(SeekFrom::Start(target_file.offset as u64))?;
                    let mut fallback_data = vec![0u8; target_file.size as usize];
                    fp.read_exact(&mut fallback_data)?;
                    Ok(fallback_data)
                }
            }
        } else {
            Ok(raw_data)
        }
    }

    pub fn unarchive<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{:?} does not exist.", path),
            ));
        }

        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{:?} is not a directory.", path),
            ));
        }

        // We clone to avoid mutable borrow overlap in the loop
        let file_list = self.file_list.clone();

        for file in file_list {
            let mut target_path = path.to_path_buf();
            target_path.push(&file.name);

            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let data = self.get_file(&file.name)?;
            std::fs::write(target_path, data)?;
        }

        Ok(())
    }

    pub fn from_directory<P: AsRef<Path>>(_path: P) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Not implemented",
        ))
    }
}
