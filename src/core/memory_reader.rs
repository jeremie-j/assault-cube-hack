use {ggez::event, std::str};

use winapi::um::winnt::HANDLE;

use {
    crate::helpers::memory_helper,
    ggez::{Context, GameResult},
};

use ggez::{
    glam::*,
    graphics::{self, Color},
};

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
    pub fn new(exe_name: &str) -> GameResult<CheatInstance> {
        let proc_id = memory_helper::get_proc_id(exe_name).unwrap();
        let game_base_adress = memory_helper::get_module_base_adress(proc_id, exe_name).unwrap();
        let game_handle = memory_helper::get_process_handle(proc_id as u32);

        Ok(CheatInstance {
            proc_id,
            game_base_adress,
            game_handle,
            cheats: Vec::new(),
        })
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
}

impl event::EventHandler<ggez::GameError> for CheatInstance {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        for cheat in &mut self.cheats {
            let _ = cheat.update();
        }
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        let circle = graphics::Mesh::new_circle(
            _ctx,
            graphics::DrawMode::fill(),
            vec2(0., 0.),
            20.0,
            2.0,
            Color::WHITE,
        )?;

        let mut canvas =
            graphics::Canvas::from_frame(_ctx, graphics::Color::from([0.0, 0.0, 0.0, 0.0]));

        canvas.draw(&circle, Vec2::new(500.0, 500.0));

        canvas.finish(_ctx)?;

        Ok(())
    }
}
