use crate::{
  error::{CompileError, Span},
  lexer::{Token, TokenKind},
};
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
  UnaryOp { op: UnaryOp, expr: Box<Expr> },
}

impl Expr {
  pub fn get_type(&self, var_types: &HashMap<String, Type>) -> Result<Type, CompileError> {
    match self {
      Expr::Int(_) => Ok(Type::Int),
      Expr::String(_) => Ok(Type::String),
      Expr::Var(name) => var_types
        .get(name)
        .cloned()
        .ok_or_else(|| CompileError::ParseError { msg: format!("Unknown variable: {}", name), span: None }),
      Expr::BinOp { op, left, right } => {
        let left_type = left.get_type(var_types)?;
        let right_type = right.get_type(var_types)?;

        if left_type != Type::Int || right_type != Type::Int {
          return Err(CompileError::ParseError {
            msg: format!("Binary operation {:?} requires integer operands", op),
            span: None,
          });
        }

        Ok(Type::Int)
      }
      Expr::UnaryOp { op, expr } => {
        let expr_type = expr.get_type(var_types)?;

        if expr_type != Type::Int {
          return Err(CompileError::ParseError {
            msg: format!("Unary operation {:?} requires an integer operand", op),
            span: None,
          });
        }

        Ok(Type::Int)
      }
    }
  }
}

#[derive(Debug)]
pub enum UnaryOp {
  Not,
  Neg,
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
  Unary,
}

#[derive(Debug)]
pub enum Stmt {
  Assign { var: String, expr: Expr },
  Print { expr: Expr },
  PrintLn { expr: Option<Expr> },
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

  pub fn parse(&mut self) -> Result<(Vec<Stmt>, HashMap<String, Type>), CompileError> {
    let mut stmts = Vec::new();
    while self.peek().kind != TokenKind::Eof {
      stmts.push(self.parse_statement()?);
    }
    Ok((stmts, self.var_types.clone()))
  }

  fn peek(&self) -> &Token {
    static EOF_TOKEN: Token = Token { kind: TokenKind::Eof, span: Span { line: 0, col: 0, length: 0 } };
    self.tokens.get(self.pos).unwrap_or(&EOF_TOKEN)
  }

  fn next(&mut self) {
    if self.pos < self.tokens.len() {
      self.pos += 1;
    }
  }

  fn parse_statement(&mut self) -> Result<Stmt, CompileError> {
    match &self.peek().kind {
      TokenKind::Ident(name) => {
        let var = name.clone();
        self.next();
        if self.peek().kind == TokenKind::Assign {
          self.next();
          let expr = self.parse_expr()?;
          let expr_type = expr.get_type(&self.var_types)?;
          self.var_types.insert(var.clone(), expr_type);
          if self.peek().kind == TokenKind::Semicolon {
            self.next();
          }
          Ok(Stmt::Assign { var, expr })
        } else {
          Err(CompileError::ParseError { msg: "Expected '='".to_string(), span: Some(self.peek().span) })
        }
      }

      TokenKind::Exit => {
        self.next();
        let exit_code = if !matches!(self.peek().kind, TokenKind::Semicolon | TokenKind::Eof) {
          let expr = self.parse_expr()?;
          let expr_type = expr.get_type(&self.var_types)?;

          if expr_type != Type::Int {
            return Err(CompileError::ParseError {
              msg: "Exit code must be an integer".to_string(),
              span: Some(self.peek().span),
            });
          }

          Some(expr)
        } else {
          None
        };

        if self.peek().kind == TokenKind::Semicolon {
          self.next();
        }

        Ok(Stmt::Exit(exit_code))
      }

      TokenKind::Print | TokenKind::PrintLn => {
        let is_newline = matches!(self.peek().kind, TokenKind::PrintLn);
        self.next();
        if is_newline && matches!(self.peek().kind, TokenKind::Semicolon | TokenKind::Eof) {
          if self.peek().kind == TokenKind::Semicolon {
            self.next();
          }
          return Ok(Stmt::PrintLn { expr: None });
        }

        let expr = self.parse_expr()?;

        if self.peek().kind == TokenKind::Semicolon {
          self.next();
        }

        if is_newline { Ok(Stmt::PrintLn { expr: Some(expr) }) } else { Ok(Stmt::Print { expr }) }
      }

      _ => Err(CompileError::ParseError {
        msg: format!("Unexpected token: {:?}", self.peek().kind),
        span: Some(self.peek().span),
      }),
    }
  }

