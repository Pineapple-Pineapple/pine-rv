use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Span {
  pub line: usize,
  pub col: usize,
  pub length: usize,
}

impl Span {
  pub fn new(line: usize, col: usize, length: usize) -> Self {
    Span { line, col, length }
  }
}

#[derive(Debug)]
pub enum CompileError {
  LexError { msg: String, span: Span },
  ParseError { msg: String, span: Option<Span> },
}

impl CompileError {
  pub fn display_with_source(&self, source: &str) -> String {
    match self {
      CompileError::LexError { msg, span } => format_error_with_context("Lexer error", msg, source, *span),
      CompileError::ParseError { msg, span } => {
        if let Some(span) = span {
          format_error_with_context("Parser error", msg, source, *span)
        } else {
          format!("Parser error: {}", msg)
        }
      }
    }
  }
}

fn format_error_with_context(error_type: &str, msg: &str, source: &str, span: Span) -> String {
  let lines: Vec<&str> = source.lines().collect();

  if span.line == 0 || span.line > lines.len() {
    return format!("{}:{}: {}: {}", span.line, span.col, error_type, msg);
  }

  let line_content = lines[span.line - 1];
  let line_num_width = span.line.to_string().len();

  let mut output = String::new();
  output.push_str(&format!("{}:{}: {}: {}\n", span.line, span.col, error_type, msg));
  output.push_str(&format!("{:width$} |\n", "", width = line_num_width));
  output.push_str(&format!("{} | {}\n", span.line, line_content));
  output.push_str(&format!("{:width$} | ", "", width = line_num_width));

  for _ in 0..span.col.saturating_sub(1) {
    output.push(' ');
  }
  for _ in 0..span.length.max(1) {
    output.push('^');
  }

  output
}

impl fmt::Display for CompileError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CompileError::LexError { msg, span } => {
        write!(f, "Lexer error at {}:{}: {}", span.line, span.col, msg)
      }
      CompileError::ParseError { msg, span } => {
        if let Some(span) = span {
          write!(f, "Parser error at {}:{}: {}", span.line, span.col, msg)
        } else {
          write!(f, "Parser error: {}", msg)
        }
      }
    }
  }
}

impl std::error::Error for CompileError {}
