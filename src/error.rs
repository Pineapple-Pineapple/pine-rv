use std::fmt;

#[derive(Debug)]
pub enum CompileError {
  LexError(String),
  ParseError(String),
}

impl fmt::Display for CompileError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      | CompileError::LexError(msg) => write!(f, "Lexer error: {}", msg),
      | CompileError::ParseError(msg) => write!(f, "Parser error: {}", msg),
    }
  }
}

impl std::error::Error for CompileError {}
