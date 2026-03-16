use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct NifMap {
    pub header_string: String,
    pub format_version: String,
    pub is_little_endian: bool,
    pub user_version: u32,
    pub block_number: u32,
    pub block_type_number: u16,
    pub types: Vec<String>,
    pub type_indexes: Vec<String>,
    pub size_map: Vec<u32>,
    pub string_num: u32,
    pub max_string_length: u32,
    pub strings: Vec<String>,
    pub group_num: u32,
    pub header_end_pos: u64,
}

impl NifMap {
    pub fn new(data: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(data.to_vec());
        Self::read_header(&mut cursor)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::new(&data)
    }

    fn read_sized_string(cursor: &mut Cursor<Vec<u8>>) -> io::Result<String> {
        let length = cursor.read_u32::<LittleEndian>()?;
        let mut value_bytes = vec![0u8; length as usize];
        cursor.read_exact(&mut value_bytes)?;
        let value = String::from_utf8(value_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(value)
    }

    fn read_header(cursor: &mut Cursor<Vec<u8>>) -> io::Result<Self> {
        // Read first 100 bytes to find header string (ends with \n)
        let mut tmp_find_bytes = vec![0u8; 100];
        let bytes_read = cursor.read(&mut tmp_find_bytes)?;
        cursor.seek(SeekFrom::Start(0))?;

        let end_header_string_pos = tmp_find_bytes[..bytes_read]
            .iter()
            .position(|&b| b == 0x0A)
            .map(|pos| pos + 1)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Header string newline not found")
            })?;

        let mut header_bytes = vec![0u8; end_header_string_pos];
        cursor.read_exact(&mut header_bytes)?;

        // Remove trailing newline '\n' (which is included in end_header_string_pos)
        // Also remove any carriage return '\r' if present.
        let header_string = String::from_utf8(header_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
            .trim_end()
            .to_string();

        let format_version = header_string
            .split_whitespace()
            .last()
            .unwrap_or("")
            .to_string();

        // skip 4 bytes
        cursor.seek(SeekFrom::Current(4))?;

        let is_little_endian = cursor.read_u8()? != 0;
        let user_version = cursor.read_u32::<LittleEndian>()?;
        let block_number = cursor.read_u32::<LittleEndian>()?;
        let block_type_number = cursor.read_u16::<LittleEndian>()?;

        let mut types = Vec::new();
        for _ in 0..block_type_number {
            types.push(Self::read_sized_string(cursor)?);
        }

        let mut type_indexes = Vec::new();
        for _ in 0..block_number {
            let type_num = cursor.read_i16::<LittleEndian>()?;
            let type_str = types
                .get(type_num as usize)
                .cloned()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("Invalid type index {}", type_num))
                })?;
            type_indexes.push(type_str);
        }

        let mut size_map = Vec::new();
        for _ in 0..block_number {
            size_map.push(cursor.read_u32::<LittleEndian>()?);
        }

        let string_num = cursor.read_u32::<LittleEndian>()?;
        let max_string_length = cursor.read_u32::<LittleEndian>()?;

        let mut strings = Vec::new();
        for _ in 0..string_num {
            strings.push(Self::read_sized_string(cursor)?);
        }

        let group_num = cursor.read_u32::<LittleEndian>()?;

        if group_num != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Group num is not zero; please report error",
            ));
        }

        let header_end_pos = cursor.position();

        Ok(Self {
            header_string,
            format_version,
            is_little_endian,
            user_version,
            block_number,
            block_type_number,
            types,
            type_indexes,
            size_map,
            string_num,
            max_string_length,
            strings,
            group_num,
            header_end_pos,
        })
    }
}
