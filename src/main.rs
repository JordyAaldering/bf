mod lexer;
mod parser;
mod opt;

use std::{env, fs, io::{self, Read, Write}};

use crate::{lexer::Lexer, parser::Parser, opt::*};

#[derive(Debug)]
pub enum Instruction {
    /// `>`
    ///
    /// Increment the data pointer by one.
    IncPtr(usize),
    /// `<`
    ///
    /// Decrement the data pointer by one.
    DecPtr(usize),
    /// `+`
    ///
    /// Increment the byte at the data pointer by one.
    IncVal(u8),
    /// `-`
    ///
    /// Decrement the byte at the data pointer by one.
    DecVal(u8),
    /// `[+]` `[-]`
    ///
    /// Reset the byte at the data pointer to zero.
    ClearVal,
    /// `.`
    ///
    /// Output the byte at the data pointer.
    Write,
    /// `,`
    ///
    /// Accept one byte of input, storing its value in the byte at the data pointer.
    Read,
    /// `[ ... ]`
    ///
    /// While the byte at the data pointer is zero, repeat all instructions until the matching `]`.
    /// Otherwise, jump forward to the command after the matching `]`.
    Loop(Vec<Instruction>),
}

struct Context<'a> {
    rdr: Box<&'a mut dyn Read>,
    wtr: Box<&'a mut dyn Write>,
    tape: [u8; 64],
    ptr: usize,
}

impl<'a> Context<'a> {
    fn new(rdr: &'a mut impl Read, wtr: &'a mut impl Write) -> Self {
        Self {
            rdr: Box::new(rdr),
            wtr: Box::new(wtr),
            tape: [0u8; 64],
            ptr: 0,
        }
    }

    fn eval(&mut self, prog: &Vec<Instruction>) -> io::Result<()> {
        for instr in prog {
            use Instruction::*;
            match instr {
                IncPtr(x) => self.ptr += *x,
                DecPtr(x) => self.ptr -= *x,
                IncVal(x) => self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(*x),
                DecVal(x) => self.tape[self.ptr] = self.tape[self.ptr].wrapping_sub(*x),
                ClearVal => self.tape[self.ptr] = 0,
                Write  => {
                    self.wtr.write(&[self.tape[self.ptr]])?;
                },
                Read => {
                    let mut input = [0u8; 1];
                    self.rdr.read_exact(&mut input)?;
                    self.tape[self.ptr] = input[0];
                },
                Loop(inner) => {
                    while self.tape[self.ptr] != 0 {
                        self.eval(inner)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let src = fs::read_to_string(&args[1])
        .map_err(|e| e.to_string())?;

    // Parse
    let lexer = Lexer::new(&src);
    let mut parser = Parser::new(lexer);
    let mut prog = parser.parse()
        .map_err(|e| e.to_string())?;

    // Optimize
    cancel(&mut prog);
    clearloop(&mut prog);

    // Interpret
    let mut rdr = io::stdin();
    let mut wtr = io::stdout();
    let mut ctx = Context::new(&mut rdr, &mut wtr);
    ctx.eval(&mut prog)
        .map_err(|e| e.to_string())?;

    Ok(())
}
