# TUI Calculator

A terminal-based calculator written in Rust that supports arbitrary large numbers and expression evaluation.

## Features

- **REPL Interface**: Interactive command-line interface with a `> ` prompt
- **History Navigation**: Use arrow keys (↑/↓) to navigate through expression history
- **Arbitrary Large Numbers**: Handles arbitrarily large integers using BigInt
- **Comprehensive Operations**:
  - Addition (`+`)
  - Subtraction (`-`)
  - Multiplication (`*`)
  - Division (`/`)
  - Modulo (`%`)
  - Exponentiation (`^`)
  - Parentheses for grouping `(` `)`
- **Error Handling**:
  - Division by zero detection
  - Unbalanced parentheses detection
  - Invalid expression detection

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Usage Examples

```
> 2 + 3
5

> 10 * 5
50

> 2 ^ 10
1024

> (2 + 3) * 4
20

> 999999999999999999 + 1
1000000000000000000

> 5 / 0
Error: Division by zero

> (2 + 3
Error: Unbalanced parentheses
```

## Operator Precedence

The calculator follows standard mathematical operator precedence:

1. Parentheses `( )`
2. Exponentiation `^` (right associative)
3. Multiplication `*`, Division `/`, Modulo `%`
4. Addition `+`, Subtraction `-`

## History

The calculator maintains an in-memory history of expressions. Use:
- **Arrow Up** (↑): Navigate to previous expressions
- **Arrow Down** (↓): Navigate to next expressions

You can edit any expression retrieved from history before pressing Enter to evaluate it.

## Exiting

To exit the calculator, use:
- **Ctrl+C**: Interrupt
- **Ctrl+D**: EOF

## Architecture

The project is divided into two main files:

- `main.rs`: Contains the REPL loop and handles user interaction
- `calc.rs`: Contains the expression evaluator with lexer and parser

The expression evaluator uses:
- **Lexer**: Tokenizes the input string into tokens
- **Recursive Descent Parser**: Parses tokens respecting operator precedence
- **BigInt Evaluation**: Evaluates the parsed expression using arbitrary precision integers
