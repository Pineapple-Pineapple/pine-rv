use std::fmt;

#[derive(Debug)]
pub enum CompileError {
  LexError(String),
}

impl fmt::Display for CompileError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      | CompileError::LexError(msg) => write!(f, "Lexer error: {}", msg),
    }
  }
}

impl std::error::Error for CompileError {}
