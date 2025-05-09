use std::io::{BufWriter, Write};

use crate::parser::{BinOpKind, Expr, ExprKind};

pub struct Generator {}

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
        }
        Ok(())
    }
}
