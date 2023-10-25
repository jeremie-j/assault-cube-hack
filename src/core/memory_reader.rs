use std::str;

use winapi::um::winnt::HANDLE;

use crate::helpers::memory_helper;

pub enum Keybind {
    InfiniteAmmo = 0x4F, // O
    AntiRecoil = 0x50,   // P
    InfiniteJump = 0x46, // F
}

const AMMO_OFFSET: [u32; 3] = [0x374, 0x14, 0x0];
const CAN_JUMP_OFFSET: [u32; 3] = [0x374, 0x8, 0x5D];

pub trait Cheat {
    fn new(
        toggle_keybind: Keybind,
        proc_id: u32,
        game_base_adress: usize,
        game_handle: HANDLE,
    ) -> Self
    where
        Self: Sized;
    fn update(&mut self) -> Result<(), String>;
}

pub struct CheatInstance {
    pub proc_id: u32,
    pub game_base_adress: usize,
    pub game_handle: HANDLE,
    cheats: Vec<Box<dyn Cheat>>,
}

impl CheatInstance {
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

    pub fn add<T: Cheat + 'static>(&mut self, keybind: Keybind) {
        let cheat = T::new(
            keybind,
            self.proc_id,
            self.game_base_adress,
            self.game_handle,
        );
        self.cheats.push(Box::new(cheat));
    }

    pub fn run(&mut self) {
        loop {
            print!("\x1B[2J\x1B[1;1H");
            for cheat in &mut self.cheats {
                let _ = cheat.update();
            }
        }
    }
}
