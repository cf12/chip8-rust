mod chip8;
mod sdlgui;

use crate::chip8::Chip8;
use crate::sdlgui::SDLGui;

use clap::Parser;

/// Chip-8 Emulator in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ROM file to load
    #[arg()]
    rom_file: String,

    /// Graphics scale
    #[arg(default_value_t = 20)]
    scale: u32,
}

pub fn main() {
    let args = Args::parse();
    let rng = rand::random::<u8>;
    let mut cpu = Chip8::new(rng);
    cpu.load_rom(&args.rom_file);
    let mut gui = SDLGui::new(cpu, args.scale);
    gui.run();
}
