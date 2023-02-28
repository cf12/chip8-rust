extern crate sdl2;
mod chip8;

use chip8::Chip8;
use chip8::VIDEO_HEIGHT;
use chip8::VIDEO_WIDTH;
use sdl2::rect::Rect;
use std::env;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

pub fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <rom_file> <scale>", args[0]);
        std::process::exit(1);
    }

    let rng = rand::random::<u8>;
    let mut cpu = Chip8::new(rng);
    let scale = &args[2].parse::<u32>().unwrap();

    cpu.load_rom(&args[1]);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "CHIP8 Rust",
            VIDEO_WIDTH as u32 * scale,
            VIDEO_HEIGHT as u32 * scale,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.clear();
        cpu.cycle();

        let video = cpu.get_video();
        // println!("{:?}", video);

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (i, pixel) in video.iter().enumerate() {
            if *pixel {
                // Convert our 1D array's index into a 2D (x,y) position
                let x = (i % VIDEO_WIDTH) as u32;
                let y = (i / VIDEO_WIDTH) as u32;

                // println!("drawing: {}, {}", x, y);

                // Draw a rectangle at (x,y), scaled up by our SCALE value
                let rect = Rect::new((x * scale) as i32, (y * scale) as i32, *scale, *scale);
                canvas.fill_rect(rect).unwrap();
            }
        }

        canvas.present();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        ::std::thread::sleep(Duration::new(1, 1_000_000_000u32 / 60));
    }
}
