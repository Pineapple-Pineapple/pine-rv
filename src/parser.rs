use crate::{error::CompileError, lexer::Token};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
  Int,
  String,
}

#[derive(Debug)]
pub enum Expr {
  Int(i32),
  Var(String),
  String(String),
  BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
}

impl Expr {
  pub fn get_type(&self, var_types: &HashMap<String, Type>) -> Result<Type, CompileError> {
    match self {
      Expr::Int(_) => Ok(Type::Int),
      Expr::String(_) => Ok(Type::String),
      Expr::Var(name) => var_types
        .get(name)
        .cloned()
        .ok_or_else(|| CompileError::ParseError(format!("Unknown variable: {}", name))),
      Expr::BinOp { op, left, right } => {
        let left_type = left.get_type(var_types)?;
        let right_type = right.get_type(var_types)?;

        if left_type != Type::Int || right_type != Type::Int {
          return Err(CompileError::ParseError(format!(
            "Binary operation {:?} requires integer operands",
            op
          )));
        }

        Ok(Type::Int)
      }
    }
  }
}

#[derive(Debug)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  GT,
  LT,
  GTE,
  LTE,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
  Lowest,
  Comp,
  AddSub,
  MulDiv,
}

#[derive(Debug)]
pub enum Stmt {
  Assign { var: String, expr: Expr },
  Print { expr: Expr },
  PrintLn { expr: Expr },
  Exit(Option<Expr>),
}

pub struct Parser {
  tokens: Vec<Token>,
  pos: usize,
  var_types: HashMap<String, Type>,
}

impl Parser {
  pub fn new(tokens: Vec<Token>) -> Self {
    Parser { tokens, pos: 0, var_types: HashMap::new() }
  }

  pub fn parse(&mut self) -> Result<Vec<Stmt>, CompileError> {
    let mut stmts = Vec::new();
    while self.peek() != &Token::Eof {
      stmts.push(self.parse_statement()?);
    }
    Ok(stmts)
  }

  fn peek(&self) -> &Token {
    self.tokens.get(self.pos).unwrap_or(&Token::Eof)
  }

  fn next(&mut self) {
    if self.pos < self.tokens.len() {
      self.pos += 1;
    }
  }

  fn parse_statement(&mut self) -> Result<Stmt, CompileError> {
    match self.peek() {
      Token::Ident(name) => {
        let var = name.clone();
        self.next();
        if self.peek() == &Token::Assign {
          self.next();
          let expr = self.parse_expr()?;
          let expr_type = expr.get_type(&self.var_types)?;
          self.var_types.insert(var.clone(), expr_type);
          if self.peek() == &Token::Semicolon {
            self.next();
          }
          Ok(Stmt::Assign { var, expr })
        } else {
          Err(CompileError::ParseError("Expected '='".to_string()))
        }
      }

      Token::Exit => {
        self.next();
        let exit_code = if !matches!(self.peek(), Token::Semicolon | Token::Eof) {
          let expr = self.parse_expr()?;
          let expr_type = expr.get_type(&self.var_types)?;

          if expr_type != Type::Int {
            return Err(CompileError::ParseError("Exit code must be an integer".to_string()));
          }

          Some(expr)
        } else {
          None
        };

        if self.peek() == &Token::Semicolon {
          self.next();
        }

        Ok(Stmt::Exit(exit_code))
      }

      Token::Print | Token::PrintLn => {
        let is_newline = matches!(self.peek(), Token::PrintLn);
        self.next();

        let expr = self.parse_expr()?;

        if self.peek() == &Token::Semicolon {
          self.next();
        }

        if is_newline { Ok(Stmt::PrintLn { expr }) } else { Ok(Stmt::Print { expr }) }
      }

      _ => Err(CompileError::ParseError(format!("Unexpected token: {:?}", self.peek()))),
    }
  }

  fn precedence(token: &Token) -> Prec {
    match token {
      Token::Star | Token::Slash => Prec::MulDiv,
      Token::Plus | Token::Minus => Prec::AddSub,
      Token::LT | Token::LTE | Token::GT | Token::GTE => Prec::Comp,
      _ => Prec::Lowest,
    }
  }

  fn parse_expr(&mut self) -> Result<Expr, CompileError> {
    self.parse_expr_proc(Prec::Lowest)
  }

  fn parse_expr_proc(&mut self, prec: Prec) -> Result<Expr, CompileError> {
    let mut left = self.parse_primary()?;

    while Self::precedence(self.peek()) > prec {
      let op_token = self.peek().clone();
      let op = match op_token {
        Token::Plus => BinOp::Add,
        Token::Minus => BinOp::Sub,
        Token::Star => BinOp::Mul,
        Token::Slash => BinOp::Div,
        Token::GT => BinOp::GT,
        Token::GTE => BinOp::GTE,
        Token::LT => BinOp::LT,
        Token::LTE => BinOp::LTE,
        _ => break,
      };

      self.next();

      let right = self.parse_expr_proc(Self::precedence(&op_token))?;

      let is_comp = matches!(op, BinOp::GT | BinOp::GTE | BinOp::LT | BinOp::LTE);
      let is_next_comp = matches!(self.peek(), Token::GT | Token::GTE | Token::LT | Token::LTE);
      if is_comp && is_next_comp {
        return Err(CompileError::ParseError("Chained comparisons are not allowed".to_string()));
      }

      left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
    }

    Ok(left)
  }

  fn parse_primary(&mut self) -> Result<Expr, CompileError> {
    match self.peek() {
      Token::Int(n) => {
        let val = *n;
        self.next();
        Ok(Expr::Int(val))
      }
      Token::Ident(name) => {
        let var = name.clone();
        self.next();
        Ok(Expr::Var(var))
      }
      Token::LParen => {
        self.next();
        let expr = self.parse_expr()?;
        if self.peek() == &Token::RParen {
          self.next();
        } else {
          return Err(CompileError::ParseError("Expected ')".to_string()));
        }
        Ok(expr)
      }
      Token::String(s) => {
        let val = s.clone();
        self.next();
        Ok(Expr::String(val))
      }
      _ => Err(CompileError::ParseError(format!("Unexpected token: {:?}", self.peek()))),
    }
  }
}
