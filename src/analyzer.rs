use std::collections::BTreeMap;

use crate::parser::{
    BinOpKind, Binary, Expr, ExprKind, Program, ProgramKind, Stmt, StmtKind, UnOp,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Analyzer {
    offset: usize,
}

impl Analyzer {
    pub fn new() -> Self {
        Self { offset: 0 }
    }

    pub fn down_program(&mut self, program: Program) -> ConvProgram {
        let mut conv_program = ConvProgram::new();
        let mut lvar_map = BTreeMap::new();
        for stmt in program.into_iter() {
            match stmt {
                ProgramKind::Stmt(stmt) => {
                    conv_program.push_stmt(self.down_stmt(stmt, &mut lvar_map))
                }
            }
        }
        conv_program
    }

    pub fn down_stmt(&mut self, stmt: Stmt, lvar_map: &mut BTreeMap<String, usize>) -> ConvStmt {
        match stmt.kind {
            StmtKind::Expr(expr) => ConvStmt::new_expr(self.down_expr(expr, lvar_map)),
            StmtKind::Return(expr) => ConvStmt::new_return(self.down_expr(expr, lvar_map)),
            // do nothing
            StmtKind::If(cond, then, els) => ConvStmt::new_if(
                self.down_expr(cond, lvar_map),
                self.down_stmt(*then, lvar_map),
                els.map(|els| self.down_stmt(*els, lvar_map)),
            ),
            StmtKind::While(cond, body) => ConvStmt::new_while(
                self.down_expr(cond, lvar_map),
                self.down_stmt(*body, lvar_map),
            ),
            StmtKind::For(init, cond, inc, then) => ConvStmt::new_for(
                init.map(|init| self.down_expr(init, lvar_map)),
                cond.map(|cond| self.down_expr(cond, lvar_map)),
                inc.map(|inc| self.down_expr(inc, lvar_map)),
                self.down_stmt(*then, lvar_map),
            ),
        }
    }
    pub fn down_expr(&mut self, expr: Expr, lvar_map: &mut BTreeMap<String, usize>) -> ConvExpr {
        match expr.kind {
            // `a >= b ` : `b <= a`
            ExprKind::Binary(Binary {
                kind: BinOpKind::Ge,
                lhs,
                rhs,
            }) => ConvExpr::new_binary(
                ConvBinOpKind::Le,
                self.down_expr(*rhs, lvar_map),
                self.down_expr(*lhs, lvar_map),
            ),
            // `a > b` : `b < a`
            ExprKind::Binary(Binary {
                kind: BinOpKind::Gt,
                lhs,
                rhs,
            }) => ConvExpr::new_binary(
                ConvBinOpKind::Lt,
                self.down_expr(*rhs, lvar_map),
                self.down_expr(*lhs, lvar_map),
            ),
            // do nothing
            ExprKind::Binary(Binary { kind, lhs, rhs }) => ConvExpr::new_binary(
                ConvBinOpKind::new(kind).unwrap(),
                self.down_expr(*lhs, lvar_map),
                self.down_expr(*rhs, lvar_map),
            ),
            // do nothing
            ExprKind::Num(n) => ConvExpr::new_num(n),
            // substitute `-x` into `0-x`
            ExprKind::Unary(UnOp::Minus, operand) => ConvExpr::new_binary(
                ConvBinOpKind::Sub,
                ConvExpr::new_num(0),
                self.down_expr(*operand, lvar_map),
            ),
            // do nothing
            ExprKind::Assign(lhs, rhs) => ConvExpr::new_assign(
                self.down_expr(*lhs, lvar_map),
                self.down_expr(*rhs, lvar_map),
            ),

            // do nothing
            ExprKind::Unary(UnOp::Plus, operand) => self.down_expr(*operand, lvar_map),
            ExprKind::Ident(ident) => ConvExpr::new_lvar(ident, &mut self.offset, lvar_map),
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

    #[allow(dead_code)]
    pub fn with_vec(components: Vec<ConvProgramKind>) -> Self {
        Self { components }
    }

    pub fn push_stmt(&mut self, stmt: ConvStmt) {
        self.components.push(ConvProgramKind::Stmt(stmt));
    }
}

impl IntoIterator for ConvProgram {
    type Item = ConvProgramKind;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.components.into_iter()
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

    pub fn new_return(expr: ConvExpr) -> Self {
        Self {
            kind: ConvStmtKind::Return(expr),
        }
    }

    pub fn new_if(cond: ConvExpr, then: ConvStmt, els: Option<ConvStmt>) -> Self {
        Self {
            kind: ConvStmtKind::If(cond, Box::new(then), els.map(Box::new)),
        }
    }

    pub fn new_while(cond: ConvExpr, body: ConvStmt) -> Self {
        Self {
            kind: ConvStmtKind::While(cond, Box::new(body)),
        }
    }

    pub fn new_for(
        init: Option<ConvExpr>,
        cond: Option<ConvExpr>,
        inc: Option<ConvExpr>,
        then: ConvStmt,
    ) -> Self {
        Self {
            kind: ConvStmtKind::For(init, cond, inc, Box::new(then)),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvStmtKind {
    Expr(ConvExpr),
    Return(ConvExpr),
    If(ConvExpr, Box<ConvStmt>, Option<Box<ConvStmt>>),
    While(ConvExpr, Box<ConvStmt>),
    For(
        Option<ConvExpr>,
        Option<ConvExpr>,
        Option<ConvExpr>,
        Box<ConvStmt>,
    ),
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

    pub fn new_lvar(
        name: String,
        new_offset: &mut usize,
        lvar_map: &mut BTreeMap<String, usize>,
    ) -> Self {
        let offset = match lvar_map.get(&name) {
            Some(offset) => *offset,
            None => {
                *new_offset += 8;
                lvar_map.insert(name, *new_offset);
                *new_offset
            }
        };
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
    /// Used like `mov rax, [rbp - offset]` to load the value from the stack.
    /// note: The stack grows from higher to lower addresses.
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

    fn conv_lvar_with_offset(offset: usize) -> ConvExpr {
        ConvExpr {
            kind: ConvExprKind::Lvar(Lvar { offset }),
        }
    }

    #[test]
    fn test_down_expr_num() {
        let expr = num(42);
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        assert_eq!(conv, conv_num(42));
    }

    #[test]
    fn test_down_expr_unary_minus() {
        let expr = unary(UnOp::Minus, num(10));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_bin(ConvBinOpKind::Sub, conv_num(0), conv_num(10));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_unary_plus() {
        let expr = unary(UnOp::Plus, num(10));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_num(10);
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary() {
        let expr = bin(BinOpKind::Add, num(1), num(2));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_bin(ConvBinOpKind::Add, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_nested() {
        let expr = unary(UnOp::Minus, unary(UnOp::Minus, num(5)));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
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
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_bin(ConvBinOpKind::Le, conv_num(2), conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_gt() {
        let expr = bin(BinOpKind::Gt, num(1), num(2));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_bin(ConvBinOpKind::Lt, conv_num(2), conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_eq() {
        let expr = bin(BinOpKind::Eq, num(1), num(2));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_bin(ConvBinOpKind::Eq, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_ne() {
        let expr = bin(BinOpKind::Ne, num(1), num(2));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_bin(ConvBinOpKind::Ne, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_assign() {
        let expr = assign(ident("a"), num(1));
        let conv = Analyzer::new().down_expr(expr, &mut BTreeMap::new());
        let expected = conv_assign(conv_lvar_with_offset(8), conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_stmt_expr() {
        let stmt = Stmt::expr(assign(ident("a"), num(1)));
        let conv = Analyzer::new().down_stmt(stmt, &mut BTreeMap::new());
        let expected = ConvStmt::new_expr(conv_assign(conv_lvar_with_offset(8), conv_num(1)));
        assert_eq!(conv, expected);

        let stmt = Stmt::ret(num(1));
        let conv = Analyzer::new().down_stmt(stmt, &mut BTreeMap::new());
        let expected = ConvStmt::new_return(conv_num(1));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_program() {
        let program = Program::with_vec(vec![ProgramKind::Stmt(Stmt::expr(assign(
            ident("a"),
            num(1),
        )))]);
        let conv = Analyzer::new().down_program(program);
        let expected = ConvProgram::with_vec(vec![ConvProgramKind::Stmt(ConvStmt::new_expr(
            conv_assign(conv_lvar_with_offset(8), conv_num(1)),
        ))]);
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_local_variable_test() {
        let mut analyzer = Analyzer::new();
        let program = Program::with_vec(vec![
            ProgramKind::Stmt(Stmt::expr(assign(ident("a"), assign(ident("k"), num(1))))),
            ProgramKind::Stmt(Stmt::expr(assign(ident("c"), num(3)))),
            ProgramKind::Stmt(Stmt::expr(bin(BinOpKind::Div, ident("a"), ident("k")))),
        ]);
        let converted_program = analyzer.down_program(program);
        assert_eq!(
            converted_program,
            ConvProgram::with_vec(vec![
                ConvProgramKind::Stmt(ConvStmt::new_expr(conv_assign(
                    conv_lvar_with_offset(8),
                    conv_assign(conv_lvar_with_offset(16), conv_num(1))
                ))),
                ConvProgramKind::Stmt(ConvStmt::new_expr(conv_assign(
                    conv_lvar_with_offset(24),
                    conv_num(3)
                ))),
                ConvProgramKind::Stmt(ConvStmt::new_expr(conv_bin(
                    ConvBinOpKind::Div,
                    conv_lvar_with_offset(8),
                    conv_lvar_with_offset(16)
                ))),
            ])
        )
    }

    #[test]
    fn test_down_stmt_if() {
        let stmt = Stmt::new_if(num(1), Stmt::expr(num(2)), Some(Stmt::expr(num(3))));
        let conv = Analyzer::new().down_stmt(stmt, &mut BTreeMap::new());
        let expected = ConvStmt::new_if(
            conv_num(1),
            ConvStmt::new_expr(conv_num(2)),
            Some(ConvStmt::new_expr(conv_num(3))),
        );
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_stmt_while() {
        let stmt = Stmt::new_while(num(1), Stmt::expr(num(2)));
        let conv = Analyzer::new().down_stmt(stmt, &mut BTreeMap::new());
        let expected = ConvStmt::new_while(conv_num(1), ConvStmt::new_expr(conv_num(2)));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_stmt_for() {
        let stmt = Stmt::new_for(Some(num(1)), Some(num(2)), Some(num(3)), Stmt::expr(num(4)));
        let conv = Analyzer::new().down_stmt(stmt, &mut BTreeMap::new());
        let expected = ConvStmt::new_for(
            Some(ConvExpr::new_num(1)),
            Some(ConvExpr::new_num(2)),
            Some(ConvExpr::new_num(3)),
            ConvStmt::new_expr(ConvExpr::new_num(4)),
        );
        assert_eq!(conv, expected);
    }
}
