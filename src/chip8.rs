use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{borrow::BorrowMut, fs};

const VIDEO_WIDTH: usize = 64;
const VIDEO_HEIGHT: usize = 32;

const MEMORY_SIZE: usize = 4096;
const MEMORY_START: u16 = 0x200;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const NUM_REGS: usize = 16;

const FONTSET_START_ADDRESS: u16 = 0x50;
const FONTSET_SIZE: usize = 5 * 16;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Clone, Debug)]
pub struct Chip8 {
    mem: [u8; MEMORY_SIZE],
    reg: [u8; NUM_REGS],

    i: usize,
    pc: u16,
    sp: u8,
    stack: [u16; STACK_SIZE],
    video: [bool; VIDEO_HEIGHT * VIDEO_WIDTH],
    keypad: [bool; NUM_KEYS],

    dt: u8,
    st: u8,

    rng: ChaCha8Rng,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut new_emu = Chip8 {
            mem: [0; MEMORY_SIZE],
            reg: [0; NUM_REGS],

            i: 0,
            pc: MEMORY_START,
            sp: 0,
            stack: [0; STACK_SIZE],
            video: [false; VIDEO_HEIGHT * VIDEO_WIDTH],
            keypad: [false; NUM_KEYS],

            dt: 0,
            st: 0,

            // TODO: remove hardcoded seed
            rng: ChaCha8Rng::seed_from_u64(12345),
        };

