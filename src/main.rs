// mod core;
mod helpers;

use std::io::Write;

use winapi::{
    ctypes::c_void,
    shared::minwindef::{BOOL, LPCVOID, LPVOID},
    um::{
        errhandlingapi::GetLastError,
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        winbase::FormatMessageA,
        winuser::GetAsyncKeyState,
    },
};

// use crate::core::memory_reader;
use crate::helpers::memory_helper;

const AMMO_OFFSET: [u32; 3] = [0x374, 0x14, 0x0];
const CAN_JUMP_OFFSET: [u32; 3] = [0x374, 0x8, 0x5D];

enum Keybinds {
    INFINITE_AMMO = 0x4F, // O
    ANTI_RECOIL = 0x50,   // P
    INFINITE_JUMP = 0x46, // F
}

struct CheatState {
    infinite_ammo: bool,
    anti_recoil: bool,
    infinite_jump: bool,
}

struct MemoryAdresses {
    player_adress: u32,
    ammo_adress: u32,
    can_jump_adress: u32,
}

struct CheatInstance {}

fn main() {
    let proc_id = memory_helper::get_proc_id("ac_client.exe");
    let base_address = memory_helper::get_module_base_adress(proc_id, "ac_client.exe")
        .expect("Could not get base adress");
    println!("{:#X}", base_address);
    let handle = memory_helper::get_process_handle(proc_id);

    loop {
        print!("\x1B[2J\x1B[1;1H");

        let ammo_address;
        let result =
            memory_helper::find_dma_addy(handle, base_address as u32 + 0x17E0A8, &AMMO_OFFSET);
        match result {
            Ok(address) => ammo_address = address,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        }

        let result = memory_helper::read_int(handle, ammo_address as usize);
        match result {
            Ok(ammo_value) => println!("{}", ammo_value),
            Err(err) => println!("{}", err),
        };
    }

    // let mut ammo_value: u32 = 200;
    // unsafe {
    //     WriteProcessMemory(
    //         handle,
    //         ammo_address as *mut _,
    //         &mut ammo_value as *mut u32 as LPCVOID,
    //         4,
    //         0 as *mut _,
    //     );
    // }

    // println!("{}", memory_helper::get_last_error_message());

    // println!("Ammos count: {}", ammo_value);
    // unsafe {
    //     ReadProcessMemory(
    //         handle,
    //         ammo_address as *mut _,
    //         &mut ammo_value as *mut u32 as *mut c_void,
    //         4,
    //         0 as *mut _,
    //     );
    // }
    // println!("Ammos count: {}", ammo_value);
}
