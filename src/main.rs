mod core;
mod helpers;
use ggez::{event, ContextBuilder, GameResult};

use core::{
    cheats::infinite_ammo::InfiniteAmmo,
    memory_reader::{CheatInstance, Keybind},
    window::{setup_window, WINDOW_MODE},
};

fn main() -> GameResult {
    let cb = ContextBuilder::new("ac_hack", "potagerdenavets").window_mode(WINDOW_MODE);
    let (ctx, event_loop) = cb.build()?;

    let _window = ctx.gfx.window();
    let _ = setup_window(&ctx, &event_loop);

    let mut cheat_instance = CheatInstance::new("ac_client.exe").unwrap();
    cheat_instance.add::<InfiniteAmmo>(Keybind::InfiniteAmmo);

    event::run(ctx, event_loop, cheat_instance)
}
