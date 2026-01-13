#[derive(Clone, Copy, Debug)]
pub enum Token {
    Gt,
    Lt,
    Plus,
    Minus,
    Dot,
    Comma,
    LSquare,
    RSquare,
}

pub struct Lexer<'src> {
    /// The input program as a string.
    src: &'src str,
    /// Index of the current character in the source string.
    current: usize,
    /// Line number of the current character.
    line: usize,
    /// Column number of the current character.
    col: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { src, current: 0, line: 1, col: 1 }
    }

    /// Get the next character and consume it.
    fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.src.chars().nth(self.current) {
            self.current += 1;
            self.col += 1;
            Some(c)
        } else {
            None
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.consume() {
            use Token::*;
            let token = match c {
                '>' => Gt,
                '<' => Lt,
                '+' => Plus,
                '-' => Minus,
                '.' => Dot,
                ',' => Comma,
                '[' => LSquare,
                ']' => RSquare,
                // Skip unknown tokens
                '\n' => {
                    self.line += 1;
                    self.col = 1;
                    continue
                }
                _ => continue,
            };

            return Some(token);
        };

        None
    }
}
