use std::fmt;

use crate::{Instruction, lexer::{Lexer, Token}};

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

#[derive(Debug)]
pub enum Error {
    MissingLoopOpen(),
    MissingLoopEnd(),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            MissingLoopOpen() => write!(f, "`]` at column {} does not have a matching `[`", 0),
            MissingLoopEnd() => write!(f, "found {} unclosed `[`", 0),
        }
    }
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>, Error> {
        let mut bf = Vec::new();

        while let Some(c) = self.lexer.next() {
            use Token::*;
            use Instruction::*;
            let instr = match c {
                Gt    => IncPtr,
                Lt    => DecPtr,
                Plus  => IncVal,
                Minus => DecVal,
                Dot   => Write,
                Comma => Read,
                LSquare => {
                    Loop(self.parse_loop()?)
                },
                RSquare => {
                    return Err(Error::MissingLoopOpen());
                },
            };

            bf.push(instr);
        }

        Ok(bf)
    }

    fn parse_loop(&mut self) -> Result<Vec<Instruction>, Error> {
        let mut bf = Vec::new();

        while let Some(c) = self.lexer.next() {
            use Token::*;
            use Instruction::*;
            let instr = match c {
                Gt    => IncPtr,
                Lt    => DecPtr,
                Plus  => IncVal,
                Minus => DecVal,
                Dot   => Write,
                Comma => Read,
                LSquare => {
                    Loop(self.parse_loop()?)
                },
                RSquare => {
                    return Ok(bf);
                },
            };

            bf.push(instr);
        }

        Err(Error::MissingLoopEnd())
    }
}
