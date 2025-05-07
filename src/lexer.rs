use std::iter::Peekable;

pub struct Lexer<'a> {
    pub input: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut input_chars = self.input.chars().peekable();

        let mut pos = Position::default();

        while let Some(c) = input_chars.next() {
            match c {
                ' ' | '\t' => {
                    pos.next_char();
                }
                '\n' => {
                    pos.next_line();
                }
                '+' => tokens.push(Token::new(
                    TokenKind::BinOp(BinOpToken::Plus),
                    pos.next_char(),
                )),
                '-' => tokens.push(Token::new(
                    TokenKind::BinOp(BinOpToken::Minus),
                    pos.next_char(),
                )),
                '0'..='9' => {
                    let mut number = c.to_string();
                    while let Some(&next_char) = input_chars.peek() {
                        if next_char.is_ascii_digit() {
                            number.push(next_char);
                            input_chars.next();
                        } else {
                            break;
                        }
                    }

                    let len_token = number.len();
                    let num = number.parse::<isize>().unwrap();
                    tokens.push(Token::new(TokenKind::Num(num), pos.next_token(len_token)));
                }
                _ => self.error_at(
                    &pos,
                    &format!("Unexpected character while tokenize: {:?}", &pos),
                ),
            }
        }

        tokens.push(Token::new(TokenKind::Eof, pos.next_token(0)));

        tokens
    }

    pub fn error_at(&self, pos: &Position, msg: &str) -> ! {
        let mut splitted = self.input.split('\n');
        let line = splitted.nth(pos.n_line).unwrap_or_else(|| {
            panic!(
                "Position is illeagl, pos: {:?}, \n input: {}",
                pos, self.input
            );
        });

        eprintln!("{}", line);
        let mut buffer = String::with_capacity(pos.n_char + 1);
        for _ in 0..pos.n_char {
            buffer.push(' ');
        }
        buffer.push('^');
        eprintln!("{}", buffer);
        eprintln!("Error: {}", msg);
        panic!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOpToken {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    BinOp(BinOpToken),
    Num(isize),
    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: Box<TokenKind>,
    pub pos: Position,
}

pub struct TokenStream<'a, I: Iterator<Item = Token>> {
    iter: Peekable<I>,
    input: &'a str,
}

#[allow(unused)]
impl Token {
    pub fn new(kind: TokenKind, pos: Position) -> Self {
        Self {
            kind: Box::new(kind),
            pos,
        }
    }

    pub fn kind_eq(&self, rhs: &Token) -> bool {
        self.kind == rhs.kind
    }

    pub fn kind(&self) -> Box<TokenKind> {
        self.kind.clone()
    }
}

#[allow(unused)]
impl<'a, I: Iterator<Item = Token>> TokenStream<'a, I> {
    pub fn new(iter: I, input: &'a str) -> Self {
        Self {
            iter: iter.peekable(),
            input,
        }
    }

    pub fn expect_number(&mut self) -> isize {
        let token = self.next();

        match token {
            Some(Token { kind, pos }) => match *kind {
                TokenKind::Num(num) => num,
                _ => self.error_at(Some(pos), &format!("number expected but got: {:?}", kind)),
            },
            _ => self.error_at(None, &format!("number expected but got: {:?}", token)),
        }
    }

    pub fn peek_kind(&mut self) -> Option<Box<TokenKind>> {
        self.iter.peek().map(|token| token.kind.clone())
    }

    pub fn at_eof(&mut self) -> bool {
        match self.peek_kind() {
            Some(token) => matches!(*token, TokenKind::Eof),
            None => panic!("Unexpected end of stream"),
        }
    }

