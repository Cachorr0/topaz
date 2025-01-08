use std::{
    time::Duration,
};
use rand::Rng;
use bitvec::prelude::*;

pub struct Display {
    pub buffer: [bool; 64 * 32],
}

impl Display {
    pub fn new() -> Self {
        Self { buffer: [false; 64 * 32] }
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: (&[u8], usize)) {
        let height = sprite.1.min(0xf);

        for row in 0..height {
            let y_pos = (y as usize + row) % 32; // Wrap vertically

            for bit in 0..8 {
                let x_pos = (x as usize + bit) % 64; // Wrap horizontally
                let pixel = (sprite.0[row] >> (7 - bit)) & 1 != 0;
                let pos = y_pos * 64 + x_pos;
                self.buffer[pos] ^= pixel;
            }
        }
    }

    pub fn get_pixel(&mut self, x: u8, y: u8) -> bool {
        self.buffer[y as usize * 64 + x as usize]
    }
}

pub struct Stack {
    pub stack: [u16; 16],
    pub sp: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: [0; 16],
            sp: 0,
        }
    }

    pub fn push(&mut self, val: u16) -> Result<(), &'static str> {
        if self.sp >= self.stack.len() {
            return Err("Stack overflow");
        }
        // println!("Pushing {:04X} onto stack at position {}", val, self.sp);
        self.stack[self.sp] = val;
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u16, &'static str> {
        if self.sp == 0 {
            return Err("Stack underflow");
        }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }

    pub fn peek(&self) -> Option<u16> {
        if self.sp == 0 {
            None
        } else {
            Some(self.stack[self.sp - 1])
        }
    }
}


pub struct Registers {
    pub v: [u8; 16], // General purpose
    pub i: u16,      // Index register
    pub dt: u8,      // Delay timer
    pub st: u8,      // Sound timer
    pub pc: u16,     // Program counter

}

impl Registers {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200, // Entry point for most programs
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Cls_00E0,
    Ret_00EE,
    Jp_1nnn,
    Call_2nnn,
    Se_3xkk,
    Sne_4xkk,
    Se_5xy0,
    Ld_6xkk,
    Add_7xkk,
    Ld_8xy0,
    Or_8xy1,
    And_8xy2,
    Xor_8xy3,
    Add_8xy4,
    Sub_8xy5,
    Shr_8xy6,
    Subn_8xy7,
    Shl_8xyE,
    Sne_9xy0,
    Ld_Annn,
    Jp_Bnnn,
    Rnd_Cxkk,
    Drw_Dxyn,
    Skp_Ex9E,
    Sknp_ExA1,
    Ld_Fx07,
    Ld_Fx0A,
    Ld_Fx15,
    Ld_Fx18,
    Add_Fx1E,
    Ld_Fx29,
    Ld_Fx33,
    Ld_Fx55,
    Ld_Fx65,
    Huh, // Unknown
}

pub struct Processor {
    pub registers: Registers,
    pub memory: [u8; 4096],
    pub stack: Stack,

}

impl Processor {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            memory: [0; 4096],
            stack: Stack::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let position = self.registers.pc as usize;
            self.registers.pc += 2;

            let instruction_bytes = [self.memory[position], self.memory[position + 1]];
            let instruction = u16::from_be_bytes(instruction_bytes);

            if instruction == 0x0000 {
                break;
            }

            self.do_instruction(instruction);


