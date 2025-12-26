# TUI Notion

A terminal-based Notion clone built with Rust and ratatui. Create, edit, and organize markdown documents in a beautiful terminal UI.

## Features

- **Two-Panel Layout**:
  - Left: Live table of contents (outline) for quick navigation
  - Right: Markdown editor with syntax highlighting and visible cursor in insert mode

- **Keyboard-First Navigation**:
  - Vi-style keybindings (j/k) and arrow keys
  - Tab to cycle between panels
  - No mouse support - pure keyboard navigation

- **Document Management**:
  - Create new documents (Ctrl+N)
  - Edit documents (press 'i' in editor)
  - Delete documents (Ctrl+D)
  - Auto-save on mode change (Ctrl+S)
  - Navigate between documents with quick search (Ctrl+K)

- **Quick Search**:
  - Ctrl+K to open search dialog
  - Shows all documents when search is empty
  - Live search across all documents with real input cursor
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
  - Visible cursor in insert mode
  - Content scrolls with cursor automatically

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
- `Tab` - Cycle between panels (Outline ↔ Editor)
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
- Arrow keys - Move cursor (content scrolls automatically)
- `Home` - Move to line start
- `End` - Move to line end
- `Enter` - New line
- `Backspace` - Delete character

### Table of Contents

- `j` / `↓` - Next heading
- `k` / `↑` - Previous heading
- `Enter` - Jump to heading in editor

### Search Dialog

- Type to search documents
- Empty query shows all documents
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
2. Press `Ctrl+K` to open search (shows all documents)
3. Select a document or create a new one with `Ctrl+N`
4. Enter insert mode: `i`
5. Type your markdown content with headings
6. Exit insert mode: `Esc` (auto-saves)
7. Navigate to Outline: `Tab`
8. Jump to a heading: `Enter`
9. Search for documents: `Ctrl+K`
10. Type to filter, select with arrows, open with `Enter`

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
