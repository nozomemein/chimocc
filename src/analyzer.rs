use crate::parser::{
    BinOpKind, Binary, Expr, ExprKind, Program, ProgramKind, Stmt, StmtKind, UnOp,
};

pub struct Analyzer {}

impl Analyzer {
    pub fn down_program(program: Program) -> ConvProgram {
        let mut conv_program = ConvProgram::new();
        for stmt in program.into_iter() {
            match stmt {
                ProgramKind::Stmt(stmt) => conv_program.push_stmt(Self::down_stmt(stmt)),
            }
        }
        conv_program
    }

    pub fn down_stmt(stmt: Stmt) -> ConvStmt {
        match stmt.kind {
            StmtKind::Expr(expr) => ConvStmt::new_expr(Self::down_expr(expr)),
        }
    }
    pub fn down_expr(expr: Expr) -> ConvExpr {
        match expr.kind {
            // `a >= b ` : `b <= a`
            ExprKind::Binary(Binary {
                kind: BinOpKind::Ge,
                lhs,
                rhs,
            }) => ConvExpr::new_binary(
                ConvBinOpKind::Le,
                Self::down_expr(*rhs),
                Self::down_expr(*lhs),
            ),
            // `a > b` : `b < a`
            ExprKind::Binary(Binary {
                kind: BinOpKind::Gt,
                lhs,
                rhs,
            }) => ConvExpr::new_binary(
                ConvBinOpKind::Lt,
                Self::down_expr(*rhs),
                Self::down_expr(*lhs),
            ),
            // do nothing
            ExprKind::Binary(Binary { kind, lhs, rhs }) => ConvExpr::new_binary(
                ConvBinOpKind::new(kind).unwrap(),
                Self::down_expr(*lhs),
                Self::down_expr(*rhs),
            ),
            // do nothing
            ExprKind::Num(n) => ConvExpr::new_num(n),
            // substitute `-x` into `0-x`
            ExprKind::Unary(UnOp::Minus, operand) => ConvExpr::new_binary(
                ConvBinOpKind::Sub,
                ConvExpr::new_num(0),
                Self::down_expr(*operand),
            ),
            // do nothing
            ExprKind::Assign(lhs, rhs) => {
                ConvExpr::new_assign(Self::down_expr(*lhs), Self::down_expr(*rhs))
            }

            // do nothing
            ExprKind::Unary(UnOp::Plus, operand) => Self::down_expr(*operand),
            ExprKind::Ident(ident) => ConvExpr::new_lvar(ident),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConvProgram {
    pub components: Vec<ConvProgramKind>,
}

impl ConvProgram {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn with_vec(components: Vec<ConvProgramKind>) -> Self {
        Self { components }
    }

    pub fn push_stmt(&mut self, stmt: ConvStmt) {
        self.components.push(ConvProgramKind::Stmt(stmt));
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvProgramKind {
    Stmt(ConvStmt),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConvStmt {
    pub kind: ConvStmtKind,
}

impl ConvStmt {
    pub fn new_expr(expr: ConvExpr) -> Self {
        Self {
            kind: ConvStmtKind::Expr(expr),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvStmtKind {
    Expr(ConvExpr),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConvExpr {
    pub kind: ConvExprKind,
}
impl ConvExpr {
    pub fn new_binary(kind: ConvBinOpKind, lhs: ConvExpr, rhs: ConvExpr) -> Self {
        Self {
            kind: ConvExprKind::Binary(ConvBinary::new(kind, Box::new(lhs), Box::new(rhs))),
        }
    }

    pub fn new_num(num: isize) -> Self {
        Self {
            kind: ConvExprKind::Num(num),
        }
    }

    pub fn new_assign(lhs: ConvExpr, rhs: ConvExpr) -> Self {
        Self {
            kind: ConvExprKind::Assign(Box::new(lhs), Box::new(rhs)),
        }
    }

    pub fn new_lvar(name: String) -> Self {
        // FIXME: Currentlty we assume the name is one character and the position in the stackcan be decided by the index of the alphabet.
        assert!(name.len() == 1);
        let index = ('a'..='z')
            .position(|c| c == name.chars().next().unwrap())
            .expect("Expected alphabet here");
        let offset = index * 8;
        Self {
            kind: ConvExprKind::Lvar(Lvar { offset }),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvExprKind {
    Binary(ConvBinary),
    Num(isize),
    Lvar(Lvar),
    Assign(Box<ConvExpr>, Box<ConvExpr>),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Lvar {
    pub offset: usize,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvBinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    /// The `==` operator (equality)
    Eq,
    /// The `<=` operator (less than or equal to)
    Le,
    /// The `<` operator (less than)
    Lt,
    /// The `!=` operator (Not equal to)
    Ne,
}

impl ConvBinOpKind {
    pub fn new(kind: BinOpKind) -> Option<Self> {
        match kind {
            BinOpKind::Add => Some(ConvBinOpKind::Add),
            BinOpKind::Sub => Some(ConvBinOpKind::Sub),
            BinOpKind::Mul => Some(ConvBinOpKind::Mul),
            BinOpKind::Div => Some(ConvBinOpKind::Div),
            BinOpKind::Eq => Some(ConvBinOpKind::Eq),
            BinOpKind::Le => Some(ConvBinOpKind::Le),
            BinOpKind::Lt => Some(ConvBinOpKind::Lt),
            BinOpKind::Ge => None,
            BinOpKind::Gt => None,
            BinOpKind::Ne => Some(ConvBinOpKind::Ne),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConvBinary {
    pub kind: ConvBinOpKind,
    pub lhs: Box<ConvExpr>,
    pub rhs: Box<ConvExpr>,
}

impl ConvBinary {
    pub fn new(kind: ConvBinOpKind, lhs: Box<ConvExpr>, rhs: Box<ConvExpr>) -> Self {
        Self { kind, lhs, rhs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{BinOpKind, Expr, UnOp};

    fn num(n: isize) -> Expr {
        Expr::new_num(n)
    }
    fn unary(op: UnOp, expr: Expr) -> Expr {
        Expr::new_unary(op, expr)
    }
    fn bin(op: BinOpKind, lhs: Expr, rhs: Expr) -> Expr {
        Expr::new_binary(op, lhs, rhs)
    }
    fn assign(lhs: Expr, rhs: Expr) -> Expr {
        Expr::new_assign(lhs, rhs)
    }
    fn ident(name: &str) -> Expr {
        Expr::new_ident(name.to_string())
    }
    fn conv_num(n: isize) -> ConvExpr {
        ConvExpr::new_num(n)
    }
    fn conv_bin(op: ConvBinOpKind, lhs: ConvExpr, rhs: ConvExpr) -> ConvExpr {
        ConvExpr::new_binary(op, lhs, rhs)
    }

    fn conv_assign(lhs: ConvExpr, rhs: ConvExpr) -> ConvExpr {
        ConvExpr::new_assign(lhs, rhs)
    }
    fn conv_lvar(name: String) -> ConvExpr {
        ConvExpr::new_lvar(name)
    }

    #[test]
    fn test_down_expr_num() {
        let expr = num(42);
        let conv = Analyzer::down_expr(expr);
        assert_eq!(conv, conv_num(42));
    }

    #[test]
    fn test_down_expr_unary_minus() {
        let expr = unary(UnOp::Minus, num(10));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Sub, conv_num(0), conv_num(10));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_unary_plus() {
        let expr = unary(UnOp::Plus, num(10));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_num(10);
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary() {
        let expr = bin(BinOpKind::Add, num(1), num(2));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Add, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_nested() {
        let expr = unary(UnOp::Minus, unary(UnOp::Minus, num(5)));
        let conv = Analyzer::down_expr(expr);
        // -(-5) => 0 - (0 - 5)
        let expected = conv_bin(
            ConvBinOpKind::Sub,
            conv_num(0),
            conv_bin(ConvBinOpKind::Sub, conv_num(0), conv_num(5)),
        );
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_ge() {
        let expr = bin(BinOpKind::Ge, num(1), num(2));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Le, conv_num(2), conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_gt() {
        let expr = bin(BinOpKind::Gt, num(1), num(2));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Lt, conv_num(2), conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_eq() {
        let expr = bin(BinOpKind::Eq, num(1), num(2));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Eq, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_ne() {
        let expr = bin(BinOpKind::Ne, num(1), num(2));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Ne, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_assign() {
        let expr = assign(ident("a"), num(1));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_assign(conv_lvar("a".to_string()), conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_stmt_expr() {
        let stmt = Stmt::expr(assign(ident("a"), num(1)));
        let conv = Analyzer::down_stmt(stmt);
        let expected = ConvStmt::new_expr(conv_assign(conv_lvar("a".to_string()), conv_num(1)));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_program() {
        let program = Program::with_vec(vec![ProgramKind::Stmt(Stmt::expr(assign(
            ident("a"),
            num(1),
        )))]);
        let conv = Analyzer::down_program(program);
        let expected = ConvProgram::with_vec(vec![ConvProgramKind::Stmt(ConvStmt::new_expr(
            conv_assign(conv_lvar("a".to_string()), conv_num(1)),
        ))]);
        assert_eq!(conv, expected);
    }
}