  fn precedence(token: &Token) -> Prec {
    match token.kind {
      TokenKind::Star | TokenKind::Slash => Prec::MulDiv,
      TokenKind::Plus | TokenKind::Minus => Prec::AddSub,
      TokenKind::LT | TokenKind::LTE | TokenKind::GT | TokenKind::GTE => Prec::Comp,
      _ => Prec::Lowest,
    }
  }

  fn parse_expr(&mut self) -> Result<Expr, CompileError> {
    match self.peek().kind {
      TokenKind::Bang | TokenKind::Minus => self.parse_unary(),
      _ => self.parse_expr_prec(Prec::Lowest),
    }
  }

  fn parse_unary(&mut self) -> Result<Expr, CompileError> {
    self.parse_expr_prec(Prec::Lowest)
  }

  fn parse_expr_prec(&mut self, prec: Prec) -> Result<Expr, CompileError> {
    let mut left = match self.peek().kind {
      TokenKind::Bang => {
        self.next();
        let expr = self.parse_expr_prec(Prec::Unary)?;
        Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) }
      }
      TokenKind::Minus => {
        self.next();
        let expr = self.parse_expr_prec(Prec::Unary)?;
        Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) }
      }
      _ => self.parse_primary()?,
    };

    while Self::precedence(self.peek()) > prec {
      let op_token = self.peek();
      let op = match op_token.kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::GT => BinOp::GT,
        TokenKind::GTE => BinOp::GTE,
        TokenKind::LT => BinOp::LT,
        TokenKind::LTE => BinOp::LTE,
        _ => break,
      };

      self.next();

      let right = self.parse_expr_prec(Self::precedence(self.peek()))?;

      let is_comp = matches!(op, BinOp::GT | BinOp::GTE | BinOp::LT | BinOp::LTE);
      let is_next_comp =
        matches!(self.peek().kind, TokenKind::GT | TokenKind::GTE | TokenKind::LT | TokenKind::LTE);
      if is_comp && is_next_comp {
        return Err(CompileError::ParseError {
          msg: "Chained comparisons are not allowed".to_string(),
          span: Some(self.peek().span),
        });
      }

      left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
    }

    Ok(left)
  }

  fn parse_primary(&mut self) -> Result<Expr, CompileError> {
    match &self.peek().kind {
      TokenKind::Int(n) => {
        let val = *n;
        self.next();
        Ok(Expr::Int(val))
      }
      TokenKind::Ident(name) => {
        let var = name.clone();
        if !self.var_types.contains_key(&var) {
          return Err(CompileError::ParseError { msg: format!("Variable '{}' not found", var), span: None });
        };
        self.next();
        Ok(Expr::Var(var))
      }
      TokenKind::LParen => {
        self.next();
        let expr = self.parse_expr()?;
        if self.peek().kind == TokenKind::RParen {
          self.next();
        } else {
          return Err(CompileError::ParseError {
            msg: "Expected ')'".to_string(),
            span: Some(self.peek().span),
          });
        }
        Ok(expr)
      }
      TokenKind::String(s) => {
        let val = s.clone();
        self.next();
        Ok(Expr::String(val))
      }
      _ => Err(CompileError::ParseError {
        msg: format!("Unexpected token: {:?}", self.peek().kind),
        span: Some(self.peek().span),
      }),
    }
  }
}
