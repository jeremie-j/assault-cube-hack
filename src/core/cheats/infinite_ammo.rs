use std::ptr::null_mut;

use winapi::um::{memoryapi::WriteProcessMemory, winnt::HANDLE};

use crate::{core::memory_reader, helpers::memory_helper, Keybind};

const AMMO_OFFSET: [u32; 3] = [0x374, 0x14, 0x0];

pub struct InfiniteAmmo {
    toggle_keybind: Keybind,
    proc_id: u32,
    game_base_adress: usize,
    game_handle: HANDLE,

    active: bool,
    ammo_adress: Option<usize>,
    ammo_value: Option<u32>,
}

impl InfiniteAmmo {
    fn update_ammo_adress(&mut self) {
        let result = memory_helper::find_dma_addy(
            self.game_handle,
            self.game_base_adress as u32 + 0x17E0A8,
            &AMMO_OFFSET,
        );
        match result {
            Ok(address) => self.ammo_adress = Some(address as usize),
            Err(err) => {
                self.ammo_adress = None;
                println!("{}", err);
            }
        };
    }

    fn get_ammo_value(&self, ammo_address: usize) -> Result<i32, String> {
        memory_helper::read_int(self.game_handle, ammo_address)
    }

    fn write_ammo_value(&self, ammo_address: usize, ammo_value: i32) {
        let value_byte = ammo_value.to_le_bytes();
        let address = ammo_address as *mut _;
        unsafe {
            WriteProcessMemory(
                self.game_handle,
                address,
                value_byte.as_ptr() as *const _,
                value_byte.len(),
                null_mut(),
            );
        }
    }
}

impl memory_reader::Cheat for InfiniteAmmo {
    fn update(&mut self) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }
        if let Some(valid_ammo_adress) = self.ammo_adress {
            match self.get_ammo_value(valid_ammo_adress) {
                Ok(value) => println!("Ammo value {}", value),
                Err(_) => self.update_ammo_adress(),
            };
            self.write_ammo_value(valid_ammo_adress, 9999);
        } else {
            self.update_ammo_adress()
        }
        Ok(())
    }

    fn new(
        toggle_keybind: Keybind,
        proc_id: u32,
        game_base_adress: usize,
        game_handle: HANDLE,
    ) -> InfiniteAmmo {
        InfiniteAmmo {
            toggle_keybind,
            proc_id,
            game_base_adress,
            game_handle,

            active: true,
            ammo_adress: None,
            ammo_value: None,
        }
    }
}
