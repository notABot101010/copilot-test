# Changes - Undo/Redo and CSV Support

## New Features

### 1. Undo/Redo System
- **Ctrl+Z**: Undo the last change
- **Ctrl+Y**: Redo the last undone change
- Tracks complete cell state history
- Redo stack is automatically cleared when new changes are made

### 2. CSV File Support
- **Command line usage**: `tui-spreadsheet file.csv`
- **Ctrl+S**: Save current spreadsheet to CSV file (in view mode)
- Automatically loads CSV file on startup
- Creates empty file if it doesn't exist
- Preserves formulas (e.g., `=SUM(A1:A10)`)
- Handles sparse data correctly (rows with empty cells)

## Technical Changes

### Modified Files
- `Cargo.toml`: Added csv crate dependency (v1.3)
- `src/main.rs`: 
  - Added undo/redo stack management
  - Added CSV read/write functions
  - Updated keyboard handlers
  - Enhanced UI to show filename and new shortcuts
  - Added 4 new comprehensive tests

### Implementation Details
- Uses flexible CSV format to handle variable-length rows
- Optimized CSV output (only writes rows and columns with data)
- Cross-platform temp directory usage in tests
- Proper error handling for file I/O

## Testing
- All 26 tests pass (added 4 new tests)
- Test coverage:
  - Undo/redo workflow
  - Redo stack clearing
  - CSV save and load
  - Sparse row handling (empty cells in middle)

## Usage Examples

### Create a new spreadsheet
```bash
tui-spreadsheet budget.csv
```

### Edit cells and formulas
1. Navigate with arrow keys
2. Press 'e' or '=' to edit
3. Press Ctrl+Z to undo
4. Press Ctrl+Y to redo
5. Press Ctrl+S to save
6. Press 'q' to quit

### Work with existing CSV
```bash
tui-spreadsheet existing_data.csv
# Make changes...
# Press Ctrl+S to save
```

## Compatibility
- Works on Linux, macOS, and Windows
- CSV files are portable between systems
- Uses standard CSV format with flexible row lengths
