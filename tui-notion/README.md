# TUI Notion

A terminal-based Notion clone built with Rust and ratatui. Create, edit, and organize markdown documents in a beautiful terminal UI.

## Features

- **Three-Panel Layout**:
  - Left: Document tree for navigation
  - Center: Markdown editor with syntax highlighting
  - Right: Live table of contents (outline)

- **Keyboard-First Navigation**:
  - Vi-style keybindings (j/k) and arrow keys
  - Tab to cycle between panels
  - No mouse support - pure keyboard navigation

- **Document Management**:
  - Create new documents (Ctrl+N)
  - Edit documents (press 'i' in editor)
  - Delete documents (Ctrl+D)
  - Auto-save on mode change (Ctrl+S)

- **Quick Search**:
  - Ctrl+K to open search dialog
  - Live search across all documents
  - Jump to any document instantly

- **Live Table of Contents**:
  - Automatically extracts headings from markdown
  - Live synchronization as you type
  - Press Enter in TOC to jump to heading

- **Markdown Support**:
  - Syntax highlighting for:
    - Headings (# ## ###)
    - Code blocks (\`\`\`)
    - Lists (- *)
  - Edit in plain markdown

## Installation

```bash
cd tui-notion
cargo build --release
```

## Usage

Run the application:

```bash
cargo run
```

Or run the built binary:

```bash
./target/release/tui-notion
```

## Keybindings

### Global

- `q` - Quit application
- `Tab` - Cycle between panels (Tree → Editor → TOC)
- `Ctrl+K` - Open search dialog
- `Ctrl+N` - Create new document
- `Ctrl+S` - Save current document
- `Ctrl+D` - Delete current document

### Normal Mode (Editor)

- `i` - Enter insert mode
- `j` / `↓` - Scroll down
- `k` / `↑` - Scroll up
- `PageDown` - Scroll down one page
- `PageUp` - Scroll up one page

### Insert Mode (Editor)

- `Esc` - Exit insert mode (auto-saves)
- Arrow keys - Move cursor
- `Home` - Move to line start
- `End` - Move to line end
- `Enter` - New line
- `Backspace` - Delete character

### Document Tree

- `j` / `↓` - Next document
- `k` / `↑` - Previous document
- `Enter` - Open selected document

### Table of Contents

- `j` / `↓` - Next heading
- `k` / `↑` - Previous heading
- `Enter` - Jump to heading in editor

### Search Dialog

- Type to search
- `↓` - Next result
- `↑` - Previous result
- `Enter` - Open selected document
- `Esc` - Close search dialog

## Data Storage

Documents are stored as JSON files in `~/.tui-notion/`.

Each document has:
- Unique ID (UUID)
- Title
- Content (markdown)
- Parent ID (for future hierarchical organization)

## Example Workflow

1. Start the application: `cargo run`
2. Create a new document: `Ctrl+N`
3. Enter insert mode: `i`
4. Type your markdown content with headings
5. Exit insert mode: `Esc` (auto-saves)
6. Navigate to TOC: `Tab` twice
7. Jump to a heading: `Enter`
8. Search for documents: `Ctrl+K`
9. Type to filter, select with arrows, open with `Enter`

## Markdown Syntax Highlighting

The editor highlights:

- **Level 1 headings** (`#`) - Light Blue, Bold
- **Level 2 headings** (`##`) - Light Cyan, Bold
- **Level 3 headings** (`###`) - Light Green, Bold
- **Other headings** (`####` etc.) - Green, Bold
- **Code blocks** (\`\`\`) - Yellow
- **Lists** (`-` or `*`) - Cyan

## Architecture

- `main.rs` - Application entry point and event handling
- `document.rs` - Document data model
- `tree.rs` - Document tree management
- `editor.rs` - Text editor logic
- `toc.rs` - Table of contents extraction
- `search.rs` - Search functionality
- `storage.rs` - File persistence
- `ui.rs` - UI rendering with ratatui

## Future Enhancements

- Hierarchical document tree (nested documents)
- Copy/paste support
- Undo/redo
- More markdown syntax highlighting (bold, italic, links)
- Export to HTML/PDF
- Tags and filtering
- Dark/light theme support
- Custom keybindings

## License

MIT
