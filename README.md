# Pine Language Compiler

A lightweight compiler for my own programming language (called Pine) that targets RISC-V assembly. Written in Rust with a focus on simplicity and educational value (or at least some value).

## Overview
Pine is a simple imperative language designed by me to learn how compilers, lexers, and parsers work and RISC-V assembly programming. The compiler can perform lexical analysis, parsing (with type checking), and can generate RISC-V assembly code.

## Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- RARS (RISC-V Assembler and Runtime Simulator)

### Installing RARS
The releases and installation process for installing RARS can be found here:
https://github.com/TheThirdOne/rars

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

# Run with RARS
rars hello.s
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

**Bitwise:**
- `&` Bitwise AND
- `|` Bitwise OR
- `^` Bitwise XOR
- `~` Bitwise NOT (unary)
- `<<` Left shift
- `>>` Right shift (arithmetic)

**Unary:**
- `-` Negation
- `!` Logical NOT
- `~` Bitwise NOT

### Control Flow

**If/Else statements:**
```pine
x = 10;
if x > 5 {
  println "x is greater than 5";
} else {
  println "x is 5 or less";
}
```

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

**Input:**
```pine
x = input();           # Read integer from user
println x;
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
pine-rv [OPTIONS] <FILE>
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
pine-rv program.pine

# Specify output location
pine-rv program.pine -o output/result.s

# View generated assembly
pine-rv program.pine -p

# Debug mode with full information
pine-rv program.pine -v --dump-tokens tokens.txt --dump-ast ast.txt

# Compile and run
pine-rv program.pine && rars program.s
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

**Syscalls Used:**
All syscalls that rars supports are shown [Here](https://github.com/TheThirdOne/rars/wiki/Environment-Calls)
- `1` - PrintInt
- `4` - PrintString
- `5` - ReadInt
- `10` - Exit
- `11` - PrintChar

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

## License

This project is licensed under the GNU General Public License v3.0 (GPLv3)

---

**Note**: This is a compiler made mainly for myself. This is not by any means the "proper" way of going able making something resembling a compiler. It's just a fun side-project.
