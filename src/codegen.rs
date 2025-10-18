use std::collections::HashMap;

use crate::parser::{BinOp, Expr, Stmt};

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

  pub fn generate(&mut self, stmts: &Vec<Stmt>) -> String {
    self.output.push("  .data".to_string());
    self.output.push("".to_string());
    self.output.push("  .text".to_string());
    self.output.push("  .globl main".to_string());
    self.output.push("main:".to_string());
    self.output.push("  addi sp, sp, -128 # Set up stack frame".to_string());

    for stmt in stmts {
      self.gen_stmt(stmt);
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
        let offset = *self.vars.get(var).unwrap();
        self.output.push(format!("  sw {}, {}(sp) # Store variable {}", reg, offset, var));
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

        self.output.push("  ecall".to_string());
        self.output.push("".to_string());
      }
    }
  }

  fn gen_expr(&mut self, expr: &Expr) -> String {
    match expr {
      Expr::Int(n) => {
        let reg = self.alloc_reg();
        self.output.push(format!("  li {}, {} # Load immediate {}", reg, n, n));
        reg
      }
      Expr::Var(var) => {
        if let Some(&offset) = self.vars.get(var) {
          let reg = self.alloc_reg();
          self.output.push(format!("  lw {}, {}(sp) # Load variable {}", reg, offset, var));
          reg
        } else {
          panic!("Unknown variable '{}'", var);
        }
      }
      Expr::BinOp { op, left, right } => {
        let left_reg = self.gen_expr(left);
        let right_reg = self.gen_expr(right);
        let result_reg = self.alloc_reg();

        match op {
          BinOp::Add => {
            self.output.push(format!("  add {}, {}, {} # addition", result_reg, left_reg, right_reg))
          }
          BinOp::Sub => {
            self.output.push(format!("  sub {}, {}, {} # subtraction", result_reg, left_reg, right_reg))
          }
          BinOp::Mul => {
            self.output.push(format!("  mul {}, {}, {} # multiplication", result_reg, left_reg, right_reg))
          }
          BinOp::Div => {
            self.output.push(format!("  div {}, {}, {} # division", result_reg, left_reg, right_reg))
          }
        }

        result_reg
      }
    }
  }
}
