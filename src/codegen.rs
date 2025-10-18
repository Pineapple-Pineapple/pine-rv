use std::collections::HashMap;

use crate::parser::{Expr, Stmt};

pub struct CodeGen {
  vars: HashMap<String, i32>,
  var_offset: i32,
  output: Vec<String>,
  reg_counter: usize,
}

impl CodeGen {
  pub fn new() -> Self {
    CodeGen { vars: HashMap::new(), var_offset: 0, output: Vec::new(), reg_counter: 0 }
  }

  fn alloc_reg(&mut self) -> String {
    let reg = match self.reg_counter {
      0 => "t0",
      1 => "t1",
      2 => "t2",
      3 => "t3",
      4 => "t4",
      5 => "t5",
      6 => "t6",
      _ => "t0",
    };
    self.reg_counter = (self.reg_counter + 1) % 7;
    reg.to_string()
  }

  pub fn generate(&mut self, stmts: Vec<Stmt>) -> String {
    self.output.push("  .data".to_string());
    self.output.push("".to_string());
    self.output.push("  .text".to_string());
    self.output.push("  .globl main".to_string());
    self.output.push("main:".to_string());
    self.output.push("  addi sp, sp, -128 # Set up stack frame".to_string());

    for stmt in stmts {
      self.gen_stmt(&stmt);
    }

    self.output.push("".to_string());
    self.output.push("  # Exit with code 0".to_string());
    self.output.push("  li a0, 17 # Syscall 17: exit2".to_string());
    self.output.push("  ecall".to_string());

    self.output.join("\n")
  }

  fn gen_stmt(&mut self, stmt: &Stmt) {
    match stmt {
      Stmt::Assign { var, expr } => {
        let reg = self.gen_expr(expr);
        if !self.vars.contains_key(var) {
          self.vars.insert(var.clone(), self.var_offset);
          self.var_offset += 4;
        }
        let offset = self.vars[var];
        self.output.push(format!("  sw, {}, {}(sp) # Store variable {}", reg, offset, var));
        self.output.push("".to_string());
      }
      Stmt::Exit(code) => {
        if let Some(expr) = code {
          let reg = self.gen_expr(expr);
          self.output.push(format!("  mv a1, {} # exit code", reg));
          self.output.push("  li a0, 17 # Syscall 17: exit2".to_string());
        } else {
          self.output.push("  li a0, 10 # Syscall 10: exit".to_string());
        }

        self.output.push("ecall".to_string());
        self.output.push("".to_string());
      }
    }
  }

  fn gen_expr(&mut self, expr: &Expr) -> String {
    panic!("TODO: Implement gen_expr");
  }
}
