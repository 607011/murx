/**
 * Copyright (c) 2022 Oliver Lau <oliver@ersatzworld.net>
 * All rights reserved.
 */

extern crate riscvm;
extern crate regex;
extern crate log;

use riscvm::error::Error;
// use riscvm::opcode::Opcode;
use std::fs::File;
use std::env;
use std::io::{self, BufRead, Lines};
use std::collections::HashMap;
use regex::Regex;

/*
  <alpha>  := 'a' | 'b' | 'c' | 'd' | 'e' | /* ... */ 'z'
            | 'A' | 'B' | 'C' | 'D' | 'E' | /* ... */ 'Z'
  <digit>  := 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9
  <hexdigit> := <digit> | 'a' | 'b' | 'c' | 'd' | 'e' | 'f'
  <Ident>  := <alpha>+ (<alpha> | <digit> | '_')*
  <Label>  := <Ident>
  <Opcode> := 'cp' | 'add' | 'sub' | 'mul' | 'div' | 'xor' | 'and' | 'or' | 'not'
            | 'shr' | 'shl' | 'cmp'
            | 'be' | 'bne' | 'bg' | 'bge' | 'bl' | 'ble' | 'bc'
            | 'jmp' | 'puts' | 'getc' | 'call' | 'ret' | 'halt'
  <Register> := 'r' <digit>+
  <UnarySign> := '-' | '+'
  <DecNum> := '#' <UnarySign>? <digit>+
  <HexNum> := '$' <hexdigit>+
  <BinNum> := '!' (0 | 1)+
*/

struct AST {
    label: Option<String>,
    opcode: Option<String>,
    operands: Option<Vec<String>>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            label: Option::default(),
            opcode: Option::default(),
            operands: Option::default(),
        }
    }
}

pub struct Compiler {
    obj: Vec<u8>,
    labels: HashMap<String, usize>,
}

pub enum Token {
    Reg(u8),
    Mem(usize),
    Imm(i16),
    Str(String),
    Dat(Vec<i16>),
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            obj: Vec::new(),
            labels: HashMap::new(),
        }
    }

    fn tokenize(&mut self, line: String) -> Result<Vec<String>, Error> {
        let mut line = line;
        // strip comment
        if let Some(idx)= line.find(';') {
            line = line.get(0..idx).unwrap().to_string();
        }
        let re = Regex::new(r#""[^"]+"|[^\s"]+"#).expect("Invalid regex");
        let matches: Vec<String> = re.captures_iter(&line)
            .map(|m| m.get(0).map_or("", |m| m.as_str()).to_string())
            .collect::<Vec<String>>();
        Ok(matches)
    }

    fn parse<B: BufRead>(&mut self, code: &mut Lines<B>) -> Result<(), Error> {
        for (_line_no, line) in code.enumerate() {
            if let Ok(line) = line {
                let mut ast = AST::new();
                /*
                 * LABEL OPCODE OPERANDS ; COMMENT
                 * .DIRECTIVE
                 */
                let tokens = self.tokenize(line);
                for token in tokens.iter().peekable() {

                }
            }
        }
        Ok(())
    }

    pub fn assemble(&mut self, filename: &String) -> Result<(), Error> {
        let file = match File::open(filename) {
            Ok(file) => file,
            Err(e) => return Err(Error::FileNotFound(e.to_string())),
        };
        let mut lines = io::BufReader::new(file).lines();
        self.parse(&mut lines)
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut compiler = Compiler::new();
    match compiler.assemble(&filename) {
        Ok(()) => (),
        Err(e) => panic!("{}", e),
    }
}
