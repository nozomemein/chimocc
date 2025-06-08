use std::io::{BufWriter, Write};

use crate::parser::{BinOpKind, Expr, ExprKind, UnOp};

pub struct Generator {}

#[allow(unused)]
impl Generator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen_head<W: Write>(f: &mut BufWriter<W>, expr: Expr) -> Result<(), std::io::Error> {
        writeln!(f, ".intel_syntax noprefix")?;
        writeln!(f, ".global main")?;
        writeln!(f, "main:")?;

        Self::gen_expr(f, expr)?;
        writeln!(f, "  pop rax")?;
        writeln!(f, "  ret")?;

        // Specify NX (No eXecute) for the stack
        writeln!(f, ".section .note.GNU-stack,\"\",@progbits")?;
        Ok(())
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
            ExprKind::Unary(UnOp::Plus, expr) => Self::gen_expr(f, *expr)?,
            ExprKind::Unary(UnOp::Minus, expr) => {
                Self::gen_expr(f, *expr)?;
                writeln!(f, "  pop rdi")?;
                writeln!(f, "  mov rax, 0")?;
                // Ideally, this can be optimized by re-parsing the expression,
                // but we don't care about the performance here.
                // -x := 0 - x
                writeln!(f, "  sub rax, rdi")?;
                writeln!(f, "  push rax")?;
            }
        }
        Ok(())
    }
}
