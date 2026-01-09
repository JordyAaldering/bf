use std::{env, fs, io::{self, Read, Write}};

#[derive(Clone, Copy, Debug)]
enum Instruction {
    /// `>`
    ///
    /// Increment the data pointer by one.
    IncPtr,
    /// `<`
    ///
    /// Increment the data pointer by one.
    DecPtr,
    /// `+`
    ///
    /// Increment the byte at the data pointer by one.
    IncVal,
    /// `-`
    ///
    /// Increment the byte at the data pointer by one.
    DecVal,
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

fn eval(instructions: &Vec<Instruction>, rdr: &mut impl Read, wtr: &mut impl Write) -> io::Result<()> {
    let mut tape = [0u8; 64];
    let mut ptr = 0;
    let mut pc = 0;

    while let Some(instr) = instructions.get(pc) {
        use Instruction::*;
        match instr {
            IncPtr => ptr += 1,
            DecPtr => ptr -= 1,
            IncVal => tape[ptr] += 1,
            DecVal => tape[ptr] -= 1,
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

        //sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let src = fs::read_to_string(&args[1])
        .map_err(|e| e.to_string())?;

    let bf = parse(&src)
        .map_err(|e| format!("{:?}", e))?;

    eval(&bf, &mut io::stdin(), &mut io::stdout())
        .map_err(|e| format!("{:?}", e))?;

    Ok(())
}