    pub fn error_at(&mut self, pos: impl Into<Option<Position>>, msg: &str) -> ! {
        let pos: Option<Position> = pos.into();
        match pos {
            None => panic!("Passed pos info was None. \n{}", msg),
            Some(pos) => {
                let mut splitted = self.input.split('\n');
                let line = splitted.nth(pos.n_line).unwrap_or_else(|| {
                    panic!(
                        "Position is illeagl, pos: {:?}, \n input: {}",
                        pos, self.input
                    );
                });

                eprintln!("{}", line);
                let mut buffer = String::with_capacity(pos.n_char + 1);
                for _ in 0..pos.n_char {
                    buffer.push(' ');
                }
                buffer.push('^');
                eprintln!("{}", buffer);
                eprintln!("Error: {}", msg);
                panic!()
            }
        }
    }
}

impl<I: Iterator<Item = Token>> Iterator for TokenStream<'_, I> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Position {
    pub n_char: usize,
    pub n_line: usize,
}

#[allow(unused)]
impl Position {
    pub fn new(n_char: usize, n_line: usize) -> Self {
        Self { n_char, n_line }
    }

    pub fn next_line(&mut self) -> Self {
        let ret = self.clone();
        self.n_char = 0;
        self.n_line += 1;
        ret
    }

    pub fn next_char(&mut self) -> Self {
        let ret = self.clone();
        self.n_char += 1;
        ret
    }

    pub fn next_token(&mut self, len_token: usize) -> Self {
        let ret = self.clone();
        self.n_char += len_token;
        ret
    }
}

#[allow(unused)]
trait Expect {
    type Item;

    fn expect(kind: Self::Item) -> Self::Item;
}

#[cfg(test)]
#[allow(unused)]
macro_rules! tokens {
    ( $( $token_kind:expr ),* $(,)? ) => {{
        let mut tmp_vec = Vec::new();
        $(
            let pos = crate::lexer::Position::default();
            tmp_vec.push(Token::new($token_kind, pos));
        )*
        tmp_vec
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! token_poses {
    ( $( ($token_kind:expr, $pos:expr) ),* $(,)? ) => {
        vec![
            $( Token::new($token_kind, $pos) ),*
        ]
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! token_kinds {
    ( $( $token_kind:expr ), *) => {{
        let mut temp_vec = Vec::new();
        $(
            let pos = $crate::lexer::Position::default();
            temp_vec.push(Token::new($token_kind, pos));
        )*
        temp_vec
            .into_iter()
            .map(|token| token.kind())
            .collect::<Vec<_>>()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = String::from("1 + 4 - 909");

        let lexer = Lexer::new(&input);

        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(4),
                TokenKind::BinOp(BinOpToken::Minus),
                TokenKind::Num(909),
                TokenKind::Eof
            ]
        );

        let input = String::from("0\t + 5+1+9-3 - \n 909");
        let lexer = Lexer::new(&input);

        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(0),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(5),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(1),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(9),
                TokenKind::BinOp(BinOpToken::Minus),
                TokenKind::Num(3),
                TokenKind::BinOp(BinOpToken::Minus),
                TokenKind::Num(909),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_pos() {
        let input = String::from("1 +1");
        let tokenizer = Lexer::new(&input);
        assert_eq!(
            tokenizer.tokenize(),
            token_poses![
                (TokenKind::Num(1), Position::new(0, 0)),
                (TokenKind::BinOp(BinOpToken::Plus), Position::new(2, 0)),
                (TokenKind::Num(1), Position::new(3, 0)),
                (TokenKind::Eof, Position::new(4, 0))
            ]
        );

        let input = String::from("1 +1 \n\t+5");
        let tokenizer = Lexer::new(&input);
        assert_eq!(
            tokenizer.tokenize(),
            token_poses![
                (TokenKind::Num(1), Position::new(0, 0)),
                (TokenKind::BinOp(BinOpToken::Plus), Position::new(2, 0)),
                (TokenKind::Num(1), Position::new(3, 0)),
                (TokenKind::BinOp(BinOpToken::Plus), Position::new(1, 1)),
                (TokenKind::Num(5), Position::new(2, 1)),
                (TokenKind::Eof, Position::new(3, 1))
            ]
        );
    }
}
