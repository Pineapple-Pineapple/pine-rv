use std::collections::HashMap;

use crate::parser::{BinOp, Expr, Stmt, Type, UnaryOp};

pub struct CodeGen {
  strings: HashMap<String, String>,
  vars: HashMap<String, i32>,
  var_types: HashMap<String, Type>,
  var_offset: i32,
  output: Vec<String>,
  reg_pool: Vec<String>,
  while_counter: usize,
  if_counter: usize,
  temp_stack_offset: i32,
}

impl CodeGen {
  pub fn new() -> Self {
    CodeGen {
      strings: HashMap::new(),
      vars: HashMap::new(),
      var_types: HashMap::new(),
      var_offset: 0,
      output: Vec::new(),
      reg_pool: ["t0", "t1", "t2", "t3", "t4", "t5", "t6"].iter().map(|&r| r.to_string()).collect(),
      while_counter: 0,
      if_counter: 0,
      temp_stack_offset: 128,
    }
  }

  #[inline]
  fn nl(&mut self) {
    self.output.push(String::new());
  }

  fn alloc_reg(&mut self) -> String {
    if let Some(reg) = self.reg_pool.pop() {
      reg
    } else {
      let victim = "t0".to_string();
      let stack_loc = self.temp_stack_offset;
      self.temp_stack_offset += 4;
      self.output.push(format!("  sw {}, {}(sp) # Spill {} to stack", victim, stack_loc, victim));
      victim
    }
  }

  fn free_reg(&mut self, reg: String) {
    self.reg_pool.push(reg);
  }

