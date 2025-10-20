# Pine Language Compiler

A lightweight compiler for my own programming (called Pine) language that targets RISC-V assembly. Written in Rust with a focus on simplicity and educational value

## Overview
Pine is a simple imperative language designed by me to learn how compilers, lexers, and parsers work and RISC-V assembly programming. The compiler can perform lexical analysis, parsing (with type checking), and can generate RISC-V assembly code.

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- A RISC-V simulator (this project is target specifically towards) [Venus](https://venus.kvakil.me/)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd pine-rv

# Build the compiler
cargo build --release

# The binary will be at target/release/pine-rv
```

### Your First Program

Create a file `hello.pine`:

```pine
println "Hello, Pine!";
counter = 5;

while counter > 0 {
  print "Counting: ";
  println counter;
  counter = counter - 1;
}

println "Done!";
exit 0;
```

Compile and run:

```bash
# Compile to RISC-V assembly
./target/release/pine-rv hello.pine

Paste content of hello.s into [Venus](https://venus.kvakil.me/)
```

## Language Reference

### Data Types

- **Integers**: 32-bit signed integers (`42`, `-10`, `0`)
- **Strings**: Double-quoted text with escape sequences (`"Hello\n"`, `"Tab\there"`)

### Variables

Variables are dynamically typed and automatically initialized on first assignment:

```pine
x = 100;
message = "Hello";
result = x * 2 + 5;
```

### Operators

**Arithmetic:**
- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Division

**Comparison:**
- `<` Less than
- `>` Greater than
- `<=` Less than or equal
- `>=` Greater than or equal
- `==` Equal
- `!=` Not equal

**Logical:**
- `&&` Logical AND
- `||` Logical OR
- `!` Logical NOT (unary)

**Unary:**
- `-` Negation
- `!` Logical NOT

### Control Flow

**While Loops:**
```pine
x = 10;
while x > 0 {
  println x;
  x = x - 1;
}
```

### Input/Output

**Print statements:**
```pine
print 42;              # Print integer without newline
println "Text";        # Print with newline
println x + 5;         # Print expressions
println;               # Print just a newline
```

### Comments

```pine
# This is a single-line comment
x = 5;  # Comments can appear after code
```

### Program Termination

```pine
exit;      # Exit with default code
exit 0;    # Exit with specific code
exit x;    # Exit with variable value
```

## Command Line Usage

```bash
riscv-compiler [OPTIONS] <FILE>
```

### Options

- `<FILE>` - Input Pine source file (must have `.pine` extension)
- `-o, --output <FILE>` - Specify output assembly file (default: input name with `.s` extension)
- `-p, --print` - Print assembly to stdout instead of writing to file
- `-v, --verbose` - Enable verbose compilation output
- `--dump-tokens <FILE>` - Write lexer tokens to file for debugging
- `--dump-ast <FILE>` - Write AST and type information to file for debugging

### Examples

```bash
# Basic compilation
riscv-compiler program.pine

# Specify output location
riscv-compiler program.pine -o output/result.s

# View generated assembly
riscv-compiler program.pine -p

# Debug mode with full information
riscv-compiler program.pine -v --dump-tokens tokens.txt --dump-ast ast.txt
```

## Architecture

The compiler is organized into distinct phases:

```
Source Code (.pine)
    ↓
┌─────────────────┐
│     Lexer       │  → Tokenization
│   (lexer.rs)    │
└─────────────────┘
    ↓
┌─────────────────┐
│     Parser      │  → AST Construction
│   (parser.rs)   │  → Type Checking
└─────────────────┘
    ↓
┌─────────────────┐
│   Code Gen      │  → Register Allocation
│  (codegen.rs)   │  → Assembly Generation
└─────────────────┘
    ↓
RISC-V Assembly (.s)
```

### Key Features

- **Type Inference**: Automatically determines variable types from assignments
- **Type Checking**: Validates type compatibility in expressions
- **Register Allocation**: Manages RISC-V temporary registers with stack spilling
- **String Management**: Deduplicates string literals in data section
- **Error Reporting**: Provides detailed error messages with source context

### RISC-V Implementation Details

Note: These are defined in [Venus Environmental Calls](https://github.com/kvakil/venus/wiki/Environmental-Calls)
**Syscalls Used:**
- `1` - print_int
- `4` - print_string  
- `10` - exit
- `11` - print_character
- `17` - exit2 (exit with code)

**Register Usage:**
- `t0-t6` - Temporary registers for expression evaluation
- `sp` - Stack pointer (512-byte frame)
- `a0-a1` - Syscall arguments

## Project Structure

```
src/
├── main.rs       # CLI interface and compilation orchestration
├── lib.rs        # Module declarations
├── lexer.rs      # Tokenization and lexical analysis
├── parser.rs     # Pratt parser with type checking
├── codegen.rs    # RISC-V assembly code generation
└── error.rs      # Error types and pretty-printing

rustfmt.toml      # Rust formatting specs
Cargo.toml        # Project dependencies
README.md         # This file
```

## TODO

### High Priority

- [ ] **If/Else conditionals**
- [ ] **Functions and function calls**
- [ ] **Break and continue statements**
- [ ] **Proper variable scoping**
- [ ] **For loops**

### Medium Priority

- [ ] **Arrays and indexing**
- [ ] **String concatenation**
- [ ] **Boolean type**
- [ ] **Modulo operator (%)**
- [ ] **Multi-line comments**
- [ ] **Optimization passes** - Constant folding, dead code elimination

### Nice to Have

- [ ] **Inline assembly**
- [ ] **Floating-point arithmetic**
- [ ] **Bitwise operators**
- [ ] **Hexadecimal and binary literals** - `0xFF`, `0b1010`
- [ ] **Multiple source files and imports** - Modular programs
- [ ] **Standard library** - Common utility functions
- [ ] **Debug symbol generation** - Better debugging support
- [ ] **REPL mode** - Interactive development environment

## License

This project is licensed under the GNU General Public License v3.0 (GPLv3)

---

**Note**: This is an educational compiler made mainly for myself. It prioritizes clarity and simplicity over performance and may not implement all production-compiler features
