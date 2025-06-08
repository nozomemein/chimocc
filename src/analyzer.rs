use crate::parser::{BinOpKind, Binary, Expr, ExprKind, UnOp};

pub struct Analyzer {}

impl Analyzer {
    pub fn down_expr(expr: Expr) -> ConvExpr {
        match expr.kind {
            // do nothing
            ExprKind::Binary(Binary { kind, lhs, rhs }) => {
                ConvExpr::new_binary(kind, Self::down_expr(*lhs), Self::down_expr(*rhs))
            }
            // do nothing
            ExprKind::Num(n) => ConvExpr::new_num(n),
            // substitute `-x` into `0-x`
            ExprKind::Unary(UnOp::Minus, operand) => ConvExpr::new_binary(
                BinOpKind::Sub,
                ConvExpr::new_num(0),
                Self::down_expr(*operand),
            ),

            // do nothing
            ExprKind::Unary(UnOp::Plus, operand) => Self::down_expr(*operand),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConvExpr {
    pub kind: ConvExprKind,
}
impl ConvExpr {
    pub fn new_binary(kind: BinOpKind, lhs: ConvExpr, rhs: ConvExpr) -> Self {
        Self {
            kind: ConvExprKind::Binary(ConvBinary::new(kind, Box::new(lhs), Box::new(rhs))),
        }
    }

    pub fn new_num(num: isize) -> Self {
        Self {
            kind: ConvExprKind::Num(num),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvExprKind {
    Binary(ConvBinary),
    Num(isize),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConvBinary {
    pub kind: BinOpKind,
    pub lhs: Box<ConvExpr>,
    pub rhs: Box<ConvExpr>,
}

impl ConvBinary {
    pub fn new(kind: BinOpKind, lhs: Box<ConvExpr>, rhs: Box<ConvExpr>) -> Self {
        Self { kind, lhs, rhs }
    }
}
