use crate::{
  error::CompileError,
  lexer::{Lexer, Token},
};

#[derive(Debug)]
pub enum Expr {
  Int(i32),
  Var(String),
  String(String),
  BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
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
  AddSub,
  MulDiv,
  Comp,
}

#[derive(Debug)]
pub enum Stmt {
  Assign { var: String, expr: Expr },
  Print { expr: Expr },
  PrintLn { expr: Expr },
  Exit(Option<Expr>),
}

pub struct Parser {
  lexer: Lexer,
  current: Token,
}

impl Parser {
  pub fn new(mut lexer: Lexer) -> Self {
    let current = lexer.next_token().unwrap_or(Token::Eof);
    Parser { lexer, current }
  }

  pub fn parse_program(&mut self) -> Result<Vec<Stmt>, CompileError> {
    let mut stmts = Vec::new();
    while self.current != Token::Eof {
      stmts.push(self.parse_statement()?);
    }
    Ok(stmts)
  }

  fn next(&mut self) -> Result<(), CompileError> {
    self.current = self.lexer.next_token()?;
    Ok(())
  }

  fn parse_statement(&mut self) -> Result<Stmt, CompileError> {
    match &self.current {
      Token::Ident(name) => {
        let var = name.clone();
        self.next()?;
        if self.current == Token::Assign {
          self.next()?;
          let expr = self.parse_expr()?;
          if self.current == Token::Semicolon {
            self.next()?;
          }
          Ok(Stmt::Assign { var, expr })
        } else {
          Err(CompileError::ParseError("Expected '='".to_string()))
        }
      }

      Token::Exit => {
        self.next()?;
        let exit_code = if !matches!(self.current, Token::Semicolon | Token::Eof) {
          Some(self.parse_expr()?)
        } else {
          None
        };

        if self.current == Token::Semicolon {
          self.next()?;
        }

        Ok(Stmt::Exit(exit_code))
      }

      Token::Print | Token::PrintLn => {
        let is_newline = matches!(self.current, Token::PrintLn);
        self.next()?;

        let expr = self.parse_expr()?;

        if self.current == Token::Semicolon {
          self.next()?;
        }

        if is_newline { Ok(Stmt::PrintLn { expr }) } else { Ok(Stmt::Print { expr }) }
      }

      _ => Err(CompileError::ParseError(format!("Unexpected token: {:?}", self.current))),
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

    while Self::precedence(&self.current) > prec {
      let op_token = self.current.clone();
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

      self.next()?;

      let right = self.parse_expr_proc(Self::precedence(&op_token))?;

      let is_comp = matches!(op, BinOp::GT | BinOp::GTE | BinOp::LT | BinOp::LTE);
      let is_next_comp = matches!(self.current, Token::GT | Token::GTE | Token::LT | Token::LTE);
      if is_comp && is_next_comp {
        return Err(CompileError::ParseError("Chained comparisons are not allowed".to_string()));
      }

      left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
    }

    Ok(left)
  }

  fn parse_primary(&mut self) -> Result<Expr, CompileError> {
    match &self.current {
      Token::Int(n) => {
        let val = *n;
        self.next()?;
        Ok(Expr::Int(val))
      }
      Token::Ident(name) => {
        let var = name.clone();
        self.next()?;
        Ok(Expr::Var(var))
      }
      Token::LParen => {
        self.next()?;
        let expr = self.parse_expr()?;
        if self.current == Token::RParen {
          self.next()?;
        } else {
          return Err(CompileError::ParseError("Expected ')".to_string()));
        }
        Ok(expr)
      }
      Token::String(s) => {
        let val = s.clone();
        self.next()?;
        Ok(Expr::String(val))
      }
      _ => Err(CompileError::ParseError(format!("Unexpected token: {:?}", self.current))),
    }
  }
}
