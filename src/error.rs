/**
 * Copyright (c) 2022 Oliver Lau <oliver@ersatzworld.net>
 * All rights reserved.
 */

 extern crate thiserror;
use self::thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("division by zero")]
    DivisionByZero,
    #[error("unknown opcode 0x{0:02x} @ 0x{1:04x}")]
    UnknownOpcode(u8, usize),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("cannot read file metadata: {0}")]
    CannotReadFileMetadata(String),
    #[error("object file too large ({0} bytes)")]
    ObjectFileTooLarge(usize),
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("invalid character '{0}'")]
    InvalidCharacter(char),
}
