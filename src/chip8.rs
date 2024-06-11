use std::fs::File;
use std::io::{self, Read};

// Fontset stored between 0x50 and onwards
const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,   // 0
    0x20, 0x60, 0x20, 0x20, 0x70,   // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
    0xF0, 0x80, 0xF0, 0x80, 0x80    // F
];

// Chip8 components struct
pub struct Chip8 {
    v: [u8; 16],        // General Purpose Registers v0 - vF
    index: u16,         // Index Register
    pc: u16,            // Program Counter
    sp: u8,             // Stack Pointer
    stack: [u16; 16],   // Stack
    memory: [u8; 4096], // Memory
    delay_timer: u8,    // Delay Timer
    sound_timer: u8,    // Sound Timer
    opcode: u16,        // Program Opperation Code
}

impl Chip8 {
    // New Chip8 emulation initialization
    // Initializes values at a default of 0, except for pc which is defined to start at 0x200
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            v: [0; 16],
            index: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
            opcode: 0,
        };
        chip8.load_fontset();
        chip8
    }

    // Load full fontset into memory starting at 0x50 as defined
    fn load_fontset(&mut self) {
        for(i, &byte) in CHIP8_FONTSET.iter().enumerate() {
            self.memory[0x50 + i] = byte;
        }
    }

    // Main emulation/program loop
    pub fn run(&mut self) {
        loop {
            self.opcode = self.fetch_opcode();      // Fetch
            self.decode_execute(self.opcode);       // Decode and Execute
            
            if self.delay_timer > 0 {               // Update delay timer
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {               // Update sound timer
                self.sound_timer -= 1;
            }

            // Sleep for 1/60th of a second to emulate 60 Hz cycle
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    // Fill memory with program commands
    pub fn load_rom(&mut self, path: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;     // Open File in Binary Mode
        let mut buffer: Vec<u8> = Vec::new();       // Create buffer of bytes   
        file.read_to_end(&mut buffer)?;        // Read file into buffer

        for (i, &byte) in buffer.iter().enumerate() {
            if i + 512 < self.memory.len() {
                self.memory[i + 512] = byte;
            } else {
                eprintln!("ROM is too large to fit in memory.");
                break;
            }
        }

        Ok(())
    }

    // Fetch the opcode from memory at the program counter location
    fn fetch_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16)
    }

    // Decode the opcode and run the associated function
    fn decode_execute (&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.cls(),                   // Clear Display
                0x00EE => self.ret(),                   // Return from subroutine
                _ => (),
            }
            0x1000 => self.jp(opcode * 0x0FFF),    // Jump to address NNN
            _ => (),
        }

    }

    /********************************************/
    /*          Instructions/Opcodes            */
    /********************************************/

    // Clear the display implementation
    fn cls(&mut self) {
        
    }

    // Return from subroutine implementation
    fn ret(&mut self) {

    }

    // Jump to address implementation
    fn jp(&mut self, addr: u16) {
        self.pc = addr;
    }
}