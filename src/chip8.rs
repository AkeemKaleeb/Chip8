use rand::Rng;
use std::fs::File;
use std::io::Read;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

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
    v: [u8; 16],                    // General Purpose Registers v0 - vF
    index: u16,                     // Index Register
    pc: u16,                        // Program Counter
    sp: u16,                        // Stack Pointer
    stack: [u16; 16],               // Stack
    memory: [u8; 4096],             // Memory
    delay_timer: u8,                // Delay Timer
    sound_timer: u8,                // Sound Timer
    opcode: u16,                    // Program Opperation Code
    pub display: [u8; WIDTH * HEIGHT],  // Display
    key:[u8; 16],                   // Input keys
    pub draw_flag: bool,            // Determine whether or not to update screen
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
            display: [0; WIDTH * HEIGHT],
            key: [0; 16],
            draw_flag: false,
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
    pub fn cycle(&mut self) {
        self.opcode = self.fetch_opcode();  // Fetch
        self.decode_execute(self.opcode);   // Decode and Execute

        if self.delay_timer > 0 {           // Update delay timer
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {           // Update sound timer
            self.sound_timer -= 1;
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
            0x3000 => self.skeq_c(opcode),      // Skip next instruction if v[x] == NN
            0x4000 => self.skne_c(opcode),      // Skip next instruction if v[X] != NN
            0x5000 => self.skeq_r(opcode),      // Skip next instruction if v[X] == v[Y]
            0x6000 => self.mov_c(opcode),       // Move constant NN to v[X]
            0x7000 => self.add_c(opcode),       // Add constant NN to v[X]
            0x8000 => match opcode & 0x000F {
                0x000 => self.mov_r(opcode),    // Move v[Y] into v[X]
                0x001 => self.or_r(opcode),     // OR v[Y] with v[X]
                0x002 => self.and_r(opcode),    // AND v[Y] with v[X]
                0x003 => self.xor_r(opcode),    // XOR v[Y] with v[X]
                0x004 => self.add_r(opcode),    // Add v[Y] with v[X]
                0x005 => self.sub_r(opcode),    // Subtract v[Y] from v[X]
                0x006 => self.shr_r(opcode),    // Shift v[X] right
                0x007 => self.rsb_r(opcode),    // Subtract v[X] from v[Y]
                0x00E => self.shl_r(opcode),    // Shift v[X] left
                _ => self.pc += 2,              // Skip unknown code
            }
            0x9000 => self.skne_r(opcode),      // Skip next instruction if v[X] != v[Y]
            0xA000 => self.mvi(opcode),         // Move constant NNN to I
            0xB000 => self.jmi(opcode),         // Jump to address NNN + v[0]
            0xC000 => self.rand(opcode),        // Set v[X] = rand AND NN
            0xD000 => self.sprite(opcode),      // Draw sprite at (v[X], v[Y]), height N
            0xE000 => match opcode & 0x000F {
                0x000E => self.skpr(opcode),    // Skip next instruction if key rX is pressed
                0x0001 => self.skup(opcode),    // Skip next instruction if key rX is not pressed
                _ => self.pc += 2,              // Skip unknown code
            }
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.gdelay(opcode),  // Get delay timer into vX
                0x000a => self.key(opcode),     // Wait for keypress and store in vX
                0x0015 => self.sdelay(opcode),  // Set delay timer to vX
                0x0018 => self.ssound(opcode),  // Set sound timer to vX
                0x001e => self.adi(opcode),     // Add vX to I
                0x0029 => self.font(opcode),    // Point I to the sprite for hexadecimal character vX
                0x0033 => self.bcd(opcode),     // Store bcd of vX at I, I+1, I+2
                0x0055 => self.str(opcode),     // Store v0 - vX at I incremented each time
                0x0065 => self.ldr(opcode),     // Load registers v0 - vX from I incremented each time
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
        self.pc += 2;                       // Increment counter
    }

    // 0x00EE
    // Return from subroutine implementation
    fn ret(&mut self) {
        self.sp -= 1;                                   // Decrepement stack pointer to get to last call
        self.pc = self.stack[self.sp as usize] - 2;     // Return to the memory address of the subroutine call
        self.pc += 4;                                   // Increment counter
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

    // 3XNN
    // Skip next instruction if register vX == constant NN
    fn skeq_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let nn = (opcode & 0x00FF) as u8;                  // Extract NN constant

        if self.v[x] == nn {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // 4XNN
    // Skip next instruction if register vX != constant NN
    fn skne_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let nn = (opcode & 0x00FF) as u8;                  // Extract NN constant

        if self.v[x] != nn {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // 0x5XY0
    // Skip next instruction if register vX == register vY
    fn skeq_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;      // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;      // Extract Y register

        if self.v[x] == self.v[y] {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // 0x6XNN
    // Move constant NN to register vX
    fn mov_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let nn = (opcode & 0x00FF) as u8;                   // Extract NN constant

        self.v[x] = nn;                                         // set vX = NN
        self.pc += 2;                                           // Increment counter
    }

    // 0x7XNN
    // Add constant NN to register vX, no carry generated
    fn add_c(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let nn = (opcode & 0x00FF) as u8;                   // Extract NN constant

        self.v[x] = self.v[x].wrapping_add(nn);                 // Add NN to vX
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

        let (result, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = carry as u8;

        self.pc += 2;                                           // Increment counter
    }

    // 8XY5
    // Sub register vY from register vX, vF set to 1 if borrows
    fn sub_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;       // Extract Y register
        let vx = self.v[x] as usize;                    // Extract X register
        let vy = self.v[y] as usize;                    // Extract Y register

        self.v[x] = self.v[x].wrapping_sub(self.v[y]);

        if vx >= vy {
            self.v[0xF] = 1; // No borrow needed
        } else {
            self.v[0xF] = 0; // Borrow occurred
        }
    

        self.pc += 2;                                           // Increment counter
    }

    // 8X06
    // Shift register vX right, bit 0 goes into register vF
    fn shr_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let lsb = self.v[x] & 0x1;

        self.v[x] >>= 1;                                        // Right shift register vX
        self.v[0xF] = lsb;                                      // Store LSB in Flag register
        self.pc += 2;                                           // Increment counter
    }

    // 8XY7
    // Sub register vX from register vY, store in vX, vF set to 1 if borrows
    fn rsb_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;       // Extract Y register
        let vx = self.v[x] as usize;                    // Extract X register
        let vy = self.v[y] as usize;                    // Extract Y register

        self.v[x] = self.v[y].wrapping_sub(self.v[x]);

        if vy >= vx {
            self.v[0xF] = 1; // No borrow needed
        } else {
            self.v[0xF] = 0; // Borrow occurred
        }
        
        self.pc += 2;                                           // Increment counter
    }

    // 8X0E
    // Shift register vX left, bit 7 goes into register vF
    fn shl_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let msb = (self.v[x] & 0x80) >> 7;

        self.v[x] <<= 1;                                        // Right shift register vX
        self.v[0xF] = msb;                                      // Store LSB in Flag register
        self.pc += 2;                                           // Increment counter
    }

    // 9XY0
    // Skip next instruction if register vX != register vY
    fn skne_r(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let y = ((opcode & 0x00F0) >> 4) as usize;       // Extract Y register

        if self.v[x] != self.v[y] {
            self.pc += 2;                                      // Increment program counter by 2 = skip next instruction
        }
        self.pc += 2;                                          // Increment counter
    }

    // ANNN
    // Load index register I with constant NNN
    fn mvi(&mut self, opcode: u16) {
        let nnn = (opcode & 0x0FFF) as u16;    // Extract NNN constant

        self.index = nnn;                           // Set index register to constant
        self.pc += 2;
    }

    // BNNN
    // Jump to address NNN + register v0
    fn jmi(&mut self, opcode: u16) {
        let nnn = (opcode & 0x0FFF) as u8;      // Extract NNN constant

        self.pc = (nnn + self.v[0]) as u16;         // Point program counter to new address
    }

    // CXNN
    // Set register vX to a random number AND NN
    fn rand(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register
        let nn = (opcode & 0x00FF) as u8;                   // Extract NN constant
        let mut rng = rand::thread_rng();            // Create random generator

        self.v[x] = rng.gen::<u8>() & nn;                       // Set X register to random number AND nn
    }

    // DXYN
    // Draw a sprite at screen location (vX, vY) height N
    fn sprite(&mut self, opcode: u16) {
        let vx = self.v[((opcode & 0x0F00) >> 8) as usize] as usize; // Extract X register
        let vy = self.v[((opcode & 0x00F0) >> 4) as usize] as usize; // Extract Y register
        let height: usize = (opcode & 0x000F) as usize;                     // Extract height

        self.v[0xF] = 0;                                                    // Reset flag register

        // Loop through line by line and update display map
        for yline in 0..height {
            let pixel = self.memory[self.index as usize + yline];
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    let x_pos = (vx + xline) % 64;
                    let y_pos = (vy + yline) % 32;
                    let idx = x_pos + (y_pos * 64);
                    if self.display[idx] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.display[idx] ^= 1;
                }
            }
        }

        self.draw_flag = true;                                  // Update screen needs redrawing
        self.pc += 2;
    }

    // EX9E
    // Skip if key rX is pressed
    fn skpr(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        if (self.key[self.v[x] as usize]) != 0 {
            self.pc += 2;                                       // Skip next instruction
        }

        self.pc += 2;
    }

    // EXA1
    // Skip if key rX is not pressed
    fn skup(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        if (self.key[self.v[x] as usize]) == 0 {
            self.pc += 2;                                       // Skip next instruction
        }

        self.pc += 2;
    }

    // FX07
    // Get delay timer into vX
    fn gdelay(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        self.v[x] = self.delay_timer;                           // Load register X with delay timer
        self.pc += 2;
    }

    // FX0A
    // Wait for keypress, put key in register vX
    fn key(&mut self, _opcode: u16) {

    }

    // FX15
    // Set the delay timer to vX
    fn sdelay(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        self.v[x] = self.sound_timer;                           // Load register X with sound timer
        self.pc += 2;
    }

    // FX18
    // Set the sound timer to vX
    fn ssound(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        self.sound_timer = self.v[x];                           // Load register X with sound timer
        self.pc += 2;
    }

    // FX1E
    // Add register vX to the index register I
    fn adi(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        self.index += self.v[x] as u16;                         // Add vX to index
        self.pc += 2;
    }

    // FX29
    // Point I to the sprite for hexadecimal character in vX
    fn font(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;       // Extract X register

        self.index = (0x50 + (self.v[x] * 5)) as u16;
        self.pc += 2;
    }

    // FX33
    // Store the bcd representation of register vX at location I, I+1, I+2
    fn bcd(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;              // Extract X register
        
        self.memory[self.index as usize] = self.v[x] / 100;             // Get hundreds location
        self.memory[self.index as usize + 1] = (self.v[x] / 10) % 10;   // Get tens location
        self.memory[self.index as usize + 2] = (self.v[x] % 100) % 10;  // Get ones location

        self.pc += 2;
    }

    // FX55
    // Store registers v0-vX at location I onwards, incrementing I to the next location each time
    fn str(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;              // Extract X register

        for i in 0..=x {
            self.memory[self.index as usize + i] = self.v[i];
        }

        self.pc += 2;
    }

    // FX65
    // Load registers v0 to vX from location I onwards, incrementing I to the next location each time
    fn ldr(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;              // Extract X register

        for i in 0..=x {
            self.v[i] = self.memory[self.index as usize + i];
        }

        self.pc += 2;
    }
}