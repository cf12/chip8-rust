use std::fs;


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
    0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
    0x20, 0x60, 0x20, 0x20, 0x70,  // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
    0xF0, 0x80, 0xF0, 0x80, 0x80   // F
];


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
}

impl Chip8 {
    pub fn new () -> Chip8 {
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
        };

        new_emu.mem[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn cycle (&mut self) {
        let opcode = self.mem[self.pc as usize] << 8 | self.mem[(self.pc + 1) as usize];
        println!("{:?}", opcode);
    }

    pub fn load_rom (&mut self, path: &str) {
        let data = fs::read(path).unwrap();
        self.mem[(MEMORY_START as usize)..].copy_from_slice(&data);
    }
}