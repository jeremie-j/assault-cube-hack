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
    instance.run();
}
