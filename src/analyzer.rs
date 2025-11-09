use crate::parser::{BinOpKind, Binary, Expr, ExprKind, UnOp};

pub struct Analyzer {}

impl Analyzer {
    pub fn down_expr(expr: Expr) -> ConvExpr {
        match expr.kind {
            // `a >= b ` : `b <= a`
            ExprKind::Binary(Binary {
                kind: BinOpKind::Ge,
                lhs,
                rhs,
            }) => ConvExpr::new_binary(
                ConvBinOpKind::Le,
                Self::down_expr(*lhs),
                Self::down_expr(*rhs),
            ),
            // `a > b` : `b < a`
            ExprKind::Binary(Binary {
                kind: BinOpKind::Gt,
                lhs,
                rhs,
            }) => ConvExpr::new_binary(
                ConvBinOpKind::Lt,
                Self::down_expr(*lhs),
                Self::down_expr(*rhs),
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
            ExprKind::Unary(UnOp::Plus, operand) => Self::down_expr(*operand),
        }
    }
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
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConvExprKind {
    Binary(ConvBinary),
    Num(isize),
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
            BinOpKind::Lt => Some(ConvBinOpKind::Le),
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
    fn conv_num(n: isize) -> ConvExpr {
        ConvExpr::new_num(n)
    }
    fn conv_bin(op: ConvBinOpKind, lhs: ConvExpr, rhs: ConvExpr) -> ConvExpr {
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
        let expected = conv_bin(ConvBinOpKind::Le, conv_num(1), conv_num(2));
        assert_eq!(conv, expected);
    }

    #[test]
    fn test_down_expr_binary_gt() {
        let expr = bin(BinOpKind::Gt, num(1), num(2));
        let conv = Analyzer::down_expr(expr);
        let expected = conv_bin(ConvBinOpKind::Lt, conv_num(1), conv_num(2));
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
}
