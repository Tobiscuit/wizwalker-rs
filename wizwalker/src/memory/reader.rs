use crate::errors::{Result, WizWalkerError};

pub trait MemoryReader: Send + Sync {
    /// Read some bytes from memory
    fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>>;

    /// Write bytes to memory
    fn write_bytes(&self, address: u64, value: &[u8]) -> Result<()>;

    /// Scan for a pattern
    fn pattern_scan(&self, pattern: &[u8], module: Option<&str>, return_multiple: bool) -> Result<Vec<u64>>;
}

pub trait MemoryReaderExt: MemoryReader {
    /// Read a sized type from memory
    fn read_typed<T>(&self, address: u64) -> Result<T>
    where
        T: Copy + Default,
    {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_bytes(address, size)?;
        if bytes.len() != size {
            return Err(WizWalkerError::Other(format!("Failed to read {} bytes", size)));
        }

        let mut t = T::default();
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), &mut t as *mut T as *mut u8, size);
        }
        Ok(t)
    }

    /// Write a sized type to memory
    fn write_typed<T>(&self, address: u64, value: &T) -> Result<()>
    where
        T: Copy,
    {
        let size = std::mem::size_of::<T>();
        let bytes_ptr = value as *const T as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(bytes_ptr, size) };
        self.write_bytes(address, bytes)
    }
}

impl<R: MemoryReader + ?Sized> MemoryReaderExt for R {}
