use std::str;

use winapi::um::winnt::HANDLE;

use crate::helpers::memory_helper;

const AMMO_OFFSET: [u32; 3] = [0x374, 0x14, 0x0];
const CAN_JUMP_OFFSET: [u32; 3] = [0x374, 0x8, 0x5D];

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

pub trait Cheat {
    fn update(&mut self) -> Result<(), String>;
}

pub struct CheatInstance<T: Cheat> {
    pub proc_id: u32,
    pub game_base_adress: usize,
    pub game_handle: HANDLE,
    cheats: Vec<T>,
}

impl<T: Cheat> CheatInstance<T> {
    pub fn new(exe_name: &str) -> Self {
        let proc_id = memory_helper::get_proc_id(exe_name).unwrap();
        let game_base_adress = memory_helper::get_module_base_adress(proc_id, exe_name).unwrap();
        let game_handle = memory_helper::get_process_handle(proc_id as u32);

        Self {
            proc_id,
            game_base_adress,
            game_handle,
            cheats: Vec::new(),
        }
    }

    pub fn add(&mut self, cheat: T) {
        self.cheats.push(cheat);
    }

    pub fn refresh(&mut self) {
        for cheat in &mut self.cheats {
            cheat.update();
        }
    }
}
