use crate::Instruction;

/// Cancel out adjacent increments and decrements.
///
/// `><` `<>` `+-` `-+`
pub fn cancel(bf: &mut Vec<Instruction>) {
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

/// Replace `[+]` and `[-]` by a single instruction
/// that resets the byte at the data pointer to zero.
pub fn clearloop(bf: &mut Vec<Instruction>) {
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