            std::thread::sleep(Duration::from_secs_f32(0.2));
        }
    }

    pub fn do_instruction(&mut self, instruction: u16) {
        let decoded = self.parse_instruction(instruction);

        let x = ((instruction & 0x0f00) >> 8) as usize;
        let y = ((instruction & 0x00f0) >> 4) as usize;

        match decoded {
            Instruction::Cls_00E0 => {}
            Instruction::Ret_00EE => {
                let adr = self.stack.pop();
                self.registers.pc = adr.unwrap();
            }
            Instruction::Jp_1nnn => {
                self.registers.pc = instruction & 0x0fff;
            }
            Instruction::Call_2nnn => {
                self.stack.push(self.registers.pc).unwrap();
                self.registers.pc = instruction & 0x0fff;
            }
            Instruction::Se_3xkk => {
                let register = self.registers.v[((instruction & 0x0f00) >> 8) as usize];
                let val = instruction & 0x00ff;
                if self.registers.v[register as usize] == val as u8 {
                    self.registers.pc += 2;
                }
            }
            Instruction::Sne_4xkk => {
                let register = self.registers.v[((instruction & 0x0f00) >> 8) as usize];
                let val = instruction & 0x00ff;
                if self.registers.v[register as usize] != val as u8 {
                    self.registers.pc += 2;
                }
            }
            Instruction::Se_5xy0 => {
                let register_x = self.registers.v[x];
                let register_y = self.registers.v[y];
                if self.registers.v[register_x as usize] == self.registers.v[register_y as usize] {
                    self.registers.pc += 2;
                }
            }
            Instruction::Ld_6xkk => {
                self.registers.v[x] = (instruction & 0x00ff) as u8;
            }
            Instruction::Add_7xkk => {
                self.registers.v[x] += (instruction & 0x00ff) as u8;
            }
            Instruction::Ld_8xy0 => {
                self.registers.v[x] = self.registers.v[y];
            }
            Instruction::Or_8xy1 => {
                self.registers.v[x] = self.registers.v[x] | self.registers.v[y];
            }
            Instruction::And_8xy2 => {
                self.registers.v[x] = self.registers.v[x] & self.registers.v[y];
            }
            Instruction::Xor_8xy3 => {
                self.registers.v[x] = self.registers.v[x] ^ self.registers.v[y];
            }
            Instruction::Add_8xy4 => {
                let sum = self.registers.v[x] as u16 + self.registers.v[y] as u16;
                self.registers.v[0xF] = if sum > 255 { 1 } else { 0 };
                self.registers.v[x] = self.registers.v[x].wrapping_sub(self.registers.v[y]);
            }
            Instruction::Sub_8xy5 => {
                self.registers.v[0xF] = if self.registers.v[x] > self.registers.v[y] { 1 } else { 0 };
                self.registers.v[x] = self.registers.v[x].wrapping_sub(self.registers.v[y]);
            }
            Instruction::Shr_8xy6 => {
                self.registers.v[0xF] = self.registers.v[x] & 1;
                self.registers.v[x] >>= 1;
            }
            Instruction::Subn_8xy7 => {
                self.registers.v[0xF] = if self.registers.v[y] > self.registers.v[x] { 1 } else { 0 };
                self.registers.v[x] = self.registers.v[y].wrapping_sub(self.registers.v[x]);
            }
            Instruction::Shl_8xyE => {
                self.registers.v[0xF] = (self.registers.v[x] >> 7) & 1;
                self.registers.v[x] <<= 1;
            }
            Instruction::Sne_9xy0 => {
                if self.registers.v[x] != self.registers.v[y] {
                    self.registers.pc += 2;
                }
            }
            Instruction::Ld_Annn => {
                self.registers.i = instruction & 0x0fff;
            }
            Instruction::Jp_Bnnn => {
                self.registers.pc = (instruction & 0x0fff) + self.registers.v[0x0] as u16;
            }
            Instruction::Rnd_Cxkk => {
                let kk = (instruction & 0x00ff) as u8;
                self.registers.v[x] = rand::random::<u8>() & kk;
            }
            Instruction::Drw_Dxyn => {}
            Instruction::Skp_Ex9E => {}
            Instruction::Sknp_ExA1 => {}
            Instruction::Ld_Fx07 => {
                self.registers.v[x] = self.registers.dt;
            }
            Instruction::Ld_Fx0A => {}
            Instruction::Ld_Fx15 => {
                self.registers.dt = self.registers.v[x];
            }
            Instruction::Ld_Fx18 => {
                self.registers.st = self.registers.v[x];
            }
            Instruction::Add_Fx1E => {
                self.registers.i += self.registers.v[x] as u16;
            }
            Instruction::Ld_Fx29 => {}
            Instruction::Ld_Fx33 => {}
            Instruction::Ld_Fx55 => {
                let i = self.registers.i as usize;

                // Store registers V0 through Vx into memory starting at I
                for offset in 0..=x {
                    self.memory[i + offset] = self.registers.v[offset];
                }
            }
            Instruction::Ld_Fx65 => {
                let i = self.registers.i as usize;

                // Load registers V0 through Vx from memory starting at I
                for offset in 0..=x {
                    self.registers.v[offset] = self.memory[i + offset];
                }
            }
            Instruction::Huh => {}
        }

        println!("Sp: {}\t{:04X}\t{:?}\t{:?}", self.stack.sp, instruction, self.registers.v, decoded);
    }

    pub fn parse_instruction(&mut self, instruction: u16) -> Instruction {
        match instruction >> 12 {
            0x0 => match instruction {
                0x00E0 => Instruction::Cls_00E0,
                0x00EE => Instruction::Ret_00EE,
                _ => Instruction::Huh,
            },
            0x1 => Instruction::Jp_1nnn,
            0x2 => Instruction::Call_2nnn,
            0x3 => Instruction::Se_3xkk,
            0x4 => Instruction::Sne_4xkk,
            0x5 => Instruction::Se_5xy0,
            0x6 => Instruction::Ld_6xkk,
            0x7 => Instruction::Add_7xkk,
            0x8 => match instruction & 0x000f {
                0x0 => Instruction::Ld_8xy0,
                0x1 => Instruction::Or_8xy1,
                0x2 => Instruction::And_8xy2,
                0x3 => Instruction::Xor_8xy3,
                0x4 => Instruction::Add_8xy4,
                0x5 => Instruction::Sub_8xy5,
                0x6 => Instruction::Shr_8xy6,
                0x7 => Instruction::Subn_8xy7,
                0xE => Instruction::Shl_8xyE,
                _ => Instruction::Huh,
            },
            0x9 => Instruction::Sne_9xy0,
            0xA => Instruction::Ld_Annn,
            0xB => Instruction::Jp_Bnnn,
            0xC => Instruction::Rnd_Cxkk,
            0xD => Instruction::Drw_Dxyn,
            0xE => match instruction & 0x000F {
                0xE => Instruction::Skp_Ex9E,
                0x1 => Instruction::Sknp_ExA1,
                _ => Instruction::Huh,
            },
            0xF => match instruction & 0x00ff {
                0x07 => Instruction::Ld_Fx07,
                0x0A => Instruction::Ld_Fx0A,
                0x15 => Instruction::Ld_Fx15,
                0x18 => Instruction::Ld_Fx18,
                0x1E => Instruction::Add_Fx1E,
                0x29 => Instruction::Ld_Fx29,
                0x33 => Instruction::Ld_Fx33,
                0x55 => Instruction::Ld_Fx55,
                0x65 => Instruction::Ld_Fx65,
                _ => Instruction::Huh,
            },
            _ => Instruction::Huh,
        }
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        let start = 0x200;
        let end = start + data.len();
        // println!("Boutta load {} bytes", end - start);

        // Ensure we don't exceed memory bounds
        if end > self.memory.len() {
            panic!("ROM is too large to fit in memory!");
        }

        self.memory[start..end].copy_from_slice(&data);
    }
}
