# TUI Calculator

A terminal-based calculator written in Rust that supports arbitrary precision arithmetic for both integers and floating-point numbers.

## Features

- **REPL Interface**: Interactive command-line interface with a `> ` prompt
- **History Navigation**: Use arrow keys (↑/↓) to navigate through expression history
- **Arbitrary Precision Integers**: Handles arbitrarily large integers using BigInt
- **Arbitrary Precision Floats**: Handles high-precision floating-point numbers using BigFloat
- **Smart Formatting**: Displays integers without decimal points (e.g., `1 + 2 = 3`) and floats with decimals (e.g., `1.5 + 1.5 = 3.0`)
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

> 1.5 + 1.5
3.0

> 5 / 2
2.5

> 10.5 - 5.5
5.0

> 999999999999999999 + 1
1000000000000000000

> 10 ^ 50
100000000000000000000000000000000000000000000000000

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
- **Lexer**: Tokenizes the input string into tokens (supports both integers and decimal numbers)
- **Recursive Descent Parser**: Parses tokens respecting operator precedence
- **Hybrid Evaluation**: Uses BigInt for integer operations and BigFloat for floating-point operations, maintaining arbitrary precision for both
