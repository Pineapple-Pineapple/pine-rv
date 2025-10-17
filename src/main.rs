use std::env;
use std::fs;
use std::process;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() != 2 {
    eprintln!("Usage: {} <input.pine>", args[0]);
    eprintln!("Example: {} <program.pine>", args[0]);
    process::exit(1);
  }

  let input_path = &args[1];

  if !input_path.ends_with(".pine") {
    eprintln!("Error: Input file must have the extension .pine");
  }

  let src = match fs::read_to_string(input_path) {
    | Ok(content) => content,
    | Err(e) => {
      eprintln!("Error reading file '{}': {}", input_path, e);
      process::exit(1);
    }
  };

  println!("Compiling {}", input_path);
  let asm = "TODO";

  let output_path = input_path.strip_suffix(".pine").unwrap().to_string() + ".s";

  match fs::write(&output_path, asm) {
    | Ok(_) => println!("Successfully compiled to '{}'", output_path),
    | Err(e) => {
      eprintln!("Error writing to file '{}': {}", output_path, e);
      process::exit(1);
    }
  }
}
