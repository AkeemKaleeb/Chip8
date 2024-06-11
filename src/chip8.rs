// Chip8 components struct
pub struct Chip8 {
    v: [u8; 16],        // General Purpose Registers v0 - vF
    i: u16,             // Index Register
    pc: u16,            // Program Counter
    sp: u8,             // Stack Pointer
    stack: [u16; 16],   // Stack
    memory: [u8; 4096], // Memory
    delay_timer: u8,    // Delay Timer
    sound_timer: u8,    // Sound Timer
}

impl Chip8 {
    // New Chip8 emulation initialization
    pub fn new() -> Self {
        Chip8 {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            memory: [0; 4096],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    // Main emulation loop
    pub fn run(&mut self) {
        loop {
            let opcode = self.fetch_opcode();       // Fetch
            self.decode_execute(opcode);            // Decode and Execute
            
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

    // Fetch the opcode from memory at the program counter location
    fn fetch_opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16)
    }

    // Decode the opcode and run the associated function
    fn decode_execute (&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.cls(),           // Clear Display
                0x00EE => self.ret(),           // Return from subroutine
                _ => (),
            }
            0x1000 => self.jp(opcode * 0x0FFF), // Jump to address NNN
            _ => (),
        }

    }

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