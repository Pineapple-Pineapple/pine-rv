use crate::error::CompileError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
  Int(i32),
  Ident(String),
  Plus,
  Minus,
  Star,
  Slash,
  Assign,
  Semicolon,
  LParen,
  RParen,
  Exit,
  Eof,
}

pub struct Lexer {
  input: Vec<char>,
  pos: usize,
}

impl Lexer {
  pub fn new(input: &str) -> Self {
    Lexer { input: input.chars().collect(), pos: 0 }
  }

  fn peek(&self) -> Option<char> {
    if self.pos < self.input.len() { Some(self.input[self.pos]) } else { None }
  }

  fn next(&mut self) {
    self.pos += 1;
  }

  fn skip_whitespace(&mut self) {
    while let Some(ch) = self.peek() {
      if ch.is_whitespace() {
        self.next();
      } else {
        break;
      }
    }
  }

  fn read_number(&mut self) -> Result<i32, CompileError> {
    let mut num = String::new();
    while let Some(ch) = self.peek() {
      if ch.is_numeric() {
        num.push(ch);
        self.next();
      } else {
        break;
      }
    }
    num.parse::<i32>().map_err(|e| CompileError::LexError(format!("Invalid number: {}", e)))
  }

  fn read_identifier(&mut self) -> String {
    let mut id = String::new();
    while let Some(ch) = self.peek() {
      if ch.is_alphanumeric() || ch == '_' {
        id.push(ch);
        self.next();
      } else {
        break;
      }
    }
    id
  }

  pub fn next_token(&mut self) -> Result<Token, CompileError> {
    self.skip_whitespace();

    match self.peek() {
      None => Ok(Token::Eof),
      Some(ch) => match ch {
        '+' => {
          self.next();
          Ok(Token::Plus)
        }
        '-' => {
          self.next();
          Ok(Token::Minus)
        }
        '*' => {
          self.next();
          Ok(Token::Star)
        }
        '/' => {
          self.next();
          Ok(Token::Slash)
        }
        '=' => {
          self.next();
          Ok(Token::Assign)
        }
        ';' => {
          self.next();
          Ok(Token::Semicolon)
        }
        '(' => {
          self.next();
          Ok(Token::LParen)
        }
        ')' => {
          self.next();
          Ok(Token::RParen)
        }
        _ if ch.is_numeric() => Ok(Token::Int(self.read_number()?)),
        _ if ch.is_alphabetic() => {
          let id = self.read_identifier();
          match id.as_str() {
            "exit" => Ok(Token::Exit),
            _ => Ok(Token::Ident(id)),
          }
        }
        _ => Err(CompileError::LexError(format!("Unexpected character: '{}'", ch))),
      },
    }
  }
}
