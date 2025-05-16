use core::fmt;
use std::error::Error;
use std::result::Result;
use std::{borrow::BorrowMut, fs};

#[derive(Debug)]
pub enum Chip8Error {
    InvalidInstruction(u16),
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidInstruction(op) => write!(f, "Invalid instruction: {:#04X}", op),
        }
    }
}

impl Error for Chip8Error {}

pub const VIDEO_WIDTH: usize = 64;
pub const VIDEO_HEIGHT: usize = 32;

const MEMORY_SIZE: usize = 4096;
const MEMORY_START: usize = 0x200;
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
    stack: Vec<u16>,
    video: [bool; VIDEO_HEIGHT * VIDEO_WIDTH],
    keypad: [bool; NUM_KEYS],

    dt: u8,
    st: u8,

    rng: fn() -> u8,
}

#[derive(Debug, Clone, Copy)]
enum Opcode {
    ClearScreen,
    Return,
    Jump(u16),
    Call(u16),
    SkipEqualByte(usize, u8),
    SkipNotEqualByte(usize, u8),
    SkipEqual(usize, usize),
    LoadByte(usize, u8),
    AddByte(usize, u8),
    Load(usize, usize),
    Add(usize, usize),
    Or(usize, usize),
    And(usize, usize),
    Xor(usize, usize),
    Sub(usize, usize),
    SubN(usize, usize),
    ShiftRight(usize),
    ShiftLeft(usize),
    SkipNotEqual(usize, usize),
    LoadI(u16),
    JumpV0(u16),
    Random(usize, u8),
    Draw(usize, usize, u8),
    SkipKeyPress(usize),
    SkipKeyNotPress(usize),
    LoadDelayTimer(usize),
    LoadKeyPress(usize),
    LoadDelayTimerSet(usize),
    LoadSoundTimer(usize),
    AddI(usize),
    LoadFont(usize),
    LoadBCD(usize),
    StoreRegisters(usize),
    LoadRegisters(usize),
    Invalid(u16),
}

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, reg) in self.reg.iter().enumerate() {
            write!(f, "[v{:X}]: {:#02X}\n", i, reg)?;
        }

        let op =
            ((self.mem[self.pc as usize] as u16) << 8) | (self.mem[(self.pc + 1) as usize] as u16);

        write!(f, "[pc]: {:#02X}\n", self.pc)?;
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
            stack: vec![],
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

    pub fn set_keypad(&mut self, key: usize, value: bool) {
        self.keypad[key] = value;
    }

    fn decode_opcode(&self, op: u16) -> Opcode {
        let b1 = (op & 0xF000) >> 12;
        let x = ((op & 0x0F00) >> 8) as usize;
        let y = ((op & 0x00F0) >> 4) as usize;
        let nnn = op & 0x0FFF;
        let nn = (op & 0x00FF) as u8;
        let n = (op & 0x000F) as u8;

        match b1 {
            0x0 => match nnn {
                0x00E0 => Opcode::ClearScreen,
                0x00EE => Opcode::Return,
                _ => Opcode::Invalid(op),
            },
            0x1 => Opcode::Jump(nnn),
            0x2 => Opcode::Call(nnn),
            0x3 => Opcode::SkipEqualByte(x, nn),
            0x4 => Opcode::SkipNotEqualByte(x, nn),
            0x5 => Opcode::SkipEqual(x, y),
            0x6 => Opcode::LoadByte(x, nn),
            0x7 => Opcode::AddByte(x, nn),
            0x8 => match n {
                0x0 => Opcode::Load(x, y),
                0x1 => Opcode::Or(x, y),
                0x2 => Opcode::And(x, y),
                0x3 => Opcode::Xor(x, y),
                0x4 => Opcode::Add(x, y),
                0x5 => Opcode::Sub(x, y),
                0x6 => Opcode::ShiftRight(x),
                0x7 => Opcode::SubN(x, y),
                0xE => Opcode::ShiftLeft(x),
                _ => Opcode::Invalid(op),
            },
            0x9 => Opcode::SkipNotEqual(x, y),
            0xA => Opcode::LoadI(nnn),
            0xB => Opcode::JumpV0(nnn),
            0xC => Opcode::Random(x, nn),
            0xD => Opcode::Draw(x, y, n),
            0xE => match nn {
                0x9E => Opcode::SkipKeyPress(x),
                0xA1 => Opcode::SkipKeyNotPress(x),
                _ => Opcode::Invalid(op),
            },
            0xF => match nn {
                0x07 => Opcode::LoadDelayTimer(x),
                0x0A => Opcode::LoadKeyPress(x),
                0x15 => Opcode::LoadDelayTimerSet(x),
                0x18 => Opcode::LoadSoundTimer(x),
                0x1E => Opcode::AddI(x),
                0x29 => Opcode::LoadFont(x),
                0x33 => Opcode::LoadBCD(x),
                0x55 => Opcode::StoreRegisters(x),
                0x65 => Opcode::LoadRegisters(x),
                _ => Opcode::Invalid(op),
            },
            _ => Opcode::Invalid(op),
        }
    }

    fn fetch_opcode(&self) -> u16 {
        let op =
            ((self.mem[self.pc as usize] as u16) << 8) | (self.mem[(self.pc + 1) as usize] as u16);
        op
    }

    pub fn cycle(&mut self) -> Result<(), Chip8Error> {
        let raw_opcode = self.fetch_opcode();
        let opcode = self.decode_opcode(raw_opcode);

        self.pc += 2;

        match opcode {
            // 00E0 - CLS
            Opcode::ClearScreen => {
                self.video.fill(false);
            }

            // 00EE - RET
            Opcode::Return => {
                self.pc = self.stack.pop().unwrap();
            }

            // 1nnn - JP addr
            Opcode::Jump(nnn) => {
                self.pc = nnn;
            }

            // 2nnn - CALL addr
            Opcode::Call(nnn) => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }

            // 3xkk - SE Vx, byte
            Opcode::SkipEqualByte(x, nn) => {
                if self.reg[x] == nn {
                    self.pc += 2
                };
            }

            // 4xkk - SNE Vx, byte
            Opcode::SkipNotEqualByte(x, nn) => {
                if self.reg[x] != nn {
                    self.pc += 2
                };
            }

            // 5xy0 - SE Vx, Vy
            Opcode::SkipEqual(x, y) => {
                if self.reg[x] == self.reg[y] {
                    self.pc += 2
                };
            }

            // 6xkk - LD Vx, byte
            Opcode::LoadByte(x, nn) => {
                self.reg[x] = nn;
            }

            // 7xkk - ADD Vx, byte
            Opcode::AddByte(x, nn) => {
                self.reg[x] = self.reg[x].wrapping_add(nn);
            }

            // 8xy0 - LD Vx, Vy
            Opcode::Load(x, y) => {
                self.reg[x] = self.reg[y];
            }

            // 8xy1 - OR Vx, Vy
            Opcode::Or(x, y) => {
                self.reg[x] |= self.reg[y];
            }

            // 8xy2 - AND Vx, Vy
            Opcode::And(x, y) => {
                self.reg[x] &= self.reg[y];
            }

            // 8xy3 - XOR Vx, Vy
            Opcode::Xor(x, y) => {
                self.reg[x] ^= self.reg[y];
            }

            // 8xy4 - ADD Vx, Vy
            Opcode::Add(x, y) => {
                let (res, carry) = self.reg[x].overflowing_add(self.reg[y]);

                self.reg[x] = res;
                self.reg[0xF] = carry as u8;
            }

            // 8xy5 - SUB Vx, Vy
            Opcode::Sub(x, y) => {
                let (res, borrow) = self.reg[x].overflowing_sub(self.reg[y]);
                self.reg[x] = res;
                self.reg[0xF] = !borrow as u8;
            }

            // 8xy6 - SHR Vx {, Vy}
            Opcode::ShiftRight(x) => {
                self.reg[0xF] = self.reg[x] & 1;
                self.reg[x] >>= 1;
            }

            // 8xy7 - SUBN Vx, Vy
            Opcode::SubN(x, y) => {
                let (res, borrow) = self.reg[y].overflowing_sub(self.reg[x]);
                self.reg[x] = res;
                self.reg[0xF] = !borrow as u8;
            }

            // 8xyE - SHL Vx {, Vy}
            Opcode::ShiftLeft(x) => {
                self.reg[0xF] = (self.reg[x] >> 7) & 1;
                self.reg[x] <<= 1;
            }

            // 9xy0 - SNE Vx, Vy
            Opcode::SkipNotEqual(x, y) => {
                if self.reg[x] != self.reg[y] {
                    self.pc += 2
                };
            }

            // Annn - LD I, addr
            Opcode::LoadI(nnn) => {
                self.i = nnn;
            }

            // Bnnn - JP V0, addr
            Opcode::JumpV0(nnn) => {
                self.pc = (self.reg[0x0] as u16) + nnn;
            }

            // Cxkk - RND Vx, byte
            Opcode::Random(x, nn) => {
                self.reg[x] = (self.rng)() & nn;
            }

            // Dxyn - DRW Vx, Vy, nibble
            Opcode::Draw(x, y, n) => {
                let x = self.reg[x] as u16;
                let y = self.reg[y] as u16;
                let height = n as u16;

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

            // Ex9E - SKP Vx
            Opcode::SkipKeyPress(x) => {
                let key = self.reg[x] as usize;
                if self.keypad[key] {
                    self.pc += 2
                };
            }

            // ExA1 - SKNP Vx
            Opcode::SkipKeyNotPress(x) => {
                let key = self.reg[x] as usize;
                if !self.keypad[key] {
                    self.pc += 2
                };
            }

            // Fx07 - LD Vx, DT
            Opcode::LoadDelayTimer(x) => {
                self.reg[x] = self.dt;
            }

            // Fx0A - LD Vx, K
            Opcode::LoadKeyPress(x) => {
                if let Some(i) = self.keypad.iter().position(|&key| key) {
                    self.reg[x] = i as u8;
                } else {
                    self.pc -= 2;
                }
            }

            // Fx15 - LD DT, Vx
            Opcode::LoadDelayTimerSet(x) => {
                self.dt = self.reg[x];
            }

            // Fx18 - LD ST, Vx
            Opcode::LoadSoundTimer(x) => {
                self.st = self.reg[x];
            }

            // Fx1E - ADD I, Vx
            Opcode::AddI(x) => {
                self.i += self.reg[x] as u16;
            }

            // Fx29 - LD F, Vx
            Opcode::LoadFont(x) => {
                let digit = self.reg[x];

                self.i = FONTSET_START_ADDRESS as u16 + digit as u16 * 5;
            }

            // Fx33 - LD B, Vx
            Opcode::LoadBCD(x) => {
                let mut value = self.reg[x];

                self.mem[(self.i + 2) as usize] = value % 10;
                value /= 10;
                self.mem[(self.i + 1) as usize] = value % 10;
                value /= 10;
                self.mem[self.i as usize] = value % 10;
            }

            // Fx55 - LD [I], Vx
            Opcode::StoreRegisters(x) => {
                for v in 0..=x {
                    self.mem[self.i as usize + v] = self.reg[v];
                }
            }

            // Fx65 - LD Vx, [I]
            Opcode::LoadRegisters(x) => {
                for v in 0..=x {
                    self.reg[v] = self.mem[self.i as usize + v];
                }
            }

            // Invalid opcode
            Opcode::Invalid(_) => {
                return Err(Chip8Error::InvalidInstruction(raw_opcode));
            }
        }

        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }

        return Ok(());
    }
}
