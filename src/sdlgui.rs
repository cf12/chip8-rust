extern crate sdl2;

use crate::chip8::Chip8;
use crate::chip8::VIDEO_HEIGHT;
use crate::chip8::VIDEO_WIDTH;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::EventPump;
use std::collections::HashMap;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::Duration;

/*
    1	2	3	4
    Q	W	E	R
    A   S   D   F
    Z   X   C   V

    1	2	3	C
    4	5	6	D
    7	8	9	E
    A	0	B	F
*/

pub struct SDLGui {
    cpu: Chip8,
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    scale: u32,
    keymap: HashMap<&'static str, usize>,
}

impl SDLGui {
    pub fn new(cpu: Chip8, scale: u32) -> SDLGui {
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

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        let keymap: HashMap<&str, usize> = HashMap::from([
            ("1", 0x1),
            ("2", 0x2),
            ("3", 0x3),
            ("4", 0xC),
            ("Q", 0x4),
            ("W", 0x5),
            ("E", 0x6),
            ("R", 0xD),
            ("A", 0x7),
            ("S", 0x8),
            ("D", 0x9),
            ("F", 0xE),
            ("Z", 0xA),
            ("X", 0x0),
            ("C", 0xB),
            ("V", 0xF),
        ]);

        SDLGui {
            cpu,
            sdl_context,
            canvas,
            event_pump,
            scale,
            keymap,
        }
    }

    pub fn read_keys(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    let kval = self.keymap.get(k.to_string().as_str());
                    match kval {
                        Some(val) => self.cpu.set_keypad(*val, true),
                        None => {}
                    }
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    let kval = self.keymap.get(k.to_string().as_str());
                    match kval {
                        Some(val) => self.cpu.set_keypad(*val, false),
                        None => {}
                    }
                }
                _ => {}
            }
        }

        return true;
    }

    pub fn run(&mut self) {
        let fps = 10;
        let duration = Duration::new(0, 1_000_000_000 / (60 * fps));

        loop {
            if !self.read_keys() {
                break;
            }

            self.canvas.clear();

            let now = Instant::now();
            self.cpu.cycle();
            let elapsed = now.elapsed();

            let video = self.cpu.get_video();

            self.canvas.set_draw_color(Color::RGB(255, 255, 255));
            for (i, pixel) in video.iter().enumerate() {
                if *pixel {
                    let x = (i % VIDEO_WIDTH) as u32;
                    let y = (i / VIDEO_WIDTH) as u32;

                    let rect = Rect::new(
                        (x * self.scale) as i32,
                        (y * self.scale) as i32,
                        self.scale,
                        self.scale,
                    );
                    self.canvas.fill_rect(rect).unwrap();
                }
            }

            self.canvas.present();
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));

            if elapsed < duration {
                std::thread::sleep(duration - elapsed);
            }
        }
    }
}
