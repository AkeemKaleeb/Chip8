use std::env;

mod chip8;

fn main() -> Result<(), std::io::Error> {
    // Command Line arguments: Usage: cargo run <rom_path>
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <rom_path>", args[0]);
        return Ok(());
    }

    let mut chip8 = chip8::Chip8::new();
    chip8.load_rom(&args[1])?;
    chip8.run();

    Ok(())
}