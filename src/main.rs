use std::fs;
use std::path::PathBuf;
use std::process;

use clap::Parser as ClapParser;

use riscv_compiler::{codegen::CodeGen, lexer::Lexer, parser::Parser};

#[derive(ClapParser, Debug)]
#[command(name = "riscv-compiler")]
#[command(about = "A compiler for the Pine language targeting RISC-V assembly", long_about = None)]
#[command(version)]
struct Args {
  /// Input file (.pine)
  #[arg(value_name = "FILE")]
  input: PathBuf,

  /// Output file (.s). If not specified, uses input filename with .s extension
  #[arg(short, long, value_name = "FILE")]
  output: Option<PathBuf>,

  /// Print the generated assembly to stdout instead of writing to file
  #[arg(short = 'p', long)]
  print: bool,

  /// Verbose output
  #[arg(short, long)]
  verbose: bool,

  /// Dump lexer tokens to file
  #[arg(long, value_name = "FILE")]
  dump_tokens: Option<PathBuf>,

  /// Dump AST to file
  #[arg(long, value_name = "FILE")]
  dump_ast: Option<PathBuf>,
}

fn main() {
  let args = Args::parse();

  if args.input.extension().and_then(|s| s.to_str()) != Some("pine") {
    eprintln!("Error: Input file must have the extension .pine");
    process::exit(1);
  }

  let src = match fs::read_to_string(&args.input) {
    Ok(content) => content,
    Err(e) => {
      eprintln!("Error reading file '{}': {}", args.input.display(), e);
      process::exit(1);
    }
  };

  if args.verbose {
    println!("Compiling {}...", args.input.display());
  }

  let mut lexer = Lexer::new(&src);
  let tokens = match lexer.tokenize() {
    Ok(tokens) => tokens,
    Err(e) => {
      eprintln!("{}", e);
      process::exit(1);
    }
  };

  if args.verbose {
    println!("Lexing complete: {} tokens", tokens.len());
  }

  if let Some(token_file) = &args.dump_tokens {
    let token_output = format!("{:#?}", tokens);
    match fs::write(token_file, token_output) {
      Ok(_) => {
        if args.verbose {
          println!("Tokens dumped to '{}'", token_file.display());
        }
      }
      Err(e) => {
        eprintln!("Error writing tokens to '{}': {}", token_file.display(), e);
        process::exit(1);
      }
    }
  }

  let mut parser = Parser::new(tokens);
  let ast = match parser.parse() {
    Ok(ast) => ast,
    Err(e) => {
      eprintln!("{}", e);
      process::exit(1);
    }
  };

  if args.verbose {
    println!("Parsing complete: {} statements", ast.len());
  }

  if let Some(ast_file) = &args.dump_ast {
    let ast_output = format!("{:#?}", ast);
    match fs::write(ast_file, ast_output) {
      Ok(_) => {
        if args.verbose {
          println!("AST dumped to '{}'", ast_file.display());
        }
      }
      Err(e) => {
        eprintln!("Error writing AST to '{}': {}", ast_file.display(), e);
        process::exit(1);
      }
    }
  }

  let mut codegen = CodeGen::new();
  let asm = codegen.generate(&ast);

  if args.verbose {
    println!("Code generation complete");
  }

  if args.print {
    println!("{}", asm);
  } else {
    let output_path = args.output.unwrap_or_else(|| args.input.with_extension("s"));

    match fs::write(&output_path, asm) {
      Ok(_) => {
        if args.verbose {
          println!("Successfully compiled to '{}'", output_path.display());
        } else {
          println!("Compiled to '{}'", output_path.display());
        }
      }
      Err(e) => {
        eprintln!("Error writing to file '{}': {}", output_path.display(), e);
        process::exit(1);
      }
    }
  }
}
