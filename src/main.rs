mod lexer;
mod parser;

use std::{env, fs, io::{self, Read, Write}};

use crate::{lexer::Lexer, parser::Parser};

#[derive(Debug)]
enum Instruction {
    /// `>`
    ///
    /// Increment the data pointer by one.
    IncPtr,
    /// `<`
    ///
    /// Decrement the data pointer by one.
    DecPtr,
    /// `+`
    ///
    /// Increment the byte at the data pointer by one.
    IncVal,
    /// `-`
    ///
    /// Decrement the byte at the data pointer by one.
    DecVal,
    /// `[-]` or `[+]`
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

/// Cancel out adjacent increments and decrements.
///
/// `><`
/// `<>`
/// `+-`
/// `-+`
fn cancel(bf: &mut Vec<Instruction>) {
    // Go from back to front, to reduce the number of shifts when removing
    let mut i = bf.len() - 1;

    while i > 0 {
        use Instruction::*;
        if let Loop(instr) = &mut bf[i] {
            // Recurse
            cancel(instr);
            i -= 1;
        } else {
            let r = &bf[i];
            let l = &bf[i - 1];
            match (l, r) {
                (IncPtr, DecPtr) |
                (DecPtr, IncPtr) |
                (IncVal, DecVal) |
                (DecVal, IncVal) => {
                    bf.remove(i);
                    bf.remove(i - 1);
                }
                _ => {
                    i -= 1;
                },
            }
        }
    }
}

/// Replace `[+]` and `[-]` by a single instruction.
fn clearloop(bf: &mut Vec<Instruction>) {
    for x in bf {
        use Instruction::*;
        if let Loop(instr) = x {
            match instr[..] {
                [IncVal] |
                [DecVal] => {
                    *x = ClearVal;
                },
                _ => {
                    // Recurse
                    clearloop(instr);
                }
            }
        }
    }
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
                IncPtr => self.ptr += 1,
                DecPtr => self.ptr -= 1,
                IncVal => self.tape[self.ptr] = self.tape[self.ptr].wrapping_add(1),
                DecVal => self.tape[self.ptr] = self.tape[self.ptr].wrapping_sub(1),
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

    let mut rdr = io::stdin();
    let mut wtr = io::stdout();
    let mut ctx = Context::new(&mut rdr, &mut wtr);
    ctx.eval(&mut prog)
        .map_err(|e| e.to_string())?;

    Ok(())
}
