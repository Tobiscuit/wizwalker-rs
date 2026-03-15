use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::ProcessStatus::{EnumProcessModulesEx, GetModuleInformation, MODULEINFO, ENUM_PROCESS_MODULES_EX_FLAGS, LIST_MODULES_ALL};
use windows::Win32::System::Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE};
use windows::Win32::Foundation::HANDLE;

fn main() {}
