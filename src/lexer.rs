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
        let mut pos = Position::default(); // 0, 0
        let mut input = <&str>::clone(&self.input);

        while !input.is_empty() {
            // <, <=, >, >=, ==, !=
            if input.starts_with("<=") {
                tokens.push(Token::new(TokenKind::Le, pos.next_token(2)));
                input = &input[2..];
                continue;
            } else if input.starts_with(">=") {
                tokens.push(Token::new(TokenKind::Ge, pos.next_token(2)));
                input = &input[2..];
                continue;
            } else if input.starts_with("==") {
                tokens.push(Token::new(TokenKind::EtEq, pos.next_token(2)));
                input = &input[2..];
                continue;
            } else if input.starts_with("!=") {
                tokens.push(Token::new(TokenKind::Ne, pos.next_token(2)));
                input = &input[2..];
                continue;
            }
            // skip white spaces
            if input.starts_with(' ') || input.starts_with('\t') {
                pos.next_char();
            } else if input.starts_with('\n') {
                pos.next_line();
            } else if input.starts_with('+') {
                tokens.push(Token::new(
                    TokenKind::BinOp(BinOpToken::Plus),
                    pos.next_char(),
                ));
            } else if input.starts_with('-') {
                tokens.push(Token::new(
                    TokenKind::BinOp(BinOpToken::Minus),
                    pos.next_char(),
                ));
            } else if input.starts_with('*') {
                tokens.push(Token::new(
                    TokenKind::BinOp(BinOpToken::Mul),
                    pos.next_char(),
                ));
            } else if input.starts_with('/') {
                tokens.push(Token::new(
                    TokenKind::BinOp(BinOpToken::Div),
                    pos.next_char(),
                ));
            } else if input.starts_with('(') {
                tokens.push(Token::new(
                    TokenKind::OpenDelim(DelimToken::Paren),
                    pos.next_char(),
                ));
            } else if input.starts_with(')') {
                tokens.push(Token::new(
                    TokenKind::CloseDelim(DelimToken::Paren),
                    pos.next_char(),
                ));
            } else if input.starts_with(';') {
                tokens.push(Token::new(TokenKind::Semi, pos.next_char()));
            } else if input.starts_with('=') {
                tokens.push(Token::new(TokenKind::Eq, pos.next_char()));
            } else if input.starts_with('<') {
                tokens.push(Token::new(TokenKind::Lt, pos.next_char()));
            } else if input.starts_with('>') {
                tokens.push(Token::new(TokenKind::Gt, pos.next_char()));
            } else if input.starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']) {
                let mut chars = input.chars().peekable();

                let mut number = String::from(chars.next().unwrap());

                while let Some(&('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')) =
                    chars.peek()
                {
                    number.push(chars.next().unwrap())
                }

                let len_token = number.len();

                let num = number
                    .parse::<isize>()
                    .expect("Currently support only a number literal.");

                tokens.push(Token::new(TokenKind::Num(num), pos.next_token(len_token)));

                input = &input[len_token..];
                continue;
            } else if input
                .starts_with(&('a'..='z').chain(vec!['_'].into_iter()).collect::<Vec<_>>()[..])
            {
                let mut chars = input.chars().peekable();
                let mut ident = String::from(chars.next().unwrap());

                while let Some(&('a'..='z') | '_') = chars.peek() {
                    ident.push(chars.next().unwrap());
                }

                let len_token = ident.len();

                tokens.push(Token::new(
                    // Identifier or Reserved word
                    match ident.as_str() {
                        "return" => TokenKind::Return,
                        "if" => TokenKind::If,
                        "else" => TokenKind::Else,
                        "while" => TokenKind::While,
                        "for" => TokenKind::For,
                        _ => TokenKind::Ident(ident),
                    },
                    pos.next_token(len_token),
                ));

                input = &input[len_token..];
                continue;
            } else {
                self.error_at(
                    &pos,
                    &format!("Unexpected char while tokenize : {:?}", &pos),
                )
            } // one character tokenize

            input = &input[1..];
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
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    BinOp(BinOpToken),
    Num(isize),
    // An identifier
    Ident(String),
    /// return
    Return,
    /// if
    If,
    /// else
    Else,
    /// while
    While,
    /// for
    For,
    /// An opening delimiter e.g., `{`
    OpenDelim(DelimToken),
    /// An closing delimiter e.g., `}`
    CloseDelim(DelimToken),
    /// semicolon
    Semi,
    /// Less than
    Lt,
    /// Greater than
    Gt,
    /// Less than or equal to
    Le,
    /// Greater than or equal to
    Ge,
    /// Equal to
    EtEq,
    /// Not equal to
    Ne,
    // assign
    Eq,
    Eof,
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelimToken {
    Brace,   // `{` or `}`
    Paren,   // `(` or `)`
    Bracket, // `[` or `]`
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

    /// consume the next token if it is the expected kind and return true, otherwise return false
    pub fn consume(&mut self, kind: TokenKind) -> bool {
        if self.peek().is_some_and(|token| *token.kind == kind) {
            self.next();
            return true;
        }
        false
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

    pub fn expect(&mut self, kind: TokenKind) {
        let next_token = self.next();

        match next_token {
            Some(Token { kind: k, pos }) => {
                if *k != kind {
                    self.error_at(Some(pos), &format!("expected {:?} but got {:?}", kind, k))
                }
            }
            None => self.error_at(None, &format!("expected {:?} but got None", kind)),
        }
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        self.iter.peek()
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

        let input = String::from("(1 + 2) * 3");
        let lexer = Lexer::new(&input);

        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::OpenDelim(DelimToken::Paren),
                TokenKind::Num(1),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(2),
                TokenKind::CloseDelim(DelimToken::Paren),
                TokenKind::BinOp(BinOpToken::Mul),
                TokenKind::Num(3),
                TokenKind::Eof
            ]
        );

        let input = String::from("(1 + (2 + 3)) * 4");
        let lexer = Lexer::new(&input);

        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::OpenDelim(DelimToken::Paren),
                TokenKind::Num(1),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::OpenDelim(DelimToken::Paren),
                TokenKind::Num(2),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(3),
                TokenKind::CloseDelim(DelimToken::Paren),
                TokenKind::CloseDelim(DelimToken::Paren),
                TokenKind::BinOp(BinOpToken::Mul),
                TokenKind::Num(4),
                TokenKind::Eof
            ]
        );

        let input = String::from("1 < 2");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::Lt,
                TokenKind::Num(2),
                TokenKind::Eof
            ]
        );

        let input = String::from("1 > 2");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::Gt,
                TokenKind::Num(2),
                TokenKind::Eof
            ]
        );

        let input = String::from("1 <= 2");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::Le,
                TokenKind::Num(2),
                TokenKind::Eof
            ]
        );

        let input = String::from("1 >= 2");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::Ge,
                TokenKind::Num(2),
                TokenKind::Eof
            ]
        );

        let input = String::from("1 == 2");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::EtEq,
                TokenKind::Num(2),
                TokenKind::Eof
            ]
        );

        let input = String::from("1 != 2");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Num(1),
                TokenKind::Ne,
                TokenKind::Num(2),
                TokenKind::Eof
            ]
        );

        let input = String::from("a = 1");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Ident("a".to_string()),
                TokenKind::Eq,
                TokenKind::Num(1),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_ident() {
        let input = String::from("a;");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Ident("a".to_string()),
                TokenKind::Semi,
                TokenKind::Eof
            ]
        );

        let input = String::from("a * c");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Ident("a".to_string()),
                TokenKind::BinOp(BinOpToken::Mul),
                TokenKind::Ident("c".to_string()),
                TokenKind::Eof
            ]
        );

        let input = String::from("aa = bb_c =  1");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Ident("aa".to_string()),
                TokenKind::Eq,
                TokenKind::Ident("bb_c".to_string()),
                TokenKind::Eq,
                TokenKind::Num(1),
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

    #[test]
    fn test_tokenize_return() {
        let input = String::from("return 1;");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::Return,
                TokenKind::Num(1),
                TokenKind::Semi,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_if() {
        let input = String::from("if (1) 2; else 3;");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::If,
                TokenKind::OpenDelim(DelimToken::Paren),
                TokenKind::Num(1),
                TokenKind::CloseDelim(DelimToken::Paren),
                TokenKind::Num(2),
                TokenKind::Semi,
                TokenKind::Else,
                TokenKind::Num(3),
                TokenKind::Semi,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_while() {
        let input = String::from("while (1) 2;");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::While,
                TokenKind::OpenDelim(DelimToken::Paren),
                TokenKind::Num(1),
                TokenKind::CloseDelim(DelimToken::Paren),
                TokenKind::Num(2),
                TokenKind::Semi,
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn test_tokenize_for() {
        let input = String::from("for (1; 2; 3) 4;");
        let lexer = Lexer::new(&input);
        assert_eq!(
            lexer
                .tokenize()
                .into_iter()
                .map(|token| token.kind())
                .collect::<Vec<_>>(),
            token_kinds![
                TokenKind::For,
                TokenKind::OpenDelim(DelimToken::Paren),
                TokenKind::Num(1),
                TokenKind::Semi,
                TokenKind::Num(2),
                TokenKind::Semi,
                TokenKind::Num(3),
                TokenKind::CloseDelim(DelimToken::Paren),
                TokenKind::Num(4),
                TokenKind::Semi,
                TokenKind::Eof
            ]
        );
    }
}
