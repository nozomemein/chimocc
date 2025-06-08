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
    fn conv_num(n: isize) -> ConvExpr {
        ConvExpr::new_num(n)
    }
    fn conv_bin(op: BinOpKind, lhs: ConvExpr, rhs: ConvExpr) -> ConvExpr {
        ConvExpr::new_binary(op, lhs, rhs)
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
        let expected = conv_bin(BinOpKind::Sub, conv_num(0), conv_num(10));
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
        let expected = conv_bin(BinOpKind::Add, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_nested() {
        let expr = unary(UnOp::Minus, unary(UnOp::Minus, num(5)));
        let conv = Analyzer::down_expr(expr);
        // -(-5) => 0 - (0 - 5)
        let expected = conv_bin(
            BinOpKind::Sub,
            conv_num(0),
            conv_bin(BinOpKind::Sub, conv_num(0), conv_num(5)),
        );
        assert_eq!(conv, expected);
    }
}
