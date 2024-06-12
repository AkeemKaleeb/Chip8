use std::env;

mod chip8;

fn main() -> Result<(), String> {
    // Command Line arguments: Usage: cargo run <rom_path>
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Error Usage: {} <rom_path>", args[0]);
        std::process::exit(1);
    }

    let mut chip8 = chip8::Chip8::new();
    let _ = chip8.load_rom(&args[1]);
    let _ = chip8.run();

    Ok(())
}