use std::collections::HashMap;

use crate::parser::{BinOp, Expr, Stmt, Type};

pub struct CodeGen {
  strings: HashMap<String, String>,
  vars: HashMap<String, i32>,
  var_types: HashMap<String, Type>,
  var_offset: i32,
  output: Vec<String>,
  reg_counter: usize,
}

impl CodeGen {
  pub fn new() -> Self {
    CodeGen {
      strings: HashMap::new(),
      vars: HashMap::new(),
      var_types: HashMap::new(),
      var_offset: 0,
      output: Vec::new(),
      reg_counter: 0,
    }
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
    self.output.push("  .text".to_string());
    self.output.push("  .globl main".to_string());
    self.output.push("main:".to_string());
    self.output.push("  addi sp, sp, -128 # Set up stack frame".to_string());

    for stmt in stmts {
      self.gen_stmt(stmt);
    }

    self.output.push("".to_string());
    self.output.push("  # Exit with code 0".to_string());
    self.output.push("  li a1, 0 # Exit code 0".to_string());
    self.output.push("  li a0, 17 # Syscall 17: exit2".to_string());
    self.output.push("  ecall".to_string());

    let mut final_out = Vec::new();
    final_out.push("".to_string());
    final_out.push("  .data".to_string());
    self.gen_strings(&mut final_out);
    final_out.push("".to_string());
    final_out.append(&mut self.output);

    final_out.join("\n")
  }

  fn gen_stmt(&mut self, stmt: &Stmt) {
    match stmt {
      Stmt::Assign { var, expr } => {
        let reg = self.gen_expr(expr);
        let expr_type = self.infer_type(expr);
        self.var_types.insert(var.clone(), expr_type);
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
      Stmt::Print { expr } => self.gen_print(expr, false),
      Stmt::PrintLn { expr } => self.gen_print(expr, true),
    }
  }

  fn infer_type(&mut self, expr: &Expr) -> Type {
    match expr {
      Expr::Int(_) => Type::Int,
      Expr::String(_) => Type::String,
      Expr::Var(name) => self
        .var_types
        .get(name)
        .cloned()
        .unwrap_or_else(|| panic!("Compiler: Variable '{}' type not tracked", name)),
      Expr::BinOp { .. } => Type::Int,
    }
  }

  fn gen_print(&mut self, expr: &Expr, newline: bool) {
    let expr_type = self.infer_type(expr);
    match expr_type {
      Type::String => {
        if let Expr::String(s) = expr {
          let label = self.ensure_string_label(s);
          self.output.push(format!("  la a1, {} # Load string {}", label, Self::escape_asciz(s)));
          self.output.push("  li a0, 4 # Syscall 4: print_string".to_string());
          self.output.push("  ecall".to_string());
        } else if let Expr::Var(name) = expr {
          let reg = self.gen_expr(expr);
          self.output.push(format!("  mv a1, {} # Load string from variable {}", reg, name));
          self.output.push("  li a0, 4 # Syscall 4: print_string".to_string());
          self.output.push("  ecall".to_string());
        }
      }
      Type::Int => {
        let reg = self.gen_expr(expr);
        self.output.push(format!("  mv a1, {} # Expression to print", reg));
        self.output.push("  li a0, 1 # Syscall 1: print_int".to_string());
        self.output.push("  ecall".to_string());
      }
    }

    if newline {
      self.output.push("  li a1, '\\n' # Load newline char".to_string());
      self.output.push("  li a0, 11 # Syscall 11: print_character".to_string());
      self.output.push("  ecall".to_string());
    }

    self.output.push("".to_string());
  }

  fn ensure_string_label(&mut self, s: &String) -> String {
    if !self.strings.contains_key(s) {
      self.strings.insert(s.clone(), format!("str{}", self.strings.len()));
    }
    self.strings.get(s).unwrap().clone()
  }

  fn gen_strings(&self, out: &mut Vec<String>) {
    let pairs: Vec<_> = self.strings.iter().collect();
    for (s, label) in pairs {
      let escaped = Self::escape_asciz(s);
      // NOTE: venus uses .asciiz instead of .asciz for some reason
      out.push(format!("{}: .asciiz \"{}\"", label, escaped));
    }
  }

  fn escape_asciz(s: &str) -> String {
    let mut escaped = String::new();

    for c in s.chars() {
      match c {
        '\\' => escaped.push_str("\\\\"),
        '\"' => escaped.push_str("\\\""),
        '\n' => escaped.push_str("\\n"),
        '\t' => escaped.push_str("\\t"),
        '\r' => escaped.push_str("\\r"),
        c if c.is_ascii_graphic() || c == ' ' => escaped.push(c),
        c => {
          use std::fmt::Write;
          write!(&mut escaped, "\\\\x{:02X}", c as u32).unwrap();
        }
      }
    }

    escaped
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
          panic!("Compiler: Variable '{}' not stored", var);
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
          BinOp::LT => {
            self.output.push(format!("  slt {}, {}, {} # left < right", result_reg, left_reg, right_reg))
          }
          BinOp::LTE => {
            self.output.push(format!("  slt {}, {}, {} # right < left", result_reg, right_reg, left_reg));
            self.output.push(format!("  xori {}, {}, 1 # For <=", result_reg, result_reg))
          }
          BinOp::GT => {
            self.output.push(format!("  slt {}, {}, {} # right < left", result_reg, right_reg, left_reg))
          }
          BinOp::GTE => {
            self.output.push(format!("  slt {}, {}, {} # left < right", result_reg, left_reg, right_reg));
            self.output.push(format!("  xori {}, {}, 1 # For >=", result_reg, result_reg))
          }
        }

        result_reg
      }
      Expr::String(s) => {
        let reg = self.alloc_reg();
        let label = self.ensure_string_label(s);
        self.output.push(format!("  la {}, {} # Store string {:?}", reg, label, Self::escape_asciz(s)));

        reg
      }
    }
  }
}
