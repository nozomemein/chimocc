use crate::lexer::{BinOpToken, DelimToken, Token, TokenKind, TokenStream};

pub struct Parser {}

#[allow(unused)]
impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_program<I>(&self, tokens: &mut TokenStream<'_, I>) -> Program
    where
        I: Clone + Iterator<Item = Token>,
    {
        let mut program = Program::new();
        while !tokens.at_eof() {
            program.push_stmt(self.parse_stmt(tokens));
            tokens.expect(TokenKind::Semi); // stmt must be followed by a semicolon
        }
        tokens.expect(TokenKind::Eof);
        assert!(tokens.next().is_none());
        program
    }

    pub fn parse_stmt<I>(&self, tokens: &mut TokenStream<'_, I>) -> Stmt
    where
        I: Clone + Iterator<Item = Token>,
    {
        let expr = self.parse_expr(tokens);
        Stmt::expr(expr)
    }

    pub fn parse_expr<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        self.parse_assign(tokens)
    }

    pub fn parse_assign<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        let lhs = self.parse_equality(tokens);
        let (kind, pos) = match tokens.peek() {
            Some(Token { kind, pos }) => (kind, pos),
            None => panic!("Expected token, but none"),
        };
        match **kind {
            TokenKind::Eq => {
                tokens.next();
                Expr::new_assign(lhs, self.parse_assign(tokens))
            }
            _ => lhs,
        }
    }

    pub fn parse_equality<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        let mut lhs = self.parse_relational(tokens);
        while let Some(Token { kind, .. }) = tokens.peek() {
            let op = match &**kind {
                TokenKind::EtEq => BinOpKind::Eq,
                TokenKind::Ne => BinOpKind::Ne,
                _ => break,
            };
            tokens.next();
            lhs = Expr::new_binary(op, lhs, self.parse_relational(tokens));
        }
        lhs
    }

    pub fn parse_relational<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        let mut lhs = self.parse_add(tokens);

        while let Some(Token { kind, .. }) = tokens.peek() {
            let op = match &**kind {
                TokenKind::Lt => BinOpKind::Lt,
                TokenKind::Le => BinOpKind::Le,
                TokenKind::Gt => BinOpKind::Gt,
                TokenKind::Ge => BinOpKind::Ge,
                _ => break,
            };
            tokens.next();
            lhs = Expr::new_binary(op, lhs, self.parse_add(tokens));
        }
        lhs
    }

    pub fn parse_add<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
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
        let mut lhs = self.parse_unary(tokens);
        while let Some(Token { kind, .. }) = tokens.peek() {
            let op = match &**kind {
                TokenKind::BinOp(BinOpToken::Mul) => BinOpKind::Mul,
                TokenKind::BinOp(BinOpToken::Div) => BinOpKind::Div,
                _ => break,
            };
            tokens.next();
            lhs = Expr::new_binary(op, lhs, self.parse_unary(tokens));
        }
        lhs
    }

    pub fn parse_unary<I>(&self, tokens: &mut TokenStream<'_, I>) -> Expr
    where
        I: Clone + Iterator<Item = Token>,
    {
        match tokens.peek() {
            Some(Token { kind, .. }) => match &**kind {
                TokenKind::BinOp(BinOpToken::Plus) => {
                    tokens.next();
                    Expr::new_unary(UnOp::Plus, self.parse_unary(tokens))
                }
                TokenKind::BinOp(BinOpToken::Minus) => {
                    tokens.next();
                    Expr::new_unary(UnOp::Minus, self.parse_unary(tokens))
                }
                _ => self.parse_primary(tokens),
            },
            None => panic!("Expected token, but none"),
        }
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
                TokenKind::Ident(ident) => Expr::new_ident(ident),
                _ => panic!("Expected a number, found {:?}", token.kind),
            },
            None => panic!("No more tokens available in parse_primary"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Program {
    pub components: Vec<ProgramKind>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_vec(components: Vec<ProgramKind>) -> Self {
        Self { components }
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        self.components.push(ProgramKind::Stmt(stmt));
    }

    pub fn into_iter(self) -> impl Iterator<Item = ProgramKind> {
        self.components.into_iter()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ProgramKind {
    Stmt(Stmt),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
}

impl Stmt {
    pub fn expr(expr: Expr) -> Self {
        Self {
            kind: StmtKind::Expr(expr),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum StmtKind {
    Expr(Expr),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ExprKind {
    Binary(Binary),
    Num(isize),
    Unary(UnOp, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Ident(String),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum UnOp {
    Plus,
    Minus,
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

    pub fn new_unary(kind: UnOp, expr: Expr) -> Self {
        Self {
            kind: ExprKind::Unary(kind, Box::new(expr)),
        }
    }

    pub fn new_assign(lhs: Expr, rhs: Expr) -> Self {
        Self {
            kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
        }
    }

    pub fn new_ident(ident: String) -> Self {
        Self {
            kind: ExprKind::Ident(ident),
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
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
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

    #[test]
    fn test_unary_op() {
        let input = "-10 + 20";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_expr(&mut token_stream);
        let expected = bin(BinOpKind::Add, unary(UnOp::Minus, num(10)), num(20));
        assert_eq!(expr.kind, expected.kind);

        let input = "-(-10)";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_expr(&mut token_stream);
        let expected = unary(UnOp::Minus, unary(UnOp::Minus, num(10)));
        assert_eq!(expr.kind, expected.kind);

        let input = "- -10";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_expr(&mut token_stream);
        let expected = unary(UnOp::Minus, unary(UnOp::Minus, num(10)));
        assert_eq!(expr.kind, expected.kind);
    }

    #[test]
    fn test_parse_equality() {
        let input = "1 == 2";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_equality(&mut token_stream);
        let expected = bin(BinOpKind::Eq, num(1), num(2));
        assert_eq!(expr.kind, expected.kind);

        let input = "1 != 2";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_equality(&mut token_stream);
        let expected = bin(BinOpKind::Ne, num(1), num(2));
        assert_eq!(expr.kind, expected.kind);
    }

    #[test]
    fn test_parse_relational() {
        let input = "1 < 2";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_relational(&mut token_stream);
        let expected = bin(BinOpKind::Lt, num(1), num(2));
        assert_eq!(expr.kind, expected.kind);

        let input = "1 <= 2";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_relational(&mut token_stream);
        let expected = bin(BinOpKind::Le, num(1), num(2));
        assert_eq!(expr.kind, expected.kind);

        let input = "1 > 2";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_relational(&mut token_stream);
        let expected = bin(BinOpKind::Gt, num(1), num(2));
        assert_eq!(expr.kind, expected.kind);

        let input = "1 >= 2";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let expr = parser.parse_relational(&mut token_stream);
        let expected = bin(BinOpKind::Ge, num(1), num(2));
        assert_eq!(expr.kind, expected.kind);
    }

    #[test]
    fn test_parse_stmt() {
        let input = "1 + 2; 3 + 4;";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let program = parser.parse_program(&mut token_stream);
        let expected = Program::with_vec(vec![
            stmt(bin(BinOpKind::Add, num(1), num(2))),
            stmt(bin(BinOpKind::Add, num(3), num(4))),
        ]);
        assert_eq!(program, expected);
    }

    #[test]
    fn test_parse_assign() {
        let input = "a = 1; b = 3;";
        let tokens = Lexer::new(input).tokenize();
        let mut token_stream = TokenStream::new(tokens.into_iter(), input);
        let parser = Parser::new();
        let program = parser.parse_program(&mut token_stream);
        let expected = Program::with_vec(vec![
            stmt(assign(ident("a"), num(1))),
            stmt(assign(ident("b"), num(3))),
        ]);
        assert_eq!(program, expected);
    }

    fn bin(op: BinOpKind, lhs: Expr, rhs: Expr) -> Expr {
        Expr::new_binary(op, lhs, rhs)
    }

    fn num(n: isize) -> Expr {
        Expr::new_num(n)
    }

    fn unary(op: UnOp, expr: Expr) -> Expr {
        Expr::new_unary(op, expr)
    }

    fn stmt(expr: Expr) -> ProgramKind {
        ProgramKind::Stmt(Stmt::expr(expr))
    }

    fn assign(lhs: Expr, rhs: Expr) -> Expr {
        Expr::new_assign(lhs, rhs)
    }

    fn ident(ident: &str) -> Expr {
        Expr::new_ident(ident.to_string())
    }
}
