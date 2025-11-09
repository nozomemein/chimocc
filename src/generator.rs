use std::io::{BufWriter, Write};

use crate::analyzer::{ConvBinOpKind, ConvExpr, ConvExprKind};

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
                    ConvBinOpKind::Add => writeln!(f, "  add rax, rdi")?,
                    ConvBinOpKind::Sub => writeln!(f, "  sub rax, rdi")?,
                    ConvBinOpKind::Mul => writeln!(f, "  imul rax, rdi")?,
                    ConvBinOpKind::Div => {
                        // rdx-rax = rax
                        writeln!(f, "  cqo")?;
                        // rax = rdx-rax / rdi
                        // rdx = rdx-rax % rdi
                        writeln!(f, "  idiv rdi")?;
                    }
                    ConvBinOpKind::Eq => {
                        writeln!(f, "  cmp rax, rdi")?;
                        // al : lowwer 8bit of rax
                        // al = flag-reg(eq)
                        writeln!(f, "  sete al")?;
                        writeln!(f, "  movzx rax, al")?;
                    }
                    ConvBinOpKind::Ne => {
                        writeln!(f, "  cmp rax, rdi")?;
                        // al = flag-reg(not equal to)
                        writeln!(f, "  setne al")?;
                        writeln!(f, "  movzx rax, al")?;
                    }
                    ConvBinOpKind::Le => {
                        writeln!(f, "  cmp rax, rdi")?;
                        // al = flag-reg(less than or equal to)
                        writeln!(f, "  setle al")?;
                        writeln!(f, "  movzx rax, al")?;
                    }
                    ConvBinOpKind::Lt => {
                        writeln!(f, "  cmp rax, rdi")?;
                        // al = flag-reg(less than)
                        writeln!(f, "  setl al")?;
                        writeln!(f, "  movzx rax, al")?;
                    }
                }
                writeln!(f, "  push rax")?;
            }
        }
        Ok(())
    }
}
