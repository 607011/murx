/*
 * Copyright (c) 2022 Oliver Lau <oliver@ersatzworld.net>
 * All rights reserved.
 */

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Debug};

#[repr(u8)]
pub enum Opcode {
    CpRR = 0x01,
    CpRImm,
    CpMemR,
    CpRMem,
    AddRR,
    AddRImm,
    AddRMem,
    SubRR,
    SubRImm,
    SubRMem,
    MulRR,
    MulRImm,
    MulRMem,
    DivRR,
    DivRImm,
    DivRMem,
    NegR,
    NegMem,
    XorRR,
    XorRImm,
    XorRMem,
    AndRR,
    AndRImm,
    AndRMem,
    OrRR,
    OrRImm,
    OrRMem,
    NotR,
    NotMem,
    ShrRR,
    ShrRImm,
    ShlRR,
    ShlRImm,
    CmpRR,
    CmpRImm,
    CmpRMem,
    Be,
    Bne,
    Bg,
    Bge,
    Bl,
    Ble,
    Bc,
    Jmp,
    PutS,
    GetC,
    Call,
    Ret,
    Halt,
}

pub type Literal = String;

#[derive(Clone, Copy)]
pub enum TokenType {
    Directive,
    Identifier,
    String,
    DecNumber,
    HexNumber,
    BinNumber,
    Cp,
    Add,
    Sub,
    Mul,
    Div,
    Xor,
    And,
    Or,
    Not,
    Shl,
    Shr,
    Cmp,
    Be,
    Bne,
    Bg,
    Bge,
    Bl,
    Ble,
    Bc,
    Jmp,
    Call,
    Ret,
    Puts,
    Getc,
    Halt,
    Eof,
}

impl Debug for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Token {
    ttype: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
    position: usize,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Option<Literal>, line: usize, position: usize) -> Self {
        Token {
            ttype,
            lexeme,
            line,
            literal,
            position,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}, {}, @ {}, {}", self.ttype, self.lexeme, self.line, self.position)
    }
}



struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start_idx: usize,
    current_idx: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    #[inline]
    pub fn new(source: String) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("cp".to_string(), TokenType::Cp);
        keywords.insert("add".to_string(), TokenType::Add);
        keywords.insert("sub".to_string(), TokenType::Sub);
        keywords.insert("mul".to_string(), TokenType::Mul);
        keywords.insert("div".to_string(), TokenType::Div);
        keywords.insert("xor".to_string(), TokenType::Xor);
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("not".to_string(), TokenType::Not);
        keywords.insert("shl".to_string(), TokenType::Shl);
        keywords.insert("shr".to_string(), TokenType::Shr);
        keywords.insert("cmp".to_string(), TokenType::Cmp);
        keywords.insert("be".to_string(), TokenType::Be);
        keywords.insert("bne".to_string(), TokenType::Bne);
        keywords.insert("bg".to_string(), TokenType::Bg);
        keywords.insert("bge".to_string(), TokenType::Bge);
        keywords.insert("bl".to_string(), TokenType::Bl);
        keywords.insert("ble".to_string(), TokenType::Ble);
        keywords.insert("bc".to_string(), TokenType::Bc);
        keywords.insert("add".to_string(), TokenType::Jmp);
        keywords.insert("call".to_string(), TokenType::Call);
        keywords.insert("ret".to_string(), TokenType::Ret);
        keywords.insert("puts".to_string(), TokenType::Puts);
        keywords.insert("getc".to_string(), TokenType::Getc);
        keywords.insert("halt".to_string(), TokenType::Halt);
        Self {
            source,
            tokens: Vec::new(),
            start_idx: 0,
            current_idx: 0,
            line: 0,
            keywords,
        }
    }

    fn error(&mut self, message: String, line: usize, position: usize) {
        eprintln!("{} in line {} at position {}", message, line, position);
    }

    #[inline]
    fn at_end(&mut self) -> bool {
        self.current_idx >= self.source.len()
    }

    pub fn tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn scan_tokens(&mut self) {
        while !self.at_end() {
            self.start_idx = self.current_idx;
            self.scan_token();
        }
    }

    fn peek(&mut self) -> char {
        if self.current_idx >= self.source.len()-1 {
            return '\0';
        }
        self.source.chars().nth(self.current_idx+1).unwrap_or('\0')
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.at_end() || self.peek() != expected {
            return false;
        }
        self.current_idx += 1;
        true
    }

    #[inline]
    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current_idx).unwrap();
        self.current_idx += 1;
        c
    }
    
    #[inline]
    fn add_literal(&mut self, t: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start_idx..self.current_idx];
        self.tokens.push(Token::new(t, text.to_string(), literal, self.line, 0));
    }

    #[inline]
    fn add_token(&mut self, t: TokenType) {
        self.add_literal(t, None);
    }

    fn consume_string(&mut self) {
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.at_end() {
            self.error("unterminated string".to_string(), self.line, 0);
        }
        let source = &self.source.to_string();
        let token = &source[self.start_idx+1..self.current_idx-1];
        self.add_literal(TokenType::String, Some(token.to_string()));
    }

    fn consume_comment(&mut self) {
        while self.peek() != '\n' && !self.at_end() {
            self.advance();
        }
    }

    fn consume_directive(&mut self) {
        while !self.at_end() && match self.peek() {
            'a'..='z' | 'A'..='Z' => true,
            _ => false,
        } 
        {
            self.advance();
        }
        let source = &self.source.to_string();
        let token = &source[self.start_idx..self.current_idx];
        self.add_literal(TokenType::Directive, Some(token.to_string()));
    }

    fn consume_hex_number(&mut self) {
        while !self.at_end() && match self.peek() {
            '0'..='9' | 'a'..='f' | 'A'..='F' => true,
            _ => false,
        } 
        {
            self.advance();
        }
        let source = &self.source.to_string();
        let token = &source[self.start_idx..self.current_idx];
        self.add_literal(TokenType::HexNumber, Some(token.to_string()));
    }
    
    fn consume_dec_number(&mut self) {
        while !self.at_end() && match self.peek() {
            '0'..='9' => true,
            _ => false,
        } 
        {
            self.advance();
        }
        let source = &self.source.to_string();
        let token = &source[self.start_idx..self.current_idx];
        self.add_literal(TokenType::DecNumber, Some(token.to_string()));
    }
    
    fn consume_bin_number(&mut self) {
        while !self.at_end() && match self.peek() {
            '0'..='1' => true,
            _ => false,
        } 
        {
            self.advance();
        }
        let source = &self.source.to_string();
        let token = &source[self.start_idx..self.current_idx];
        self.add_literal(TokenType::BinNumber, Some(token.to_string().clone()));
    }
    
    fn consume_identifier(&mut self) {
        while !self.at_end() && match self.peek() {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => true,
            _ => false,
        } 
        {
            self.advance();
        }
        let source = &self.source.to_string();
        let token = &source[self.start_idx..self.current_idx];
        match self.keywords.get(token) {
            Some(token_type) => self.add_token(*token_type),
            None => self.add_token(TokenType::Identifier),
        }
    }
    
    pub fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            ' ' | '\r' | '\t' => (/* ignore whitespace */),
            ';' => self.consume_comment(),
            '.' => self.consume_directive(),
            '$' => self.consume_hex_number(),
            '#' => self.consume_dec_number(),
            '!' => self.consume_bin_number(),
            'a'..='z' | 'A'..='Z' => self.consume_identifier(),
            '"' => self.consume_string(),
            '\n' => self.line += 1,
            _ => log::error!("unexpected character {} at line {}", c, self.current_idx),
        }
    }
}

