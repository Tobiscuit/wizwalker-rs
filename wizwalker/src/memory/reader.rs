use crate::errors::{Result, WizWalkerError};

/// Core trait for reading/writing game process memory.
///
/// This is the single canonical definition — all code should depend on this trait.
/// The trait is dyn-compatible (no generic methods) so it can be used as `dyn MemoryReader`.
///
/// Concrete implementations (e.g., `ProcessMemoryReader`) handle the actual
/// interaction with the Windows API (`ReadProcessMemory`, `WriteProcessMemory`, etc.).
pub trait MemoryReader: Send + Sync {
    /// Read `size` bytes from the target process at `address`.
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>>;

    /// Write `data` to the target process at `address`.
    fn write_bytes(&self, address: usize, data: &[u8]) -> Result<()>;

    /// Scan for a byte pattern in the target process memory.
    ///
    /// `pattern` contains the bytes to match, with `0x00` acting as a wildcard
    /// when combined with a separate mask (to be defined by the implementation).
    /// `module` optionally restricts the scan to a specific loaded module.
    /// `return_multiple` controls whether all matches are returned or just the first.
    fn pattern_scan(
        &self,
        pattern: &[u8],
        module: Option<&str>,
        return_multiple: bool,
    ) -> Result<Vec<usize>>;

    /// Allocate `size` bytes in the target process. Returns the base address.
    fn allocate(&self, size: usize) -> Result<usize>;

    /// Free previously allocated memory at `address` in the target process.
    fn free(&self, address: usize) -> Result<()>;

    /// Check whether the target process is still running.
    fn is_running(&self) -> bool;

    /// Get the process handle as a raw `isize` (for passing to Windows API calls).
    fn process_handle(&self) -> isize;

    /// Start a thread in the target process at the given address and wait for it to finish.
    ///
    /// Python: `await self.hook_handler.start_thread(shell_ptr)` — uses CreateRemoteThread
    fn start_thread(&self, address: usize) -> Result<()>;
}

/// Extension trait providing typed read/write helpers.
///
/// Automatically implemented for all `MemoryReader` implementors via blanket impl.
/// These methods use generics internally but delegate to the non-generic
/// `read_bytes`/`write_bytes`, preserving dyn-compatibility.
pub trait MemoryReaderExt: MemoryReader {
    /// Read a sized value of type `T` from the target process at `address`.
    fn read_typed<T>(&self, address: usize) -> Result<T>
    where
        T: Copy + Default,
    {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_bytes(address, size)?;
        if bytes.len() != size {
            return Err(WizWalkerError::MemoryRead { address });
        }
        unsafe { Ok(std::ptr::read_unaligned(bytes.as_ptr() as *const T)) }
    }

    /// Write a sized value of type `T` to the target process at `address`.
    fn write_typed<T>(&self, address: usize, value: &T) -> Result<()>
    where
        T: Copy,
    {
        let size = std::mem::size_of::<T>();
        let bytes =
            unsafe { std::slice::from_raw_parts(value as *const T as *const u8, size) };
        self.write_bytes(address, bytes)
    }
}

/// Blanket implementation: every `MemoryReader` automatically gets `MemoryReaderExt`.
impl<R: MemoryReader + ?Sized> MemoryReaderExt for R {}

/// Delegation: `Arc<dyn MemoryReader>` forwards all calls through the inner trait object.
///
/// This allows code that holds `Arc<dyn MemoryReader>` to call `read_typed`/`write_typed`
/// directly without manual dereferencing.
impl MemoryReader for std::sync::Arc<dyn MemoryReader> {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        (**self).read_bytes(address, size)
    }

    fn write_bytes(&self, address: usize, data: &[u8]) -> Result<()> {
        (**self).write_bytes(address, data)
    }

    fn pattern_scan(
        &self,
        pattern: &[u8],
        module: Option<&str>,
        return_multiple: bool,
    ) -> Result<Vec<usize>> {
        (**self).pattern_scan(pattern, module, return_multiple)
    }

    fn allocate(&self, size: usize) -> Result<usize> {
        (**self).allocate(size)
    }

    fn free(&self, address: usize) -> Result<()> {
        (**self).free(address)
    }

    fn is_running(&self) -> bool {
        (**self).is_running()
    }

    fn process_handle(&self) -> isize {
        (**self).process_handle()
    }

    fn start_thread(&self, address: usize) -> Result<()> {
        (**self).start_thread(address)
    }
}

