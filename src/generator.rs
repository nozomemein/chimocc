use std::io::{BufWriter, Write};

use crate::{
    analyzer::{ConvExpr, ConvExprKind},
    parser::BinOpKind,
};

pub struct Generator {}

#[allow(unused)]
impl Generator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen_head<W: Write>(f: &mut BufWriter<W>, expr: ConvExpr) -> Result<(), std::io::Error> {
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

    pub fn gen_expr<W: Write>(f: &mut BufWriter<W>, expr: ConvExpr) -> Result<(), std::io::Error> {
        match expr.kind {
            ConvExprKind::Num(num) => {
                writeln!(f, "  push {}", num)?;
            }
            ConvExprKind::Binary(binary) => {
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
