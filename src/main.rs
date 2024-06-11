mod chip8;

fn main() {
    let mut chip8 = chip8::Chip8::new();
    chip8.run();
}