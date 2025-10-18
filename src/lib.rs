use crate::{codegen::CodeGen, error::CompileError, lexer::Lexer, parser::Parser};

pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;

pub fn compile(src: &str) -> Result<String, CompileError> {
  let lexer = Lexer::new(src);
  let mut parser = Parser::new(lexer);
  let ast = parser.parse_program()?;

  let mut codegen = CodeGen::new();
  Ok(codegen.generate(&ast))
}
