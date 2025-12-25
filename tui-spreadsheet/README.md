# TUI Spreadsheet

A terminal-based spreadsheet application built with Rust and ratatui.

## Features

- **Excel-like Interface**: Grid layout with column headers (A, B, C...) and row numbers
- **View Mode**: Navigate between cells using arrow keys with numeric multiplier support
- **Edit Mode**: Edit cells with proper cursor navigation support (using tui-input crate)
- **Advanced Navigation**: Type numbers before arrow keys to move multiple cells at once (e.g., `123` + Down = move 123 cells)
- **Formula Engine**: Support for Excel-like formulas
  - Cell references (e.g., `=A1`, `=B2`)
  - Range references (e.g., `=SUM(A1:A100)`)
  - Built-in functions:
    - `SUM(range)` - Sum of values
    - `AVERAGE(range)` - Average of values
    - `MIN(range)` - Minimum value
    - `MAX(range)` - Maximum value
    - `COUNT(range)` - Count of values
    - `ROUND(value, decimals)` - Round to decimal places
    - `FLOOR(value)` - Round down
    - `CEIL(value)` - Round up
    - `ABS(value)` - Absolute value
    - `SQRT(value)` - Square root
    - `POW(base, exp)` - Power
    - `MOD(a, b)` - Modulo
    - `PI()` - Pi constant
  - Mathematical expressions (e.g., `=A1+B1*2`)

## Usage

### Running the Application

```bash
cargo run
```

### Key Bindings

#### View Mode
- **Arrow Keys**: Navigate between cells
- **Numeric Prefix + Arrow Keys**: Navigate multiple cells at once (e.g., type `5` then Down arrow to move 5 cells down, `10` then Right arrow to move 10 cells right)
- **e** or **Enter**: Enter edit mode for the current cell
- **=**: Start entering a formula (enters edit mode with `=` prefix)
- **Delete** or **Backspace**: Clear the current cell
- **q** or **Ctrl+C** or **Ctrl+Q**: Quit the application

#### Edit Mode
- **Type**: Enter cell value or formula
- **Arrow Keys**: Move cursor within the text (Left/Right to navigate, Home/End for start/end)
- **Enter**: Save the current value and return to view mode
- **Escape**: Cancel editing and return to view mode (discards changes)
- **Backspace**: Delete character before cursor
- **Delete**: Delete character at cursor

### Examples

1. **Basic values**: Simply type numbers or text
   ```
   123
   Hello
   ```

2. **Formulas**: Start with `=`
   ```
   =10+20
   =A1+B1
   =SUM(A1:A10)
   =AVERAGE(B1:B5)*2
   ```

3. **Cell references**: Reference other cells
   ```
   =A1
   =A1+A2+A3
   ```

## Architecture

- **main.rs**: TUI application logic, event handling, and rendering
- **formula.rs**: Formula engine with expression parsing and evaluation
  - Cell reference resolution
  - Range parsing
  - Function evaluation
  - Circular reference detection

## Building

```bash
cargo build --release
```

The compiled binary will be available at `target/release/tui-spreadsheet`.
