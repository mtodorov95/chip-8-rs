use std::{
    env,
    time::{Duration, Instant},
};

use chip8::Chip8;
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect};

mod chip8;

const PIXEL_SIZE: u32 = 10;
const WIDTH: u32 = 64 * PIXEL_SIZE;
const HEIGHT: u32 = 32 * PIXEL_SIZE;

fn main() {
    // SDL
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();

    let window = video_subsystem
        .window("chip-8-rs", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_ctx.event_pump().unwrap();
    let mut running: bool = true;

    // Args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: chip8-rs <path_to_rom>");
        return;
    }

    // Emulator
    let mut chip8 = Chip8::new();
    if let Err(e) = chip8.load_rom(&args[1]) {
        println!("Failed to load ROM: {}", e);
        return;
    }

    while running {
        let start = Instant::now();

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
                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = map_keycode_to_key(keycode) {
                        chip8.key_down(key);
                    }
                }
                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = map_keycode_to_key(keycode) {
                        chip8.key_up(key);
                    }
                }
                _ => {}
            }
        }

        chip8.cycle();
        draw_display(&chip8, &mut canvas);

        let delay = 2000;
        let elapsed = start.elapsed();
        if elapsed < Duration::from_micros(delay) {
            std::thread::sleep(Duration::from_micros(delay) - elapsed);
        }
    }
}

fn draw_display(chip8: &Chip8, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::GREEN);

    for y in 0..32 {
        for x in 0..64 {
            let index = y * 64 + x;
            if chip8.get_display()[index] {
                let _ = canvas.fill_rect(Rect::new(
                    (x as u32 * 10) as i32,
                    (y as u32 * 10) as i32,
                    10,
                    10,
                ));
            }
        }
    }

    canvas.present();
}

fn map_keycode_to_key(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
