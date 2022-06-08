/**
 * Copyright (c) 2022 Oliver Lau <oliver@ersatzworld.net>
 * All rights reserved.
 */

use std::fs::File;
use std::io::Read;
use std::convert::TryInto;

pub mod error;
pub mod opcode;

use error::Error;
use opcode::Opcode;

pub enum ComparisonResult {
    None,
    LessThan,
    Equal,
    GreaterThan,
}

const SCREEN_HEIGHT: usize = 24;
const SCREEN_WIDTH: usize = 80;
const STACK_SIZE: usize = 1000;
const MAX_REGISTERS: usize = 16;

type Registers = [i16; MAX_REGISTERS];

pub struct State {
    pub r: Registers,
    pub pc: usize,
}

pub struct Machine {
    pub pc: usize,
    pub r: Registers,
    pub carry: bool,
    pub cmp: ComparisonResult,
    pub mem: [i16;std::u16::MAX as usize],
    pub code: Vec<u8>,
    pub screen: Vec<u8>,
    pub stack: Vec<State>,
}

impl Machine {
    
    pub fn new() -> Self {
        Machine {
            pc: 0x0000,
            r: [0x0000; 16],
            carry: false,
            cmp: ComparisonResult::None,
            mem: [0x0000; std::u16::MAX as usize],
            code: Vec::new(),
            screen: vec![0x20; SCREEN_HEIGHT * SCREEN_WIDTH],
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, pc: usize) -> Result<(), Error>{
        if self.stack.len() >= STACK_SIZE {
            return Err(Error::StackOverflow);
        }
        self.stack.push(State {
            r: self.r,
            pc: pc
        });
        Ok(())
    }

    pub fn pop(&mut self) -> Result<State, Error> {
        match self.stack.pop() {
            Some(state) => Ok(state),
            None => Err(Error::StackUnderflow),
        }
    }

    pub fn load(&mut self, filename: &String) -> Result<(), Error> {
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => return Err(Error::FileNotFound(e.to_string())),
        };
        let metadata = match std::fs::metadata(&filename) {
            Ok(metadata) => metadata,
            Err(e) => return Err(Error::CannotReadFileMetadata(e.to_string())),
        };
        self.code = vec![0; metadata.len() as usize];
        match f.read(&mut self.code) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::ObjectFileTooLarge(metadata.len() as usize)),
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.step() {
                Ok(true) => (),
                Ok(false) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, Error> {
        let opcode: Opcode = unsafe { std::mem::transmute(self.code[self.pc]) };
        #[allow(unreachable_patterns)]
        match opcode {
            Opcode::CpRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                self.r[rd as usize] = self.r[rs as usize];
                self.pc += 2;
            },
            Opcode::CpRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rd as usize] = v;
                self.pc += 4;
            },
            Opcode::CpMemR => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                let rs = self.code[self.pc+3] & 0x0f;
                self.mem[addr as usize] = self.r[rs as usize];
                self.pc += 4;
            },
            Opcode::CpRMem => {
                let rs = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rs as usize] = self.mem[addr as usize];
                self.pc += 4;
            },
            Opcode::AddRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_add(self.r[rs as usize]);
                self.pc += 2;
            },
            Opcode::AddRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_add(v);
                self.pc += 4;
            },
            Opcode::AddRMem => {
                let rd = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_add(self.mem[addr as usize]);
                self.pc += 4;
            },
            Opcode::SubRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_sub(self.r[rs as usize]);
                self.pc += 2;
            },
            Opcode::SubRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_sub(v);
                self.pc += 4;
            },
            Opcode::SubRMem => {
                let rs = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                (self.r[rs as usize], self.carry) = self.r[rs as usize].overflowing_sub(self.mem[addr as usize]);
                self.pc += 4;
            },
            Opcode::MulRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_mul(self.r[rs as usize]);
                self.pc += 2;
            },
            Opcode::MulRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_mul(v);
                self.pc += 4;
            },
            Opcode::MulRMem => {
                let rd = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_mul(self.mem[addr as usize]);
                self.pc += 4;
            },
            Opcode::DivRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                let v = self.r[rs as usize];
                if v == 0 {
                    return Err(Error::DivisionByZero);
                }
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_div(v);
                self.pc += 2;
            },
            Opcode::DivRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                if v == 0 {
                    return Err(Error::DivisionByZero);
                }
                (self.r[rd as usize], self.carry) = self.r[rd as usize].overflowing_div(v);
                self.pc += 4;
            },
            Opcode::DivRMem => {
                let rs = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                let v = self.mem[addr as usize];
                if v == 0 {
                    return Err(Error::DivisionByZero);
                }
                (self.r[rs as usize], self.carry) = self.r[rs as usize].overflowing_div(v);
                self.pc += 4;
            },
            Opcode::NegR => {
                let rd = self.code[self.pc+1] & 0x0f;
                self.r[rd as usize] = -self.r[rd as usize];
                self.pc += 2;
            },
            Opcode::NegMem => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                self.mem[addr as usize] = -self.mem[addr as usize];
                self.pc += 3;
            },
            Opcode::XorRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                self.r[rd as usize] ^= self.r[rs as usize];
                self.pc += 2;
            },
            Opcode::XorRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rd as usize] ^= v;
                self.pc += 4;
            },
            Opcode::XorRMem => {
                let rs = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rs as usize] ^= self.mem[addr as usize];
                self.pc += 4;
            },
            Opcode::AndRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                self.r[rd as usize] &= self.r[rs as usize];
                self.pc += 2;
            },
            Opcode::AndRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rd as usize] &= v;
                self.pc += 4;
            },
            Opcode::AndRMem => {
                let rs = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rs as usize] &= self.mem[addr as usize];
                self.pc += 4;
            },
            Opcode::OrRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                self.r[rd as usize] |= self.r[rs as usize];
                self.pc += 2;
            },
            Opcode::OrRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rd as usize] |= v;
                self.pc += 4;
            },
            Opcode::OrRMem => {
                let rs = self.code[self.pc+1] & 0x0f;
                let addr = u16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rs as usize] |= self.mem[addr as usize];
                self.pc += 4;
            },
            Opcode::NotR => {
                let rd = self.code[self.pc+1] & 0x0f;
                self.r[rd as usize] = !self.r[rd as usize];
                self.pc += 2;
            },
            Opcode::NotMem => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                self.mem[addr as usize] = !self.mem[addr as usize];
                self.pc += 3;
            },
            Opcode::ShrRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                self.r[rd as usize] >>= self.r[rs as usize];
                self.pc += 2;
            },
            Opcode::ShrRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rd as usize] >>= v;
                self.pc += 4;
            },
            Opcode::ShlRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                self.r[rd as usize] <<= self.r[rs as usize];
                self.pc += 2;
            },
            Opcode::ShlRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let v = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                self.r[rd as usize] <<= v;
                self.pc += 4;
            },
            Opcode::CmpRR => {
                let op = self.code[self.pc+1];
                let rd = op >> 4;
                let rs = op & 0x0f;
                let a = self.r[rd as usize];
                let b = self.r[rs as usize];
                if a < b {
                    self.cmp = ComparisonResult::LessThan;
                }
                else if a > b {
                    self.cmp = ComparisonResult::GreaterThan;
                }
                else {
                    self.cmp = ComparisonResult::Equal;
                }
                self.pc += 2;
            },
            Opcode::CmpRImm => {
                let rd = self.code[self.pc+1] & 0x0f;
                let a = self.r[rd as usize];
                let b = i16::from_le_bytes(self.code[self.pc+2..self.pc+4].try_into().expect("slice has incorrect length"));
                if a < b {
                    self.cmp = ComparisonResult::LessThan;
                }
                else if a > b {
                    self.cmp = ComparisonResult::GreaterThan;
                }
                else {
                    self.cmp = ComparisonResult::Equal;
                }
                self.pc += 4;
                
            },
            Opcode::CmpRMem => {
                let rd = self.code[self.pc+1] & 0x0f;
                let a = self.r[rd as usize];
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                let b = self.mem[addr as usize];
                if a < b {
                    self.cmp = ComparisonResult::LessThan;
                }
                else if a > b {
                    self.cmp = ComparisonResult::GreaterThan;
                }
                else {
                    self.cmp = ComparisonResult::Equal;
                }
                self.pc += 4;
            },
            Opcode::Be => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.cmp {
                    ComparisonResult::Equal => self.pc = addr as usize,
                    _ => self.pc += 3,
                }
            },
            Opcode::Bne => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.cmp {
                    ComparisonResult::Equal => self.pc += 3,
                    _ => self.pc = addr as usize,
                }
            },
            Opcode::Bg => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.cmp {
                    ComparisonResult::GreaterThan => self.pc = addr as usize,
                    _ => self.pc += 3,
                }
            },
            Opcode::Bge => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.cmp {
                    ComparisonResult::LessThan => self.pc += 3,
                    _ => self.pc = addr as usize,
                }
            },
            Opcode::Bl => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.cmp {
                    ComparisonResult::LessThan => self.pc = addr as usize,
                    _ => self.pc += 3,
                }
            },
            Opcode::Ble => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.cmp {
                    ComparisonResult::GreaterThan => self.pc += 3,
                    _ => self.pc = addr as usize,
                }
            },
            Opcode::Bc => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.carry {
                    false => self.pc += 3,
                    true => self.pc = addr as usize,
                }
            },
            Opcode::Jmp => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                self.pc = addr as usize;
            },
            Opcode::PutS => {
                
            },
            Opcode::GetC => {
                
            },
            Opcode::Call => {
                let addr = u16::from_le_bytes(self.code[self.pc+1..self.pc+3].try_into().expect("slice has incorrect length"));
                match self.push(self.pc+3) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                }
                self.pc = addr as usize;
            }
            Opcode::Ret => {
                match self.pop() {
                    Ok(state) => {
                        self.r = state.r;
                        self.pc = state.pc;
                    }
                    Err(e) => return Err(e),
                }
            }
            Opcode::Halt => {
                return Ok(false);
            },
            _ => return Err(Error::UnknownOpcode(self.code[self.pc], self.pc)),
        }
        Ok(true)
    }
}
