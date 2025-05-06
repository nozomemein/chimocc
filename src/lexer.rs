use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpToken {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    BinOp(BinOpToken),
    Num(isize),
    Eof,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: Box<TokenKind>,
}

pub struct TokenStream<I: Iterator<Item = Token>> {
    iter: Peekable<I>,
}

pub fn tokenize(input: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut input_chars = input.chars().peekable();

    while let Some(c) = input_chars.next() {
        match c {
            ' ' | '\n' | '\t' => continue,
            '+' => tokens.push(Token::new(TokenKind::BinOp(BinOpToken::Plus))),
            '-' => tokens.push(Token::new(TokenKind::BinOp(BinOpToken::Minus))),
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

                let num = number.parse::<isize>().unwrap();
                tokens.push(Token::new(TokenKind::Num(num)));
            }
            _ => panic!("Unexpected character: {}", c),
        }
    }

    tokens.push(Token::new(TokenKind::Eof));

    tokens
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }
}

#[allow(unused)]
impl<I: Iterator<Item = Token>> TokenStream<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
        }
    }

    pub fn expect_number(&mut self) -> isize {
        let next = self.iter.next().map(|token| *token.kind);

        match next {
            Some(TokenKind::Num(num)) => num,
            _ => panic!("Expected a number, but got: {:?}", next),
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
}

impl<I: Iterator<Item = Token>> Iterator for TokenStream<I> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[allow(unused)]
trait Expect {
    type Item;

    fn expect(kind: Self::Item) -> Self::Item;
}

#[cfg(test)]
macro_rules! tokens {
    ( $( $token_kind:expr ),* $(,)? ) => {
        vec![ $( Token::new($token_kind) ),* ]
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = String::from("1 + 4 - 909");

        assert_eq!(
            tokenize(input),
            tokens![
                TokenKind::Num(1),
                TokenKind::BinOp(BinOpToken::Plus),
                TokenKind::Num(4),
                TokenKind::BinOp(BinOpToken::Minus),
                TokenKind::Num(909),
                TokenKind::Eof
            ]
        );

        let input = String::from("0\t + 5+1+9-3 - \n 909");

        assert_eq!(
            tokenize(input),
            tokens![
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
}
