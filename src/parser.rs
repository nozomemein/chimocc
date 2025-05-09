use crate::lexer::{BinOpToken, DelimToken, Token, TokenKind, TokenStream};

pub struct Parser {}

#[allow(unused)]
impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_expr<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        let mut lhs = self.parse_mul(tokens);

        while let Some(Token { kind, .. }) = tokens.peek() {
            let op = match &**kind {
                TokenKind::BinOp(BinOpToken::Plus) => BinOpKind::Add,
                TokenKind::BinOp(BinOpToken::Minus) => BinOpKind::Sub,
                _ => break,
            };
            tokens.next();
            lhs = Expr::new_binary(op, lhs, self.parse_mul(tokens));
        }
        lhs
    }

    pub fn parse_mul<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        let mut lhs = self.parse_primary(tokens);
        while let Some(Token { kind, .. }) = tokens.peek() {
            let op = match &**kind {
                TokenKind::BinOp(BinOpToken::Mul) => BinOpKind::Mul,
                TokenKind::BinOp(BinOpToken::Div) => BinOpKind::Div,
                _ => break,
            };
            tokens.next();
            lhs = Expr::new_binary(op, lhs, self.parse_mul(tokens));
        }
        lhs
    }

    pub fn parse_primary<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        match tokens.next() {
            Some(token) => match *token.kind {
                TokenKind::Num(num) => Expr::new_num(num),
                TokenKind::OpenDelim(DelimToken::Paren) => {
                    let expr = self.parse_expr(tokens);
                    tokens.expect(TokenKind::CloseDelim(DelimToken::Paren));
                    expr
                }
                _ => panic!("Expected a number, found {:?}", token.kind),
            },
            None => panic!("No more tokens available in parse_primary"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ExprKind {
    Binary(Binary),
    Num(isize),
}

impl Expr {
    pub fn new_binary(kind: BinOpKind, lhs: Expr, rhs: Expr) -> Self {
        Self {
            kind: ExprKind::Binary(Binary::new(kind, Box::new(lhs), Box::new(rhs))),
        }
    }

    pub fn new_num(num: isize) -> Self {
        Self {
            kind: ExprKind::Num(num),
        }
    }
}

// Binary Operation ( e.g. `1 + 2`, `3 - 4` )
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Binary {
    pub kind: BinOpKind,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl Binary {
    pub fn new(kind: BinOpKind, lhs: Box<Expr>, rhs: Box<Expr>) -> Self {
        Self { kind, lhs, rhs }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, TokenStream};

    #[test]
    fn test_parse_expr() {
        let input = "1 + 2 * 3 - 4 / 5";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_expr(&mut token_stream);

        let expected = bin(
            BinOpKind::Sub,
            bin(BinOpKind::Add, num(1), bin(BinOpKind::Mul, num(2), num(3))),
            bin(BinOpKind::Div, num(4), num(5)),
        );

        assert_eq!(expr.kind, expected.kind);

        let input = "1 * (2 + 3)";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_expr(&mut token_stream);
        let expected = bin(BinOpKind::Mul, num(1), bin(BinOpKind::Add, num(2), num(3)));
        assert_eq!(expr.kind, expected.kind);
    }

    fn bin(op: BinOpKind, lhs: Expr, rhs: Expr) -> Expr {
        Expr::new_binary(op, lhs, rhs)
    }

    fn num(n: isize) -> Expr {
        Expr::new_num(n)
    }
}
