use std::ffi::CStr;
use std::{i32, mem, ptr::null_mut, str};

use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};
use winapi::um::winnt::{PROCESS_ALL_ACCESS, PROCESS_VM_READ, PROCESS_VM_WRITE};
use winapi::{
    ctypes::{c_long, c_void},
    shared::ntdef::HANDLE,
    shared::{
        basetsd::SIZE_T,
        minwindef::{BOOL, DWORD, FALSE, HMODULE, LPCVOID, LPVOID, TRUE},
    },
    um::{
        handleapi::INVALID_HANDLE_VALUE,
        libloaderapi::{GetModuleHandleA, GetProcAddress},
        memoryapi::ReadProcessMemory,
        tlhelp32::{Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS},
        winnt::{PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS},
    },
};

pub fn get_module_handle(name: *const u8) -> HMODULE {
    unsafe { return GetModuleHandleA(name as _) }
}

pub fn get_module_base_adress(pid: DWORD, module_name: &str) -> Result<usize, String> {
    let base_adress: usize = 0;
    let mut module_entry = MODULEENTRY32::default();
    module_entry.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;
    let handle_snap: HANDLE =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
    if handle_snap == INVALID_HANDLE_VALUE {
        return Err(String::from("Could not get module base address"));
    }
    let module = unsafe { Module32First(handle_snap, &mut module_entry) };
    if module != 0 {
        loop {
            let mut temp = Vec::<u8>::new();

            let result = unsafe { CStr::from_ptr(module_entry.szModule.as_ptr()) }.to_owned();
            if let Ok(entry_name) = result.to_str() {
                if entry_name == module_name {
                    unsafe { CloseHandle(handle_snap) };
                    return Ok(module_entry.modBaseAddr as usize);
                }
            }
            let sucess = unsafe { Module32Next(handle_snap, &mut module_entry) };
            if sucess == 0 {
                return Err(String::from("Could not find module"));
            }
        }
    }
    Err(String::from("Could not find module"))
}

pub fn get_process_handle(pid: DWORD) -> HANDLE {
    unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, pid) }
}

pub fn pattern_to_bytes(pattern: String) -> Vec<i32> {
    pattern
        .replace(' ', "")
        .as_bytes()
        .chunks(2)
        .map(str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .into_iter()
        .filter(|&q| !q.contains('?'))
        .map(|q| i32::from_str_radix(q, 16).unwrap())
        .collect::<Vec<i32>>()
}

pub fn pattern_scan(module: HMODULE, sig: &str) -> *mut u8 {
    unsafe {
        let dos_headers = module as PIMAGE_DOS_HEADER;

        let module_addr = module as usize;
        let e_lfanew = (*dos_headers).e_lfanew as c_long;

        let nt_headers = (module_addr + e_lfanew as usize) as PIMAGE_NT_HEADERS;

        let size_of_image = (*nt_headers).OptionalHeader.SizeOfImage as usize;
        let pattern_bytes = pattern_to_bytes(sig.to_owned());
        let bytes = module as *mut u8;

        let size = pattern_bytes.len();

        for i in 0..(size_of_image - size as usize) {
            let mut found = true;
            for j in 0..size {
                if *bytes.offset(i as isize + j as isize) != pattern_bytes[j] as _
                    && pattern_bytes[j] != -1
                {
                    found = false;
                    break;
                }
            }

            if found {
                return bytes.offset(i as _) as _;
            }
        }
    }

    0 as *mut _
}

pub fn get_proc_id(exe_name: &str) -> DWORD {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            panic!("Failed to create snapshot");
        }
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; 260],
        };
        let mut proc_id: DWORD = 0;
        while Process32Next(snapshot, &mut entry) != 0 {
            if exe_name
                == std::ffi::CStr::from_ptr(entry.szExeFile.as_ptr())
                    .to_str()
                    .unwrap()
            {
                proc_id = entry.th32ProcessID;
                break;
            }
        }
        return proc_id;
    }
}

pub fn get_process_adress(module: HMODULE, proc_name: *const u8) -> *mut c_void {
    unsafe { GetProcAddress(module, proc_name as _) as _ }
}

pub fn find_dma_addy(handle: HANDLE, ptr: u32, ptrs: &[u32]) -> Result<u32, String> {
    let mut addr = ptr;
    for offset in ptrs {
        println!("Reading address: {:#X}", addr);
        let value = read_ptr(handle, addr as usize)?;
        addr = value;
        addr += offset;
    }
    Ok(addr)
}

pub fn read_bytes<const N: usize>(
    handle: HANDLE,
    address: usize,
    byte: usize,
) -> Result<[u8; N], String> {
    unsafe {
        let mut buffer: Vec<u8> = vec![0; byte];
        let success: BOOL = ReadProcessMemory(
            handle,
            address as LPCVOID,
            buffer.as_mut_ptr() as LPVOID,
            byte,
            null_mut(),
        );
        match success {
            FALSE => Err(get_last_error_message()),
            _ => Ok(buffer.try_into().unwrap_or_else(|v: Vec<u8>| {
                panic!("Expected a Vec of length {} but it was {}", N, v.len())
            })),
        }
    }
}

pub fn read_int(handle: HANDLE, address: usize) -> Result<i32, String> {
    let bytes = read_bytes(handle, address, mem::size_of::<i32>())?;
    Ok(i32::from_le_bytes(bytes))
}

pub fn read_float(handle: HANDLE, address: usize) -> Result<f32, String> {
    let bytes = read_bytes(handle, address, mem::size_of::<f32>())?;
    Ok(f32::from_le_bytes(bytes))
}

pub fn read_ulong(handle: HANDLE, address: usize) -> Result<u32, String> {
    let bytes = read_bytes(handle, address, mem::size_of::<u32>())?;
    Ok(u32::from_le_bytes(bytes))
}

pub fn read_ptr(handle: HANDLE, adress: usize) -> Result<u32, String> {
    let bytes: [u8; 4] = read_bytes(handle, adress, mem::size_of::<u32>())?;
    Ok(u32::from_le_bytes(bytes))
}

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FormatMessageA;
use winapi::um::winnt::{LANG_NEUTRAL, LPSTR, MAKELANGID, SUBLANG_DEFAULT};

pub fn get_last_error_message() -> String {
    unsafe {
        let error_code = GetLastError();

        let mut message_buffer: LPSTR = std::mem::zeroed();
        let buffer_size = FormatMessageA(
            0x00001000 | 0x00000100, // FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM
            null_mut(),
            error_code,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
            &mut message_buffer as *mut _ as LPSTR,
            0,
            null_mut(),
        );

        let message = CStr::from_ptr(message_buffer)
            .to_string_lossy()
            .into_owned();
        message
    }
}
