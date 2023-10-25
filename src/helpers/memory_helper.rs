use std::{ffi::CStr, i32, mem, ptr::null_mut, str};

use winapi::{
    ctypes::c_long,
    shared::{
        minwindef::{BOOL, DWORD, FALSE, HMODULE, LPCVOID, LPVOID},
        ntdef::HANDLE,
    },
    um::{
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        libloaderapi::GetModuleHandleA,
        memoryapi::ReadProcessMemory,
        processthreadsapi::OpenProcess,
        tlhelp32::{
            CreateToolhelp32Snapshot, Module32First, Module32Next, Process32Next, MODULEENTRY32,
            PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
        },
        winnt::{PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS, PROCESS_ALL_ACCESS},
    },
};

pub fn get_module_handle(name: *const u8) -> HMODULE {
    unsafe { return GetModuleHandleA(name as _) }
}

pub fn get_module_base_adress(pid: DWORD, module_name: &str) -> Result<usize, String> {
    let mut module_entry = MODULEENTRY32 {
        dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
        ..Default::default()
    };
    let handle_snap: HANDLE =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) };
    if handle_snap == INVALID_HANDLE_VALUE {
        return Err(String::from("Could not get module base address"));
    }
    let module = unsafe { Module32First(handle_snap, &mut module_entry) };
    if module != 0 {
        loop {
            let result = unsafe { CStr::from_ptr(module_entry.szModule.as_ptr()) }.to_owned();
            if let Ok(entry_name) = result.to_str() {
                if entry_name == module_name {
                    unsafe { CloseHandle(handle_snap) };
                    return Ok(module_entry.modBaseAddr as usize);
                }
            }
            let success = unsafe { Module32Next(handle_snap, &mut module_entry) };
            if success == FALSE {
                break;
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

pub fn get_proc_id(exe_name: &str) -> Result<DWORD, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            panic!("Failed to create snapshot");
        }
        let mut entry = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            ..Default::default()
        };
        let mut proc_id: DWORD = 0;
        while Process32Next(snapshot, &mut entry) != FALSE {
            if exe_name
                == std::ffi::CStr::from_ptr(entry.szExeFile.as_ptr())
                    .to_str()
                    .unwrap()
            {
                proc_id = entry.th32ProcessID;
                break;
            }
        }
        match proc_id {
            0 => Err(String::from("Could not find process")),
            _ => Ok(proc_id),
        }
    }
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

use winapi::um::{
    errhandlingapi::GetLastError,
    winbase::{FormatMessageA, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM},
    winnt::{LANG_NEUTRAL, LPSTR, MAKELANGID, SUBLANG_DEFAULT},
};

pub fn get_last_error_message() -> String {
    unsafe {
        let error_code = GetLastError();

        let mut message_buffer: LPSTR = std::mem::zeroed();
        FormatMessageA(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
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