        new_emu.mem[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn cycle(&mut self) {
        let op =
            ((self.mem[self.pc as usize] as u16) << 8) | (self.mem[(self.pc + 1) as usize] as u16);

        self.pc += 2;

        let b1 = op & 0xF000;
        #[allow(non_snake_case)]
        let Vx = (op & 0x0F00) as usize;
        #[allow(non_snake_case)]
        let Vy = (op & 0x00F0) as usize;
        let addr = op & 0x0FFF;
        let byte = (op & 0x00FF) as u8;
        let n = op & 0x000F;

        match b1 {
            0x0 => {
                match addr {
                    // 00E0 - CLS
                    0x0E0 => {
                        self.video.fill(false);
                    }

                    // 00EE - RET
                    0x00EE => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }

                    // 0nnn - SYS addr
                    _ => {}
                }
            }

            // 1nnn - JP addr
            0x1 => {
                self.pc = addr;
            }

            // 2nnn - CALL addr
            0x2 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = addr;
            }

            // 3xkk - SE Vx, byte
            0x3 => {
                if self.reg[Vx] == byte {
                    self.pc += 2
                };
            }

            // 4xkk - SNE Vx, byte
            0x4 => {
                if self.reg[Vx] != byte {
                    self.pc += 2
                };
            }

            // 5xy0 - SE Vx, Vy
            0x5 => {
                if self.reg[Vx] == self.reg[Vy] {
                    self.pc += 2
                };
            }

            // 6xkk - LD Vx, byte
            0x6 => {
                self.reg[Vx] = byte;
            }

            // 7xkk - ADD Vx, byte
            0x7 => {
                self.reg[Vx] += byte;
            }

            0x8 => {
                match n {
                    0x0 => {
                        self.reg[Vx] = self.reg[Vy];
                    }

                    // 8xy1 - OR Vx, Vy
                    0x1 => {
                        self.reg[Vx] |= self.reg[Vy];
                    }

                    // 8xy2 - AND Vx, Vy
                    0x2 => {
                        self.reg[Vx] &= self.reg[Vy];
                    }

                    // 8xy3 - XOR Vx, Vy
                    0x3 => {
                        self.reg[Vx] ^= self.reg[Vy];
                    }

                    // 8xy4 - ADD Vx, Vy
                    0x4 => {
                        let sum = self.reg[Vx] as u16 + self.reg[Vy] as u16;

                        // set carry bit
                        self.reg[0xF] = (sum >> 8) as u8;

                        // implicitly truncate carry bit
                        self.reg[Vx] = sum as u8;
                    }

                    // 8xy5 - SUB Vx, Vy
                    0x5 => {
                        self.reg[0xF] = (self.reg[Vx] > self.reg[Vy]) as u8;
                        self.reg[Vx] = self.reg[Vx] - self.reg[Vy];
                    }

                    // 8xy6 - SHR Vx {, Vy}
                    0x6 => {
                        self.reg[0xF] = self.reg[Vx] & 1;
                        self.reg[Vx] = self.reg[Vx] >> 1;
                    }

                    // 8xy7 - SUBN Vx, Vy
                    0x7 => {
                        self.reg[0xF] = (self.reg[Vy] > self.reg[Vx]) as u8;
                        self.reg[Vx] = self.reg[Vy] - self.reg[Vx];
                    }

                    // 8xyE - SHL Vx {, Vy}
                    0xE => {
                        self.reg[0xF] = (self.reg[Vx] >> 7) & 1;
                        self.reg[Vx] = self.reg[Vx] << 1;
                    }

                    _ => {
                        panic!("Invalid instruction: {:?}", op);
                    }
                }
            }

            // 9xy0 - SNE Vx, Vy
            0x9 => {
                if self.reg[Vx] != self.reg[Vy] {
                    self.pc += 2
                };
            }

            // Annn - LD I, addr
            0xA => {
                self.i = addr as usize;
            }

            // Bnnn - JP V0, addr
            0xB => {
                self.pc = (self.reg[0x0] as u16) + addr;
            }

            // Cxkk - RND Vx, byte
            0xC => {
                self.reg[Vx] = self.rng.gen::<u8>() & byte;
            }

            // Dxyn - DRW Vx, Vy, nibble
            0xD => {
                let height = n;

                let x = self.reg[Vx] % (VIDEO_WIDTH as u8);
                let y = self.reg[Vy] % (VIDEO_HEIGHT as u8);

                self.reg[0xF] = 0;

                for dy in 0..height as u8 {
                    let sprite = self.mem[((self.i as u8) + dy) as usize];

                    for dx in 0..8 as u8 {
                        let sprite_pixel = (sprite >> (7 - dx)) & 1;
                        let video_pixel = self.video
                            [(y + dy) as usize * VIDEO_WIDTH + (x + dx) as usize]
                            .borrow_mut();

                        if sprite_pixel == 1 {
                            if *video_pixel {
                                self.reg[0xF] = 1;
                            }

                            *video_pixel ^= true;
                        }
                    }
                }
            }

            0xE => {
                match byte {
                    // Ex9E - SKP Vx
                    0x9E => {
                        let key = self.reg[Vx] as usize;
                        if self.keypad[key] {
                            self.pc += 2
                        };
                    }

                    // ExA1 - SKNP Vx
                    0xA1 => {
                        let key = self.reg[Vx] as usize;
                        if !self.keypad[key] {
                            self.pc += 2
                        };
                    }

                    _ => {
                        panic!("Invalid instruction: {:?}", op);
                    }
                }
            }

            0xF => {
                match byte {
                    // Fx07 - LD Vx, DT
                    0x07 => {
                        self.reg[Vx] = self.dt;
                    }

                    // Fx0A - LD Vx, K
                    0x0A => {
                        for i in 0..16 {
                            if self.keypad[i as usize] {
                                self.reg[Vx] = i;
                                return;
                            }
                        }

                        self.pc -= 2;
                    }

                    // Fx15 - LD DT, Vx
                    0x15 => {
                        self.dt = self.reg[Vx];
                    }

                    // Fx18 - LD ST, Vx
                    0x18 => {
                        self.st = self.reg[Vx];
                    }

                    // Fx1E - ADD I, Vx
                    0x1E => {
                        self.i += self.reg[Vx] as usize;
                    }

                    // Fx29 - LD F, Vx
                    0x29 => {
                        let digit = self.reg[Vx];

                        self.i = (FONTSET_START_ADDRESS + digit as u16 * 5) as usize;
                    }

                    // Fx33 - LD B, Vx
                    0x33 => {
                        let mut value = self.reg[Vx];

                        self.mem[self.i as usize + 2] = value % 10;
                        value /= 10;
                        self.mem[self.i as usize + 1] = value % 10;
                        value /= 10;
                        self.mem[self.i as usize] = value % 10;
                    }

                    // Fx55 - LD [I], Vx
                    0x55 => {
                        for v in 0..Vx {
                            self.mem[self.i as usize + v] = self.reg[v];
                        }
                    }

                    // Fx65 - LD Vx, [I]
                    0x65 => {
                        for v in 0..Vx {
                            self.reg[v] = self.mem[self.i as usize + v];
                        }
                    }

                    _ => {
                        panic!("Invalid instruction: {:?}", op);
                    }
                }
            }
            _ => {}
        }

        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn load_rom(&mut self, path: &String) {
        let data = fs::read(path).expect("Cannot read ROM file");
        self.mem[(MEMORY_START as usize)..((MEMORY_START as usize) + data.len())]
            .copy_from_slice(&data);
    }
}
