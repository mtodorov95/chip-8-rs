use chip8::Chip8;
use sdl2::{keyboard::Keycode, pixels::Color};

mod chip8;

const PIXEL_SIZE: u32 = 10;
const WIDTH: u32 = 64 * PIXEL_SIZE;
const HEIGHT: u32 = 32 * PIXEL_SIZE;

fn main() {
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();

    let window = video_subsystem
        .window("chip-8-rs", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_ctx.event_pump().unwrap();
    let mut running: bool = true;

    let mut chip8 = Chip8::new();

    while running {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("Exiting...");
                    running = false;
                }
                _ => {}
            }
        }
    }
}
