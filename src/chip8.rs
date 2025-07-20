use std::{fs::File, io::Read, path::Path};

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

    fn op_2nnn(&self, opcode: u16) {
        todo!()
    }

    fn op_3xkk(&self, opcode: u16) {
        todo!()
    }

    fn op_4xkk(&self, opcode: u16) {
        todo!()
    }

    fn op_5xy0(&self, opcode: u16) {
        todo!()
    }

    fn op_6xkk(&self, opcode: u16) {
        todo!()
    }

    fn op_7xkk(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy0(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy1(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy2(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy3(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy4(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy5(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy6(&self, opcode: u16) {
        todo!()
    }

    fn op_8xy7(&self, opcode: u16) {
        todo!()
    }

    fn op_8xye(&self, opcode: u16) {
        todo!()
    }

    fn op_9xy0(&self, opcode: u16) {
        todo!()
    }

    fn op_annn(&self, opcode: u16) {
        todo!()
    }

    fn op_bnnn(&self, opcode: u16) {
        todo!()
    }

    fn op_cxkk(&self, opcode: u16) {
        todo!()
    }

    fn op_dxyn(&self, opcode: u16) {
        todo!()
    }

    fn op_ex9e(&self, opcode: u16) {
        todo!()
    }

    fn op_exa1(&self, opcode: u16) {
        todo!()
    }

    fn op_fx07(&self, opcode: u16) {
        todo!()
    }

    fn op_fx0a(&self, opcode: u16) {
        todo!()
    }

    fn op_fx15(&self, opcode: u16) {
        todo!()
    }

    fn op_fx18(&self, opcode: u16) {
        todo!()
    }

    fn op_fx1e(&self, opcode: u16) {
        todo!()
    }

    fn op_fx29(&self, opcode: u16) {
        todo!()
    }

    fn op_fx33(&self, opcode: u16) {
        todo!()
    }

    fn op_fx55(&self, opcode: u16) {
        todo!()
    }

    fn op_fx65(&self, opcode: u16) {
        todo!()
    }
}
