use std::io::{BufWriter, Write};

use crate::analyzer::{
    ConvBinOpKind, ConvBinary, ConvExpr, ConvExprKind, ConvProgram, ConvProgramKind, ConvStmt,
    ConvStmtKind,
};

pub struct Generator {}

#[allow(unused)]
impl Generator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn gen_head<W: Write>(
        f: &mut BufWriter<W>,
        program: ConvProgram,
    ) -> Result<(), std::io::Error> {
        writeln!(f, ".intel_syntax noprefix")?;
        writeln!(f, ".global main")?;
        writeln!(f, "main:")?;

        // prologue for the main function frame
        writeln!(f, "  push rbp")?;
        writeln!(f, "  mov rbp, rsp")?;

        // TODO: ideally we should count the number of local variables and push them to the stack here.
        writeln!(f, "  sub rsp, {}", 26 * 8)?; // Reserve space for all possible local variables a-z (each 8 bytes, 26 variables).
        for component in program.into_iter() {
            match component {
                ConvProgramKind::Stmt(stmt) => {
                    Self::gen_stmt(f, stmt);
                }
            }
            writeln!(f, "  pop rax")?;
        }

        // epilogue
        writeln!(f, "  mov rsp, rbp")?;
        writeln!(f, "  pop rbp")?;

        writeln!(f, "  ret")?;

        // Specify NX (No eXecute) for the stack
        writeln!(f, ".section .note.GNU-stack,\"\",@progbits")?;
        Ok(())
    }

    pub fn gen_stmt<W: Write>(f: &mut BufWriter<W>, stmt: ConvStmt) -> Result<(), std::io::Error> {
        match stmt.kind {
            ConvStmtKind::Expr(expr) => {
                Self::gen_expr(f, expr);
            }
        }
        Ok(())
    }

    pub fn gen_expr<W: Write>(f: &mut BufWriter<W>, expr: ConvExpr) -> Result<(), std::io::Error> {
        match expr.kind {
            ConvExprKind::Num(num) => {
                writeln!(f, "  push {}", num)?;
            }
            ConvExprKind::Binary(binary) => {
                Self::gen_binary(f, binary);
            }
            ConvExprKind::Lvar(lvar) => todo!(),
            ConvExprKind::Assign(lhs, rhs) => todo!(),
        }
        Ok(())
    }

    pub fn gen_binary<W: Write>(
        f: &mut BufWriter<W>,
        binary: ConvBinary,
    ) -> Result<(), std::io::Error> {
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
        Ok(())
    }
}
