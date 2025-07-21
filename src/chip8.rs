use std::{fs::File, io::Read, path::Path};

use rand::Rng;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    display: [bool; 32 * 64],
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut state = Self {
            memory: [0u8; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200, // Leaving the first 512 bytes of memory
            stack: [0; 16],
            sp: 0,
            display: [false; 32 * 64],
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
        };

        state.load_fontset();
        return state;
    }

    fn load_fontset(&mut self) {
        self.memory[0..FONTSET.len()].copy_from_slice(&FONTSET);
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        file.read(&mut self.memory[0x200..])?;
        return Ok(());
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);
        self.update_timers();
    }

    fn fetch_opcode(&self) -> u16 {
        let hi_byte = self.memory[self.pc as usize] as u16;
        let lo_byte = self.memory[self.pc as usize + 1] as u16;
        (hi_byte << 8) | lo_byte
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        // https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
        match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(opcode),
            (0x02, _, _, _) => self.op_2nnn(opcode),
            (0x03, _, _, _) => self.op_3xkk(opcode),
            (0x04, _, _, _) => self.op_4xkk(opcode),
            (0x05, _, _, 0x00) => self.op_5xy0(opcode),
            (0x06, _, _, _) => self.op_6xkk(opcode),
            (0x07, _, _, _) => self.op_7xkk(opcode),
            (0x08, _, _, 0x00) => self.op_8xy0(opcode),
            (0x08, _, _, 0x01) => self.op_8xy1(opcode),
            (0x08, _, _, 0x02) => self.op_8xy2(opcode),
            (0x08, _, _, 0x03) => self.op_8xy3(opcode),
            (0x08, _, _, 0x04) => self.op_8xy4(opcode),
            (0x08, _, _, 0x05) => self.op_8xy5(opcode),
            (0x08, _, _, 0x06) => self.op_8xy6(opcode),
            (0x08, _, _, 0x07) => self.op_8xy7(opcode),
            (0x08, _, _, 0x0e) => self.op_8xye(opcode),
            (0x09, _, _, 0x00) => self.op_9xy0(opcode),
            (0x0a, _, _, _) => self.op_annn(opcode),
            (0x0b, _, _, _) => self.op_bnnn(opcode),
            (0x0c, _, _, _) => self.op_cxkk(opcode),
            (0x0d, _, _, _) => self.op_dxyn(opcode),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(opcode),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(opcode),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(opcode),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(opcode),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(opcode),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(opcode),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(opcode),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(opcode),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(opcode),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(opcode),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(opcode),
            // If no match, move to next instruction
            _ => self.pc += 2,
        }
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    /// Clears the display
    fn op_00e0(&mut self) {
        self.display.fill(false);
        self.pc += 2;
    }

    /// Returns from subroutine
    /// Decrements the stack pointer and sets the program counter to the
    /// return address on the stack
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize] + 2;
    }

    /// Jumps to nnn address
    fn op_1nnn(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    /// Calls the subroutine at nnn address
    fn op_2nnn(&mut self, opcode: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = opcode & 0x0FFF;
    }

    /// Skips the next instruction if the value of Vx == kk
    fn op_3xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Skips the next instruction if the value of Vx != kk
    fn op_4xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Skips the next instruction if Vx == Vy
    fn op_5xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Sets Vx to kk
    fn op_6xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = kk;
        self.pc += 2;
    }

    /// Adds kk to Vx
    fn op_7xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] += kk;
        self.pc += 2;
    }

    /// Sets Vx to Vy
    fn op_8xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] = self.v[y];
        self.pc += 2;
    }

    /// Sets Vx to Vx | Vy
    fn op_8xy1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    /// Sets Vx to Vx & Vy
    fn op_8xy2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    /// Sets Vx to Vx ^ Vy
    fn op_8xy3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] ^= self.v[y];
        self.pc += 2;
    }

    /// Adds Vy to Vx and sets Vf to 1 if an overflow occurs
    fn op_8xy4(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = sum;
        self.v[0xF] = if carry { 1 } else { 0 };
        self.pc += 2;
    }

    /// Subtracts Vy from Vx and sets Vf to 0 if an underflow occurs, else 1
    fn op_8xy5(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, carry) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = if carry { 0 } else { 1 };
        self.pc += 2;
    }

    /// Shifts Vx to the right by 1, storing its least significant bit in
    /// Vf before the shift
    fn op_8xy6(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] >>= 1;
        self.pc += 2;
    }

    /// Sets Vx = Vy - Vx and sets Vf to 0 if an underflow occurs, else 1
    fn op_8xy7(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, carry) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[0xF] = if carry { 0 } else { 1 };
        self.pc += 2;
    }

    /// Shifts Vx to the left by 1. If its most significant bit before the shift
    /// was set, sets Vf to 1, else 0
    fn op_8xye(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
        self.pc += 2;
    }

    /// Skips the next instruction if Vx != Vy
    fn op_9xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] != self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Set I to the nnn address
    fn op_annn(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.i = nnn;
        self.pc += 2;
    }

    /// Jumps to the nnn address plus V0
    fn op_bnnn(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.pc = nnn + self.v[0] as u16;
    }

    /// Sets Vx to the result of kk & random number
    fn op_cxkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let r: u8 = rand::rng().random();
        self.v[x] = r & kk;
        self.pc += 2;
    }

    /// Draws a sprite to the screen at (Vx, Vy), with a width of
    /// 8 pixels and a height of n pixels.
    /// Sets Vf to 1 when there is a collision with existing screen pixels, or
    /// it sets it to 0 if there isn't.
    fn op_dxyn(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let height = (opcode & 0x00F) as usize;

        let vx = self.v[x] as usize;
        let vy = self.v[y] as usize;

        self.v[0xF] = 0;

        for row in 0..height {
            let sprite = self.memory[self.i as usize + row];
            for col in 0..8 {
                if (sprite & (0x80 >> col)) != 0 {
                    let pixel_index = (vx + col + (vy + row) * 64) % (32 * 64);
                    if self.display[pixel_index] {
                        self.v[0xF] = 1;
                    }
                    self.display[pixel_index] ^= true;
                }
            }
        }
    }

    /// Skips the next instruction if the key stored in Vx is pressed
    fn op_ex9e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;
        if self.keypad[key] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Skips the next instruction if the key stored in Vx is not pressed
    fn op_exa1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let key = self.v[x] as usize;
        if !self.keypad[key] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    /// Sets Vx to the value of delay timer
    fn op_fx07(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.delay_timer;
        self.pc += 2;
    }

    /// Waits for a key press and stores the value in Vx
    fn op_fx0a(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        // Returns the index of the first key inside keypad that is pressed
        let key_pressed = self.keypad.iter().position(|&k| k);

        match key_pressed {
            Some(key) => {
                self.v[x] = key as u8;
                self.pc += 2;
            }
            None => self.pc -= 2, // Run the same instruction again until something is pressed
        }
    }

    /// Sets the delay timer to the value of Vx
    fn op_fx15(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.delay_timer = self.v[x];
        self.pc += 2;
    }

    /// Sets the sound timer to the value of Vx
    fn op_fx18(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.sound_timer = self.v[x];
        self.pc += 2;
    }

    /// Adds Vx to I
    fn op_fx1e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i += self.v[x] as u16;
        self.pc += 2;
    }

    /// Sets I to the location of the sprite for the character in Vx
    fn op_fx29(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i += self.v[x] as u16 * 5;
        self.pc += 2;
    }

    ///
    fn op_fx33(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let vx = self.v[x];

        self.memory[self.i as usize] = vx / 100;
        self.memory[(self.i + 1) as usize] = (vx % 100) / 10;
        self.memory[(self.i + 2) as usize] = vx % 10;
        self.pc += 2;
    }

    /// Stores all registers from 0 to x (inclusive) starting at the address
    /// of I
    fn op_fx55(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        for index in 0..=x {
            self.memory[self.i as usize + index] = self.v[index];
        }
        self.pc += 2;
    }

    /// Fills registers V0 to Vx (inclusive) from memory starting at the address
    /// of I
    fn op_fx65(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        for index in 0..=x {
            self.v[index] = self.memory[self.i as usize + index];
        }
        self.pc += 2;
    }
}
