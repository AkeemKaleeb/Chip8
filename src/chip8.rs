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
                _ => self.pc += 2,              // Skip unknown code
            }
            0x1000 => self.jmp(opcode),         // Jump to address NNN
            0x2000 => self.jsr(opcode),         // Jump to subroutine NNN
            0x3000 => self.skeq_c(opcode),      // Skip next instruction if v[x] == rr
            0x4000 => self.skne_c(opcode),      // Skip next instruction if v[X] != rr
            0x5000 => self.skeq_r(opcode),      // Skip next instruction if v[X] == v[Y]
            0x6000 => self.mov_c(opcode),       // Move constant rr to register v[X]
            0x7000 => self.add_c(opcode),       // Add constant rr to register v[X]
            0x8000 => match opcode & 0x000F {
                0x000 => self.mov_r(opcode),    // Move register v[Y] into v[X]
                0x001 => self.or_r(opcode),     // OR register v[Y] with v[X]
                0x002 => self.and_r(opcode),    // AND register v[Y] with v[X]
                0x003 => self.xor_r(opcode),    // XOR register v[Y] with v[X]
                0x004 => self.add_r(opcode),    // Add register v[Y] with v[X]
                0x005 => self.sub_r(opcode),    // Subtract register v[Y] from v[X]
                0x006 => self.shr_r(opcode),    // Shift register v[X] right
                0x007 => self.rsb_r(opcode),    // Subtract register v[X] from v[Y]
                0x00E => self.shl_r(opcode),    // Shift register v[X] left
                _ => self.pc += 2,              // Skip unknown code
            }
            _ => self.pc += 2,                  // Skip unknown code
        }

    }

    /********************************************/
    /*          Instructions/Opcodes            */
    /********************************************/

    // 0x00E0
    // Clear the display implementation
    fn cls(&mut self) {
        self.pc += 2;                           // Increment counter
    }

    // 0x00EE
    // Return from subroutine implementation
    fn ret(&mut self) {
        self.sp -= 1;                                   // Decrepement stack pointer to get to last call
        self.pc = self.stack[self.sp as usize] - 2;     // Return to the memory address of the subroutine call
        self.pc += 2;                                   // Increment counter
    }

    // 1NNN
    // Jump to address implementation
    fn jmp(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;          // Set current memory position to provided address
    }

    // 2NNN
    // Jump to subroutine address NNN
    fn jsr(&mut self, opcode: u16) {
        self.stack[self.sp as usize] = self.pc;     // Set current memory position in the stack
        self.sp += 1;                               // Increment the stack pointer to avoid overwrite
        self.pc = opcode & 0x0FFF;                  // Set current memory position to provided address
    }

    // 3XRR
    // Skip next instruction if register vX == constant RR
    fn skeq_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let rr = (opcode & 0x00FF) as u8;                  // Extract rr constant

        if self.v[x] == rr {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // 4XRR
    // Skip next instruction if register vX != constant RR
    fn skne_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let rr = (opcode & 0x00FF) as u8;                  // Extract rr constant

        if self.v[x] != rr {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // 0x5XY0
    // Skip next instruction if register vX != register vY
    fn skeq_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;      // Extract Y register

        if self.v[x] != self.v[y] {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // 0x6XRR
    // Move constant rr to register vX
    fn mov_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let rr = (opcode & 0x00FF) as u8;                   // Extract rr constant

        self.v[x] = rr;                                         // set vX = rr
        self.pc += 2;                                           // Increment counter
    }

    // 0x7XRR
    // Add constant rr to register vX, no carry generated
    fn add_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let rr = (opcode & 0x00FF) as u8;                   // Extract rr constant

        self.v[x] = self.v[x].wrapping_add(rr);                 // Add rr to vX
        self.pc += 2;                                           // Increment counter
    }

    // 8XY0
    // Move register vY into register vX
    fn mov_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;      // Extract Y register

        self.v[x] = self.v[y];                                  // Set vX = vY
        self.pc += 2;                                           // Increment counter
    }

    // 8XY1
    // OR register vY with register vX, store in vX
    fn or_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;      // Extract Y register

        self.v[x] = self.v[x] | self.v[y];                     // OR registers
        self.pc += 2;                                          // Increment counter
    }

    // 8XY2
    // AND register vY with register vX, store in vX
    fn and_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;      // Extract Y register

        self.v[x] = self.v[x] & self.v[y];                     // AND registers
        self.pc += 2;                                          // Increment counter
    }

    // 8XY3
    // XOR register vY with register vX, store in vX
    fn xor_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;      // Extract Y register

        self.v[x] = self.v[x] ^ self.v[y];                     // XOR registers
        self.pc += 2;                                          // Increment counter
    }

    // 8XY4
    // Add register vY with register vX, store in vX, carry in register vF
    fn add_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;       // Extract Y register

        if (self.v[y]) > (0xFF - self.v[x]) {
            self.v[0xF] = 1;                                    // Carry bit
        }
        else {
            self.v[0xF] = 0;                                    // No carry needed
        }

        self.v[x] = self.v[x].wrapping_add(self.v[y]);          // Add registers
        self.pc += 2;                                           // Increment counter
    }

    // 8XY5
    // Sub register vY from register vX, vF set to 1 if borrows
    fn sub_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;       // Extract Y register

        if self.v[x] >= self.v[y] {
            self.v[0xf] = 0;                                    // No borrow required
        }
        else {
            self.v[0xf] = 1;                                    // Borrow required
        }

        self.v[x] = self.v[x].wrapping_sub(self.v[y]);          // Subtract registers
        self.pc += 2;                                           // Increment counter
    }

    // 8X06
    // Shift register vX right, bit 0 goes into register vF
    fn shr_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        
        self.v[0xF] = self.v[x] & 0x1;                          // Store LSB in Flag register
        self.v[x] >>= 1;                                        // Right shift register vX
        self.pc += 2;                                           // Increment counter
    }

    // 8XY7
    // Sub register vX from register vY, store in vX, vF set to 1 if borrows
    fn rsb_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;       // Extract Y register

        if self.v[y] >= self.v[x] {
            self.v[0xf] = 0;                                    // No borrow required
        }
        else {
            self.v[0xf] = 1;                                    // Borrow required
        }

        self.v[x] = self.v[y].wrapping_sub(self.v[x]);          // Subtract registers
        self.pc += 2;                                           // Increment counter
    }

    // 8X0E
    // Shift register vX left, bit 7 goes into register vF
    fn shl_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        
        self.v[0xF] = (self.v[x] & 0x80) >> 7;                  // Store LSB in Flag register
        self.v[x] <<= 1;                                        // Right shift register vX
        self.pc += 2;                                           // Increment counter
    }
}