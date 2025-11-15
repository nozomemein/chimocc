use std::io::{BufWriter, Write};

use crate::analyzer::{
    ConvBinOpKind, ConvBinary, ConvExpr, ConvExprKind, ConvProgram, ConvProgramKind, ConvStmt,
    ConvStmtKind, Lvar,
};

pub struct Generator {
    label: usize,
}

#[allow(unused)]
impl Generator {
    pub fn new() -> Self {
        Self { label: 0 }
    }

    pub fn gen_head<W: Write>(
        &mut self,
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
                    self.gen_stmt(f, stmt)?;
                }
            }
            writeln!(f, "  pop rax")?;
        }

        // epilogue
        // FIXME: This is a temporary label for the return statement.
        writeln!(f, ".main_retL:")?;
        writeln!(f, "  mov rsp, rbp")?;
        writeln!(f, "  pop rbp")?;

        writeln!(f, "  ret")?;

        // Specify NX (No eXecute) for the stack
        writeln!(f, ".section .note.GNU-stack,\"\",@progbits")?;
        Ok(())
    }

    pub fn gen_stmt<W: Write>(
        &mut self,
        f: &mut BufWriter<W>,
        stmt: ConvStmt,
    ) -> Result<(), std::io::Error> {
        match stmt.kind {
            ConvStmtKind::Expr(expr) => {
                self.gen_expr(f, expr)?;
            }
            ConvStmtKind::Return(expr) => {
                self.gen_expr(f, expr)?;
                writeln!(f, "  pop rax")?;
                writeln!(f, " jmp .main_retL")?;
            }
            ConvStmtKind::For(..) => todo!(),
            ConvStmtKind::If(cond, then, Some(els)) => {
                let label = self.label();
                self.gen_expr(f, cond)?;
                writeln!(f, "  pop rax")?; // fetch the result of the condition expression
                writeln!(f, "  cmp rax, 0")?; // compare the result with 0. 0 means false, 1 means true.
                writeln!(f, "  je .Lelse{}", label)?; // if cond is false, jump to the else block (je: jump if equal)
                self.gen_stmt(f, *then)?;
                writeln!(f, "  jmp .Lend{}", label)?; // jump to the end of the if block
                writeln!(f, ".Lelse{}:", label)?; // else block
                self.gen_stmt(f, *els)?;
                writeln!(f, ".Lend{}:", label)?; // end of the if block
            }
            ConvStmtKind::If(cond, then, None) => {
                let label = self.label();
                self.gen_expr(f, cond)?;
                writeln!(f, "  pop rax")?; // fetch the result of the condition expression
                writeln!(f, "  cmp rax, 0")?; // compare the result with 0
                writeln!(f, "  je .Lend{}", label)?; // skip body when condition is false
                self.gen_stmt(f, *then)?;
                writeln!(f, ".Lend{}:", label)?; // end of the if block
            }
            ConvStmtKind::While(cond, body) => {
                let label = self.label();
                writeln!(f, ".Lbegin{}:", label)?;
                self.gen_expr(f, cond)?;
                writeln!(f, "  pop rax")?; // fetch the result of the condition expression
                writeln!(f, "  cmp rax, 0")?; // compare the result with 0
                writeln!(f, "  je .Lend{}", label)?; // skip body when condition is false
                self.gen_stmt(f, *body)?;
                writeln!(f, "  jmp .Lbegin{}", label)?; // jump to the beginning of the while loop
                writeln!(f, ".Lend{}:", label)?;
            }
        }
        Ok(())
    }

    pub fn gen_expr<W: Write>(
        &mut self,
        f: &mut BufWriter<W>,
        expr: ConvExpr,
    ) -> Result<(), std::io::Error> {
        match expr.kind {
            ConvExprKind::Num(num) => {
                writeln!(f, "  push {}", num)?;
            }
            ConvExprKind::Binary(binary) => {
                self.gen_binary(f, binary)?;
            }
            ConvExprKind::Lvar(_) => {
                self.gen_lvalue(f, expr)?;

                writeln!(f, "  pop rax")?;
                writeln!(f, "  mov rax, [rax]")?; // fetch the value from the address stored in rax
                writeln!(f, "  push rax")?;
            }
            ConvExprKind::Assign(lhs, rhs) => {
                self.gen_lvalue(f, *lhs)?;
                self.gen_expr(f, *rhs)?;
                writeln!(f, "  pop rdi")?; // rhs's value
                writeln!(f, "  pop rax")?; // lhs's address itself
                writeln!(f, "  mov [rax], rdi")?; // store the rhs's value to the lhs's address
                writeln!(f, "  push rdi")?; // push the rhs's value back to the stack
            }
        }
        Ok(())
    }

    pub fn gen_binary<W: Write>(
        &mut self,
        f: &mut BufWriter<W>,
        binary: ConvBinary,
    ) -> Result<(), std::io::Error> {
        self.gen_expr(f, *binary.lhs)?;
        self.gen_expr(f, *binary.rhs)?;
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

    // fetch the address of the left-hand side of the assignment
    pub fn gen_lvalue<W: Write>(
        &mut self,
        f: &mut BufWriter<W>,
        expr: ConvExpr,
    ) -> Result<(), std::io::Error> {
        match expr.kind {
            ConvExprKind::Lvar(Lvar { offset }) => {
                writeln!(f, "  mov rax, rbp")?;
                writeln!(f, "  sub rax, {}", offset)?;
                writeln!(f, "  push rax")
            }
            _ => panic!("Expected Lvar, but got {:?}", expr.kind),
        }
    }

    // generate a new label, which ensures the uniqueness of the label
    fn label(&mut self) -> usize {
        self.label += 1;
        self.label
    }
}
