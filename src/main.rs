use std::{env, fmt, fs, io::{self, Read, Write}};

#[derive(Clone, Copy, Debug)]
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
    LoopOpen(usize),
    LoopEnd(usize),
}

#[derive(Debug)]
enum Error {
    MissingLoopOpen(usize),
    MissingLoopEnd(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            MissingLoopOpen(idx) => write!(f, "`]` at column {} does not have a matching `[`", idx),
            MissingLoopEnd(cnt) => write!(f, "found {} unclosed `[`", cnt),
        }
    }
}

fn parse(src: &str) -> Result<Vec<Instruction>, Error> {
    let mut bf = Vec::new();
    let mut stack = Vec::new();
    let mut idx = 0;

    for c in src.chars() {
        use Instruction::*;
        let instr = match c {
            '>' => IncPtr,
            '<' => DecPtr,
            '+' => IncVal,
            '-' => DecVal,
            '.' => Write,
            ',' => Read,
            '[' => {
                stack.push(idx);
                LoopOpen(0)
            },
            ']' => {
                let open_idx = match stack.pop() {
                    Some(open_idx) => open_idx,
                    None => return Err(Error::MissingLoopOpen(idx))
                };

                bf[open_idx] = LoopOpen(idx);
                LoopEnd(open_idx)
            },
            _ => continue,
        };

        bf.push(instr);
        idx += 1;
    }

    if !stack.is_empty() {
        return Err(Error::MissingLoopEnd(stack.len()))
    }

    Ok(bf)
}

/// Cancel out adjacent increments and decrements
///
/// TODO: this does not work, because the pointers become invalid.
/// Probably easiest if LoopOpen and LoopEnd become a recursive definition instead
/// E.g., Loop(Vec<Instruction>)
fn cancel(bf: &mut Vec<Instruction>) {
    let mut i = 0;

    while i + 1 < bf.len() {
        let a = bf[i];
        let b = bf[i + 1];

        use Instruction::*;
        match (a, b) {
            (IncPtr, DecPtr) |
            (DecPtr, IncPtr) |
            (IncVal, DecVal) |
            (DecVal, IncVal) => {
                bf.remove(i + 1);
                bf.remove(i);
            }
            _ => {
                i += 1
            },
        }
    }
}

/// Replace `[+]` and `[-]` by a single instruction.
///
/// TODO: this does not work, because the pointers become invalid.
/// Probably easiest if LoopOpen and LoopEnd become a recursive definition instead
/// E.g., Loop(Vec<Instruction>)
fn clearloop(bf: &mut Vec<Instruction>) {
    let mut i = 0;

    while i + 2 < bf.len() {
        let a = bf[i];
        let b = bf[i + 1];
        let c = bf[i + 2];

        use Instruction::*;
        match (a, b, c) {
            (LoopOpen(_), IncVal, LoopEnd(_)) |
            (LoopOpen(_), DecVal, LoopEnd(_)) => {
                bf[i] = ClearVal;
                bf.remove(i + 2);
                bf.remove(i + 1);
            }
            _ => {
                i += 1;
            }
        }
    }
}

fn eval(bf: &Vec<Instruction>, rdr: &mut impl Read, wtr: &mut impl Write) -> io::Result<()> {
    let mut tape = [0u8; 64];
    let mut ptr = 0;
    let mut pc = 0;

    while let Some(instr) = bf.get(pc) {
        use Instruction::*;
        match instr {
            IncPtr => ptr += 1,
            DecPtr => ptr -= 1,
            IncVal => tape[ptr] = tape[ptr].wrapping_add(1),
            DecVal => tape[ptr] = tape[ptr].wrapping_sub(1),
            ClearVal => tape[ptr] = 0,
            Write  => {
                wtr.write(&tape[ptr..=ptr])?;
            },
            Read => {
                let mut input = [0u8; 1];
                rdr.read_exact(&mut input)?;
                tape[ptr] = input[0];
            },
            LoopOpen(end) => {
                if tape[ptr] == 0 {
                    pc = *end;
                }
            },
            LoopEnd(open) => {
                if tape[ptr] != 0 {
                    pc = *open;
                }
            }
        }

        pc += 1;
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let src = fs::read_to_string(&args[1])
        .map_err(|e| e.to_string())?;
    let mut bf = parse(&src)
        .map_err(|e| e.to_string())?;

    // Optimize
    cancel(&mut bf);
    clearloop(&mut bf);

    eval(&bf, &mut io::stdin(), &mut io::stdout())
        .map_err(|e| e.to_string())?;

    Ok(())
}
