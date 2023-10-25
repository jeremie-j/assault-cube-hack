mod core;
mod helpers;

use core::cheats::infinite_ammo::InfiniteAmmo;
use core::memory_reader::{CheatInstance, Keybind};

fn main() {
    let mut instance = CheatInstance::new("ac_client.exe");
    instance.add::<InfiniteAmmo>(Keybind::InfiniteAmmo);
    instance.run();
}
