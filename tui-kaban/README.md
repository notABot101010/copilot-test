# TUI Kanban

A terminal-based kanban board application built with Rust and ratatui.

## Features

- **Create Cards**: Add new cards with title and description
- **Edit Cards**: Modify existing card information
- **Delete Cards**: Remove cards you no longer need
- **Move Cards**: Drag and drop cards between columns using mouse
- **Keyboard Navigation**: Full keyboard support for efficient workflow
- **Three Columns**: To Do, In Progress, and Done

## Installation

```bash
cargo build --release
```

## Usage

Run the application:

```bash
cargo run --bin tui-kaban
```

### Keyboard Shortcuts

#### Normal Mode
- `q` or `Q` - Quit the application
- `n` or `N` - Create a new card in the selected column
- `e` or `E` - Edit the selected card
- `d` or `D` - Delete the selected card
- `h` or `←` - Move to the previous column
- `l` or `→` - Move to the next column
- `k` or `↑` - Select the previous card in the current column
- `j` or `↓` - Select the next card in the current column
- `H` (Shift+h) - Move the selected card to the previous column
- `L` (Shift+l) - Move the selected card to the next column

#### Card Creation/Editing Mode
- `Tab` - Switch between title and description fields
- `Enter` - Save the card
- `Esc` - Cancel and return to normal mode
- Type to enter text in the active field
- `Backspace` - Delete characters

### Mouse Support

- **Click** on a card to select it
- **Click and drag** a card to another column to move it
- **Click** on a column to select that column

## Project Structure

- `src/main.rs` - Main application code with UI rendering and event handling
- `Cargo.toml` - Project dependencies and configuration

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation library
- `serde` - Serialization framework
- `uuid` - UUID generation for card IDs
- `anyhow` - Error handling

## License

MIT
