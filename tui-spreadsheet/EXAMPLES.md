# TUI Spreadsheet - Usage Examples

This file demonstrates various usage scenarios for the TUI spreadsheet application.

## Example 1: Basic Values

1. Start the application: `cargo run --release`
2. Navigate to cell A1 (should be selected by default)
3. Press `e` to enter edit mode
4. Type `100`
5. Press Enter to save
6. Press Down arrow to move to A2
7. Type `200` (typing automatically enters edit mode)
8. Press Enter

Result: Cells A1 and A2 now contain 100 and 200 respectively.

## Example 2: Simple Formula

1. Navigate to cell A3
2. Press `=` to start a formula
3. Type `A1+A2`
4. Press Enter

Result: Cell A3 displays `300` (the sum of A1 and A2)

## Example 3: Using the SUM Function

1. Enter values in cells B1 through B5:
   - B1: 10
   - B2: 20
   - B3: 30
   - B4: 40
   - B5: 50

2. Navigate to cell B6
3. Press `=` to start a formula
4. Type `SUM(B1:B5)`
5. Press Enter

Result: Cell B6 displays `150` (the sum of all values in B1:B5)

## Example 4: Complex Formula with Multiple Functions

1. Enter values in cells:
   - C1: 10
   - C2: 20
   - C3: 30

2. Navigate to cell C4
3. Enter formula: `=AVERAGE(C1:C3)*2`
4. Press Enter

Result: Cell C4 displays `40` (average of 20 * 2)

## Example 5: Nested Cell References

1. Set up the following:
   - D1: 5
   - D2: `=D1*2` (displays 10)
   - D3: `=D2+D1` (displays 15)
   - D4: `=SUM(D1:D3)` (displays 30)

This demonstrates how cells can reference other cells that contain formulas.

## Example 6: Mathematical Expressions

Try these formulas to see the expression evaluator in action:
- `=(10+20)*3-5` → 85
- `=10/2+3*4` → 17
- `=(100-50)/5` → 10

## Example 7: Using MIN and MAX

1. Enter values in cells:
   - E1: 45
   - E2: 12
   - E3: 89
   - E4: 23
   - E5: 67

2. In E6, enter: `=MIN(E1:E5)` → displays 12
3. In E7, enter: `=MAX(E1:E5)` → displays 89

## Example 8: COUNT Function

With the same E1:E5 range:
- In E8, enter: `=COUNT(E1:E5)` → displays 5

## Example 9: Editing Existing Cells

1. Navigate to any cell with content
2. Press `e` to edit
3. The current value appears in the formula bar
4. Modify the value
5. Press Enter to save or Escape to cancel

## Example 10: Clearing Cells

1. Navigate to a cell with content
2. Press Delete or Backspace
3. The cell is immediately cleared

## Tips

- Formulas are displayed in green when in view mode
- The current cell is highlighted in blue
- The formula bar shows the raw formula, while the grid shows computed values
- Use arrow keys to quickly navigate between cells
- Press 'q' or Ctrl+C to exit the application

## Common Functions Reference

| Function | Usage | Example | Result |
|----------|-------|---------|--------|
| SUM | `=SUM(A1:A10)` | Sum of range | Total of all values |
| AVERAGE | `=AVERAGE(A1:A10)` | Average of range | Mean value |
| MIN | `=MIN(A1:A10)` | Minimum in range | Smallest value |
| MAX | `=MAX(A1:A10)` | Maximum in range | Largest value |
| COUNT | `=COUNT(A1:A10)` | Count values in range | Number of cells |
| ROUND | `=ROUND(3.14159, 2)` | Round to decimals | 3.14 |
| SQRT | `=SQRT(16)` | Square root | 4 |
| POW | `=POW(2, 3)` | Power | 8 |
| ABS | `=ABS(-5)` | Absolute value | 5 |
| PI | `=PI()` | Pi constant | 3.14159... |

## Error Handling

The spreadsheet displays `#ERROR` for:
- Division by zero
- Invalid formulas
- Circular references
- Function evaluation errors

Example of circular reference (will show #ERROR):
- A1: `=B1+1`
- B1: `=A1+1`

Both cells will display `#ERROR` due to circular dependency.