  pub fn generate(&mut self, stmts: &Vec<Stmt>) -> String {
    self.output.push("  .text".to_string());
    self.output.push("  .globl main".to_string());
    self.output.push("main:".to_string());
    self.output.push("  addi sp, sp, -512 # Set up stack frame".to_string());
    self.nl();

    for stmt in stmts {
      self.gen_stmt(stmt);
      self.nl();
    }

    self.output.push("  # Exit with code 0".to_string());
    self.output.push("  li a1, 0 # Exit code 0".to_string());
    self.output.push("  li a0, 17 # Syscall 17: exit2".to_string());
    self.output.push("  ecall".to_string());

    let mut final_out = Vec::new();
    final_out.push("  .data".to_string());
    self.gen_strings(&mut final_out);
    final_out.push(String::new());
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
        self.free_reg(reg);
      }
      Stmt::Exit(code) => {
        if let Some(expr) = code {
          let reg = self.gen_expr(expr);
          self.output.push(format!("  mv a1, {} # exit code", reg));
          self.output.push("  li a0, 17 # Syscall 17: exit2".to_string());
          self.free_reg(reg);
        } else {
          self.output.push("  li a0, 10 # Syscall 10: exit".to_string());
        }

        self.output.push("  ecall".to_string());
      }
      Stmt::Print { expr } => self.gen_print(expr, false),
      Stmt::PrintLn { expr } => match expr {
        Some(expr) => self.gen_print(expr, true),
        None => {
          self.output.push("  li a1, '\\n' # Load newline char".to_string());
          self.output.push("  li a0, 11 # Syscall 11: print_character".to_string());
          self.output.push("  ecall".to_string());
        }
      },
      Stmt::While { condition, body } => {
        let while_count = self.while_counter;
        let while_start = format!("W{}_start", while_count);
        let while_end = format!("W{}_end", while_count);
        self.while_counter += 1;
        self.output.push(format!("{}:", while_start));
        let reg = self.gen_expr(condition);
        self.output.push(format!("  beq {}, x0, {}", reg, while_end));
        self.free_reg(reg);
        for stmt in body {
          self.gen_stmt(stmt);
        }
        self.output.push(format!("  j {}", while_start));
        self.output.push(format!("{}:", while_end));
      }
      Stmt::If { condition, then_body, else_body } => {
        let if_count = self.if_counter;
        let else_label = format!("IF{}_else", if_count);
        let end_label = format!("IF{}_end", if_count);
        self.if_counter += 1;

        let reg = self.gen_expr(condition);

        if else_body.is_some() {
          self
            .output
            .push(format!("  beq {}, x0, {} # Jump to else branch if condition is false", reg, else_label));
        } else {
          self.output.push(format!("  beq {}, x0, {} # Jump to end if condition is false", reg, end_label));
        }
        self.free_reg(reg);

        for stmt in then_body {
          self.gen_stmt(stmt);
        }

        if else_body.is_some() {
          self.output.push(format!("  j {} # Skip else block", end_label));
          self.output.push(format!("{}:", else_label));

          for stmt in else_body.as_ref().unwrap() {
            self.gen_stmt(stmt);
          }
        }

        self.output.push(format!("{}:", end_label));
      }
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
      Expr::UnaryOp { .. } => Type::Int,
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
          self.free_reg(reg);
        }
      }
      Type::Int => {
        let reg = self.gen_expr(expr);
        self.output.push(format!("  mv a1, {} # Expression to print", reg));
        self.output.push("  li a0, 1 # Syscall 1: print_int".to_string());
        self.output.push("  ecall".to_string());
        self.free_reg(reg);
      }
    }

    if newline {
      self.output.push("  li a1, '\\n' # Load newline char".to_string());
      self.output.push("  li a0, 11 # Syscall 11: print_character".to_string());
      self.output.push("  ecall".to_string());
    }
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
            self.output.push(format!("  slt {}, {}, {} # left < right", result_reg, left_reg, right_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::LTE => {
            self.output.push(format!("  slt {}, {}, {} # right < left", result_reg, right_reg, left_reg));
            self.output.push(format!("  xori {}, {}, 1 # For <=", result_reg, result_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::GT => {
            self.output.push(format!("  slt {}, {}, {} # right < left", result_reg, right_reg, left_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::GTE => {
            self.output.push(format!("  slt {}, {}, {} # left < right", result_reg, left_reg, right_reg));
            self.output.push(format!("  xori {}, {}, 1 # For >=", result_reg, result_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::Eq => {
            self
              .output
              .push(format!("  sub {}, {}, {} # diff = left - right", result_reg, left_reg, right_reg));
            self.output.push(format!("  sltu {}, x0, {} # (diff != 0)", result_reg, result_reg));
            self
              .output
              .push(format!("  xori {}, {}, 1 # !(diff != 0) -> (diff == 0)", result_reg, result_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }

          BinOp::Neq => {
            self
              .output
              .push(format!("  sub {}, {}, {} # diff = left - right", result_reg, left_reg, right_reg));
            self.output.push(format!("  sltu {}, x0, {} # diff != 0", result_reg, result_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::AND => {
            self.output.push(format!("  and {}, {}, {} # Logical and", result_reg, left_reg, right_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::OR => {
            self.output.push(format!("  or {}, {}, {} # Logical or", result_reg, left_reg, right_reg));
            self.output.push(format!("  sltu {}, x0, {} # Normalize result", result_reg, result_reg));
          }
          BinOp::BitAnd => {
            self.output.push(format!("  and {}, {}, {}", result_reg, left_reg, right_reg));
          }
          BinOp::BitOr => {
            self.output.push(format!("  or {}, {}, {}", result_reg, left_reg, right_reg));
          }
          BinOp::BitXor => {
            self.output.push(format!("  xor {}, {}, {}", result_reg, left_reg, right_reg));
          }
          BinOp::LShift => {
            self.output.push(format!("  sll {}, {}, {}", result_reg, left_reg, right_reg));
          }
          BinOp::RShift => {
            self.output.push(format!("  sra {}, {}, {}", result_reg, left_reg, right_reg));
          }
        }

        self.free_reg(left_reg);
        self.free_reg(right_reg);

        result_reg
      }
      Expr::String(s) => {
        let reg = self.alloc_reg();
        let label = self.ensure_string_label(s);
        self.output.push(format!("  la {}, {} # Store string {:?}", reg, label, Self::escape_asciz(s)));

        reg
      }
      Expr::UnaryOp { op, expr } => {
        let reg = self.gen_expr(expr);
        match op {
          UnaryOp::Not => {
            self.output.push(format!("  sltiu {}, {}, 1", reg, reg));
          }
          UnaryOp::Neg => {
            self.output.push(format!("  sub {}, x0, {}", reg, reg));
          }
          UnaryOp::BitNot => {
            self.output.push(format!("  not {}, {}", reg, reg));
          }
        }

        reg
      }
    }
  }
}
