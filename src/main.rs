mod core;
mod helpers;

use crate::core::cheats::infinite_ammo::InfiniteAmmo;
use crate::core::memory_reader::CheatInstance;

pub enum Keybind {
    InfiniteAmmo = 0x4F, // O
    AntiRecoil = 0x50,   // P
    InfiniteJump = 0x46, // F
}

fn main() {
    let mut instance = CheatInstance::new("ac_client.exe");
    instance.add(InfiniteAmmo::new(
        Keybind::InfiniteAmmo,
        instance.proc_id,
        instance.game_base_adress,
        instance.game_handle,
    ));
    loop {
        print!("\x1B[2J\x1B[1;1H");
        instance.refresh();
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
