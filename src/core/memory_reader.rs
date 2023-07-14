use std::ffi::c_void;
use std::mem;
use std::ptr::null_mut;
use std::str;

use winapi::shared::minwindef::{DWORD, HMODULE, LPCVOID, LPVOID};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::HANDLE;

use crate::helpers::memory_helper;

pub struct MemoryReader {
    pid: DWORD,
    handle: HANDLE,
    base_address: c_void,
    u_world_base: usize,
    g_object_base: usize,
    g_name_base: usize,
    g_name_start_address: usize,
}

impl MemoryReader {
    pub fn new(self, exe_name: &str) -> MemoryReader {
        MemoryReader {
            pid: self.get_proc_id(exe_name),
            handle: self.get_process_handle(),
            base_address: 
        }
    }

    pub fn get_proc_id(&self, exe_name: &str) -> DWORD {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
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
        while unsafe { Process32Next(snapshot, &mut entry) } != 0 {
            if exe_name
                == unsafe {
                    std::ffi::CStr::from_ptr(entry.szExeFile.as_ptr())
                        .to_str()
                        .unwrap()
                }
            {
                proc_id = entry.th32ProcessID;
                break;
            }
        }
        proc_id
    }

    fn get_process_handle(&self) -> HANDLE {
        unsafe { OpenProcess(0x0010, 0, self.pid) }
    }
    fn get_base_address() {
            unsafe { GetProcAddress(module, proc_name as _) as _ }
    }
    pub fn check_process_is_active() {}
    fn process_is_active() {}

    pub fn read_bytes<const N: usize>(
        &self,
        address: usize,
        byte: usize,
    ) -> Result<[u8; N], String> {
        unsafe {
            let mut buffer: Vec<u8> = vec![0; byte];
            let success = ReadProcessMemory(
                self.handle,
                address as LPCVOID,
                buffer.as_mut_ptr() as LPVOID,
                byte,
                null_mut(),
            );
            if success == 0 {
                Err(String::from("Failed to read memory"))
            } else {
                Ok(buffer.try_into().unwrap_or_else(|v: Vec<u8>| {
                    panic!("Expected a Vec of length {} but it was {}", N, v.len())
                }))
            }
        }
    }

    pub fn read_int(&self, address: usize) -> Result<i32, String> {
        let bytes = self.read_bytes(address, mem::size_of::<i32>())?;
        Ok(i32::from_le_bytes(bytes))
    }

    pub fn read_float(&self, address: usize) -> Result<f32, String> {
        let bytes = self.read_bytes(address, mem::size_of::<f32>())?;
        Ok(f32::from_le_bytes(bytes))
    }

    pub fn read_ulong(&self, address: usize) -> Result<u32, String> {
        let bytes = self.read_bytes(address, mem::size_of::<u32>())?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn read_ptr(&self, adress: usize) -> Result<usize, String> {
        let bytes = self.read_bytes(adress, mem::size_of::<usize>())?;
        Ok(usize::from_le_bytes(bytes))
    }

    pub fn read_string(&self, address: usize, byte: Option<u32>) -> Result<String, String> {
        let byte = byte.unwrap_or(50) as usize;
        let mut buffer = self.read_bytes(address, byte)?;

        // Find the first null character
        let mut sliced_buffer;
        if let Some(end) = buffer.iter().position(|&x| x == 0) {
            sliced_buffer = Vec::from(&buffer[0..end]);
        }

        // Try to convert to a string
        let mut result =
            String::from_utf8(sliced_buffer).map_err(|_| "Failed to read memory as string")?;

        // Check if the string is possibly UTF-16
        if result.len() == 1 {
            sliced_buffer.push(0); // Add an extra null byte for UTF-16
            let u16s: Vec<u16> = sliced_buffer
                .chunks(2)
                .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                .collect();
            if let Ok(utf16_result) = String::from_utf16(&u16s) {
                result = utf16_result.to_string();
            }
        }

        Ok(result)
    }

    pub fn read_gname(&self, actor_id: u32) -> Result<String, String> {
        if let Ok(adress) = self.find_dma_addy(
            self.g_name_start_address + ((actor_id / 0x4000) * 0x8) as usize,
            vec![0x8 * (actor_id % 0x4000) as usize],
        ) {
            self.read_string(adress + 0x10 as usize, Some(64))
        } else {
            Err(String::from("Failed to resolve pointer chain"))
        }
    }

    pub fn find_dma_addy(&self, mut ptr: usize, offsets: Vec<usize>) -> Result<usize, String> {
        for offset in offsets {
            ptr = self.read_ptr(ptr + offset)?
        }
        Ok(ptr)
    }
}

//     self.rm = ReadMemory("SoTGame.exe")
//         base_address = self.rm.base_address
//         logging.info(f"Process ID: {self.rm.pid}")

//         u_world_offset = self.rm.read_ulong(
//             base_address + self.rm.u_world_base + 3
//         )
//         u_world = base_address + self.rm.u_world_base + u_world_offset + 7
//         self.world_address = self.rm.read_ptr(u_world)

//         g_name_offset = self.rm.read_ulong(
//             base_address + self.rm.g_name_base + 3
//         )
//         g_name = base_address + self.rm.g_name_base + g_name_offset + 7
//         logging.info(f"SoT gName Address: {hex(g_name)}")
//         self.g_name = self.rm.read_ptr(g_name)

//         g_objects_offset = self.rm.read_ulong(
//             base_address + self.rm.g_object_base + 2
//         )
//         g_objects = base_address + self.rm.g_object_base + g_objects_offset + 22
//         logging.info(f"SoT gObject Address: {hex(g_objects)}")
//         self.g_objects = self.rm.read_ptr(g_objects)

//         self.u_level = self.rm.read_ptr(self.world_address +
//                                         OFFSETS.get('World.PersistentLevel'))

//         self.u_local_player = self._load_local_player()
//         self.player_controller = self.rm.read_ptr(
//             self.u_local_player + OFFSETS.get('LocalPlayer.PlayerController')
//         )

//         self.my_coords = self._coord_builder(self.u_local_player)
//         self.my_coords['fov'] = 90

//         self.actor_name_map = {}
//         self.server_players = []
//         self.display_objects = []
//         self.crew_data = None
// }
