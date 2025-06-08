use std::io::{BufWriter, Write};

use crate::parser::{BinOpKind, Expr, ExprKind, UnOp};

pub struct Generator {}

#[allow(unused)]
impl Generator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen_expr<W: Write>(f: &mut BufWriter<W>, expr: Expr) -> Result<(), std::io::Error> {
        match expr.kind {
            ExprKind::Num(num) => {
                writeln!(f, "  push {}", num)?;
            }
            ExprKind::Binary(binary) => {
                Self::gen_expr(f, *binary.lhs)?;
                Self::gen_expr(f, *binary.rhs)?;
                writeln!(f, "  pop rdi")?;
                writeln!(f, "  pop rax")?;
                match binary.kind {
                    BinOpKind::Add => writeln!(f, "  add rax, rdi")?,
                    BinOpKind::Sub => writeln!(f, "  sub rax, rdi")?,
                    BinOpKind::Mul => writeln!(f, "  imul rax, rdi")?,
                    BinOpKind::Div => {
                        // rdx-rax = rax
                        writeln!(f, "  cqo")?;
                        // rax = rdx-rax / rdi
                        // rdx = rdx-rax % rdi
                        writeln!(f, "  idiv rdi")?;
                    }
                }
                writeln!(f, "  push rax")?;
            }
            ExprKind::Unary(unary, expr) => {
                // wip
                Self::gen_expr(f, *expr)?;
                writeln!(f, "  pop rax")?;
                match unary {
                    UnOp::Plus => writeln!(f, "  push rax")?,
                    UnOp::Minus => writeln!(f, "  neg rax")?,
                }
            }
        }
        Ok(())
    }
}
