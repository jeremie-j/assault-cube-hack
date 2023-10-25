use {
    ggez::{
        conf::{FullscreenType, WindowMode},
        winit::platform::windows::WindowExtWindows,
        Context,
    },
    winapi::{
        shared::windef::HWND,
        um::winuser::{
            GetWindowLongA, SetWindowLongA, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST, SWP_NOMOVE,
            SWP_NOSIZE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
        },
    },
};

pub const WINDOW_MODE: WindowMode = WindowMode {
    width: 800.0,
    height: 600.0,
    maximized: true,
    fullscreen_type: FullscreenType::Windowed,
    borderless: false,
    min_width: 1.0,
    max_width: 0.0,
    min_height: 1.0,
    max_height: 0.0,
    resizable: false,
    visible: true,
    transparent: true,
    resize_on_scale_factor_change: false,
    logical_size: None,
};

pub fn setup_window(ctx: &Context, event_loop: &ggez::event::EventLoop<()>) -> Result<(), i32> {
    let primary_monitor = event_loop.primary_monitor();
    let window: &ggez::winit::window::Window = ctx.gfx.window();
    window.set_fullscreen(Some(ggez::winit::window::Fullscreen::Borderless(
        primary_monitor,
    )));
    // window.set_always_on_top(true);
    let hwnd = window.hwnd() as HWND;

    unsafe {
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);

        let ex_style = GetWindowLongA(hwnd, GWL_EXSTYLE);
        SetWindowLongA(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_LAYERED as i32 | WS_EX_TRANSPARENT as i32,
        );
    };
    Ok(())
}
