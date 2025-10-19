use crate::error::{CompileError, Span};

#[derive(Debug)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
  Int(i32),
  Ident(String),
  String(String),
  Plus,
  Minus,
  Star,
  Slash,
  Assign,
  Semicolon,
  LParen,
  RParen,
  LT,
  GT,
  LTE,
  GTE,
  Bang,
  Print,
  PrintLn,
  Comment,
  Exit,
  Eof,
}

pub struct Lexer {
  input: Vec<char>,
  pos: usize,
  line: usize,
  col: usize,
}

impl Lexer {
  pub fn new(input: &str) -> Self {
    Lexer { input: input.chars().collect(), pos: 0, line: 1, col: 1 }
  }

  pub fn tokenize(&mut self) -> Result<Vec<Token>, CompileError> {
    let mut tokens = Vec::new();
    loop {
      let token = self.next_token()?;
      if token.kind == TokenKind::Eof {
        tokens.push(token);
        break;
      }
      tokens.push(token);
    }
    Ok(tokens)
  }

  fn peek(&self) -> Option<char> {
    if self.pos < self.input.len() { Some(self.input[self.pos]) } else { None }
  }

  fn next(&mut self) {
    if let Some(ch) = self.peek() {
      if ch == '\n' {
        self.line += 1;
        self.col = 1;
      } else {
        self.col += 1;
      }
      self.pos += 1;
    }
  }

  fn skip_comment(&mut self) {
    self.next();
    while let Some(ch) = self.peek() {
      if ch == '\n' {
        break;
      }
      self.next();
    }
  }

  fn skip_whitespace(&mut self) {
    while let Some(ch) = self.peek() {
      match ch {
        ch if ch.is_whitespace() => {
          self.next();
        }
        '#' => {
          self.skip_comment();
        }
        _ => break,
      }
    }
  }

  fn read_number(&mut self) -> Result<(i32, usize), CompileError> {
    let start_line = self.line;
    let start_col = self.col;
    let mut num = String::new();
    let mut length = 0;
    while let Some(ch) = self.peek() {
      if ch.is_numeric() {
        num.push(ch);
        length += 1;
        self.next();
      } else {
        break;
      }
    }

    num.parse::<i32>().map(|n| (n, length)).map_err(|e| CompileError::LexError {
      msg: format!("Invalid number: {}", e),
      span: Span::new(start_line, start_col, length),
    })
  }

  fn read_identifier(&mut self) -> (String, usize) {
    let mut id = String::new();
    let mut length = 0;

    while let Some(ch) = self.peek() {
      if ch.is_alphanumeric() || ch == '_' {
        id.push(ch);
        length += 1;
        self.next();
      } else {
        break;
      }
    }
    (id, length)
  }

  fn read_string(&mut self) -> Result<(String, usize), CompileError> {
    let start_line = self.line;
    let start_col = self.col - 1;
    let mut length = 1;
    let mut s = String::new();
    while let Some(ch) = self.peek() {
      length += 1;
      match ch {
        '"' => {
          self.next();
          return Ok((s, length));
        }
        '\\' => {
          self.next();
          if let Some(escaped) = self.peek() {
            length += 1;
            let real_ch = match escaped {
              'n' => '\n',
              't' => '\t',
              '"' => '"',
              '\\' => '\\',
              other => other,
            };
            s.push(real_ch);
            self.next();
          } else {
            return Err(CompileError::LexError {
              msg: "Unterminated escape in string".to_string(),
              span: Span::new(start_line, start_col, length),
            });
          }
        }
        _ => {
          s.push(ch);
          self.next();
        }
      }
    }

    Err(CompileError::LexError {
      msg: "Unterminated string literal".to_string(),
      span: Span::new(start_line, start_col, length),
    })
  }

  fn next_token(&mut self) -> Result<Token, CompileError> {
    self.skip_whitespace();

    let start_line = self.line;
    let start_col = self.col;

    match self.peek() {
      None => Ok(Token { kind: TokenKind::Eof, span: Span::new(start_line, start_col, 0) }),
      Some(ch) => {
        let (kind, length) = match ch {
          '+' => {
            self.next();
            (TokenKind::Plus, 1)
          }
          '-' => {
            self.next();
            (TokenKind::Minus, 1)
          }
          '*' => {
            self.next();
            (TokenKind::Star, 1)
          }
          '/' => {
            self.next();
            (TokenKind::Slash, 1)
          }
          '=' => {
            self.next();
            (TokenKind::Assign, 1)
          }
          ';' => {
            self.next();
            (TokenKind::Semicolon, 1)
          }
          '(' => {
            self.next();
            (TokenKind::LParen, 1)
          }
          ')' => {
            self.next();
            (TokenKind::RParen, 1)
          }
          '!' => {
            self.next();
            (TokenKind::Bang, 1)
          }
          '<' => {
            self.next();
            if let Some('=') = self.peek() {
              self.next();
              (TokenKind::LTE, 2)
            } else {
              (TokenKind::LT, 1)
            }
          }
          '>' => {
            self.next();
            if let Some('=') = self.peek() {
              self.next();
              (TokenKind::GTE, 2)
            } else {
              (TokenKind::GT, 1)
            }
          }
          '"' => {
            self.next();
            let (s, len) = self.read_string()?;
            (TokenKind::String(s), len)
          }
          _ if ch.is_numeric() => {
            let (n, len) = self.read_number()?;
            (TokenKind::Int(n), len)
          }
          _ if ch.is_alphabetic() => {
            let (id, len) = self.read_identifier();
            let kind = match id.as_str() {
              "exit" => TokenKind::Exit,
              "print" => TokenKind::Print,
              "println" => TokenKind::PrintLn,
              _ => TokenKind::Ident(id),
            };
            (kind, len)
          }
          _ => {
            return Err(CompileError::LexError {
              msg: format!("Unexpected character: '{}'", ch),
              span: Span::new(start_line, start_col, 1),
            });
          }
        };

        Ok(Token { kind, span: Span::new(start_line, start_col, length) })
      }
    }
  }
}
