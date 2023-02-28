use chip8::Chip8;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let mut cpu = Chip8::new();
    cpu.load_rom(&args[1]);
    cpu.cycle();
}
