use core::fmt;
use std::{borrow::BorrowMut, fs};

pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;

const MEMORY_SIZE: usize = 4096;
const MEMORY_START: usize = 0x200;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const NUM_REGS: usize = 16;

const FONTSET_START_ADDRESS: usize = 0x50;
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

#[derive(Debug, Clone)]
pub struct Chip8 {
    mem: [u8; MEMORY_SIZE],
    reg: [u8; NUM_REGS],

    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; STACK_SIZE],
    video: [bool; VIDEO_HEIGHT * VIDEO_WIDTH],
    keypad: [bool; NUM_KEYS],

    dt: u8,
    st: u8,

    rng: fn() -> u8,
}

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, reg) in self.reg.iter().enumerate() {
            write!(f, "[v{:X}]: {:#02X}\n", i, reg)?;
        }

        let op =
            ((self.mem[self.pc as usize] as u16) << 8) | (self.mem[(self.pc + 1) as usize] as u16);

        write!(f, "[pc]: {:#02X}\n", self.pc)?;
        write!(f, "[sp]: {:#02X}\n", self.sp)?;
        write!(f, "[i]: {:#02X}\n", self.i)?;
        write!(f, "[opcode]: {:#04X}\n", op)
    }
}

impl Chip8 {
    pub fn new(rng: fn() -> u8) -> Chip8 {
        let mut new_emu = Chip8 {
            mem: [0; MEMORY_SIZE],
            reg: [0; NUM_REGS],

            i: 0,
            pc: MEMORY_START as u16,
            sp: 0,
            stack: [0; STACK_SIZE],
            video: [false; VIDEO_HEIGHT * VIDEO_WIDTH],
            keypad: [false; NUM_KEYS],

            dt: 0,
            st: 0,

            rng: rng,
        };

        new_emu.mem[FONTSET_START_ADDRESS..FONTSET_START_ADDRESS + FONTSET_SIZE]
            .copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn load_rom(&mut self, path: &String) {
        let data = fs::read(path).expect("Cannot read ROM file");
        self.mem[MEMORY_START..MEMORY_START + data.len()].copy_from_slice(&data);
    }

    pub fn get_video(&self) -> &[bool] {
        return &self.video;
    }

    pub fn cycle(&mut self) {
        // println!("{}", &self);
        let op =
            ((self.mem[self.pc as usize] as u16) << 8) | (self.mem[(self.pc + 1) as usize] as u16);

        self.pc += 2;

        let b1 = (op & 0xF000) >> 12;
        #[allow(non_snake_case)]
        let Vx = ((op & 0x0F00) >> 8) as usize;
        #[allow(non_snake_case)]
        let Vy = ((op & 0x00F0) >> 4) as usize;
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
                    0x0EE => {
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
                self.reg[Vx] = self.reg[Vx].wrapping_add(byte);
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
                        let (res, carry) = self.reg[Vx].overflowing_add(self.reg[Vy]);

                        self.reg[Vx] = res;
                        self.reg[0xF] = carry as u8;
                    }

                    // 8xy5 - SUB Vx, Vy
                    0x5 => {
                        let (res, carry) = self.reg[Vx].overflowing_sub(self.reg[Vy]);
                        self.reg[Vx] = res;
                        self.reg[0xF] = carry as u8;
                    }

                    // 8xy6 - SHR Vx {, Vy}
                    0x6 => {
                        self.reg[0xF] = self.reg[Vx] & 1;
                        self.reg[Vx] = self.reg[Vx] >> 1;
                    }

                    // 8xy7 - SUBN Vx, Vy
                    0x7 => {
                        let (res, carry) = self.reg[Vy].overflowing_sub(self.reg[Vx]);
                        self.reg[Vx] = res;
                        self.reg[0xF] = carry as u8;
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
                self.i = addr;
            }

            // Bnnn - JP V0, addr
            0xB => {
                self.pc = (self.reg[0x0] as u16) + addr;
            }

            // Cxkk - RND Vx, byte
            0xC => {
                self.reg[Vx] = (self.rng)() & byte;
            }

            // Dxyn - DRW Vx, Vy, nibble
            0xD => {
                let x = self.reg[Vx] as u16;
                let y = self.reg[Vy] as u16;
                let height = n;

                self.reg[0xF] = 0;

                for dy in 0..height {
                    let sprite = self.mem[(self.i + dy as u16) as usize];

                    for dx in 0..8u16 {
                        let x = (x + dx) as usize % VIDEO_WIDTH;
                        let y = (y + dy) as usize % VIDEO_HEIGHT;

                        let sprite_pixel = sprite & (0b1000_0000 >> dx);
                        let video_pixel = self.video[y * VIDEO_WIDTH + x].borrow_mut();

                        if sprite_pixel != 0 {
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
                        panic!("Invalid instruction: {:#04X}", op);
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
                        self.i = self.i.wrapping_add(self.reg[Vx] as u16);
                    }

                    // Fx29 - LD F, Vx
                    0x29 => {
                        let digit = self.reg[Vx];

                        self.i = FONTSET_START_ADDRESS as u16 + digit as u16 * 5;
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
                        panic!("Invalid instruction: {:#04X}", op);
                    }
                }
            }
            _ => {
                panic!("Invalid instruction: {:#04X}", op);
            }
        }

        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }
}
