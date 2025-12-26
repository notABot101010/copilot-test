# TUI Notion - Visual Guide

## Application Layout

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                       TUI NOTION                                                 │
├──────────────────┬──────────────────────────────────────────────────┬─────────────────────────┤
│ Documents        │ Editor [NORMAL]                                   │ Outline                 │
├──────────────────┤                                                   ├─────────────────────────┤
│ 📄 Welcome to... │ # Welcome to TUI Notion                           │ Welcome to TUI Notion   │
│                  │                                                   │ Features                │
│                  │ A terminal-based Notion clone built with Rust!   │ Quick Start             │
│                  │                                                   │   Keyboard Shortcuts    │
│                  │ ## Features                                       │   Navigation            │
│                  │                                                   │ Try It Out              │
│                  │ - **Three-Panel Layout**: Navigate docs, edit... │   Markdown Syntax       │
│                  │ - **Markdown Support**: Full markdown editing... │   Document Management   │
│                  │ - **Live Table of Contents**: Auto-generated...  │                         │
│                  │ - **Keyboard Navigation**: Vi-style keys...      │                         │
│                  │                                                   │                         │
│                  │ ## Quick Start                                    │                         │
│                  │                                                   │                         │
│                  │ 1. Press `i` to enter INSERT mode                │                         │
│                  │ 2. Type your markdown content                    │                         │
│                  │ 3. Press `Esc` to save                           │                         │
│                  │ 4. Use `Tab` to cycle between panels             │                         │
│                  │                                                   │                         │
│                  │ ### Keyboard Shortcuts                            │                         │
│                  │                                                   │                         │
│                  │ - **Ctrl+K**: Quick search across documents      │                         │
│                  │ - **Ctrl+N**: Create new document                │                         │
│                  │ - **Ctrl+S**: Save current document              │                         │
│                  │ ...                                               │                         │
│                  │                                            Ln 12, Col 1                     │
└──────────────────┴──────────────────────────────────────────────────┴─────────────────────────┘
```

## Search Dialog (Ctrl+K)

```
                       ┌────────────────────────────────────────────┐
                       │ Search Documents (Ctrl+K)                  │
                       ├────────────────────────────────────────────┤
                       │ ┌Query──────────────────────────────────┐ │
                       │ │ welcome                                │ │
                       │ └───────────────────────────────────────┘ │
                       │ ┌Results (2)────────────────────────────┐ │
                       │ │ 📄 Welcome to TUI Notion               │ │
                       │ │ 📄 Welcome Guide                       │ │
                       │ └───────────────────────────────────────┘ │
                       └────────────────────────────────────────────┘
```

## Key Features

### 1. Document Tree (Left Panel)
- Shows all documents in a list format
- Selected document is highlighted in cyan
- Navigate with j/k or arrow keys
- Press Enter to open document

### 2. Markdown Editor (Center Panel)
- Syntax highlighting:
  - `# Heading 1` - Light Blue, Bold
  - `## Heading 2` - Light Cyan, Bold
  - `### Heading 3` - Light Green, Bold
  - ``` Code blocks ``` - Yellow
  - `- List items` - Cyan
- Two modes: NORMAL and INSERT
- Cursor position shown at bottom right

### 3. Table of Contents (Right Panel)
- Auto-generated from markdown headings
- Live updates as you type
- Indented based on heading level
- Jump to heading with Enter

### 4. Quick Search
- Ctrl+K to open
- Live filtering as you type
- Shows document titles
- Navigate results with arrows

## Color Scheme

- **Cyan**: Focused panel border, highlighted selections
- **Light Blue**: H1 headings
- **Light Cyan**: H2 headings
- **Light Green**: H3 headings
- **Yellow**: Code blocks
- **Dark Gray**: Cursor line, hints
- **White**: Normal text

## Workflow Example

1. **Start**: `cargo run`
2. **Create Document**: `Ctrl+N`
3. **Enter Insert Mode**: `i`
4. **Type Content**:
   ```markdown
   # My Project
   
   ## Todo
   - Task 1
   - Task 2
   
   ## Notes
   Some important notes here.
   ```
5. **Exit Insert Mode**: `Esc` (auto-saves)
6. **View TOC**: `Tab` twice, see headings listed
7. **Jump to Section**: Select "Notes", press `Enter`
8. **Search**: `Ctrl+K`, type "project", `Enter`
9. **Quit**: `q`

## Technical Details

- **Language**: Rust
- **TUI Framework**: ratatui 0.29
- **Terminal**: crossterm 0.28
- **Data Format**: JSON
- **Storage**: `~/.tui-notion/*.json`
- **Total Lines**: ~1255 lines of Rust code
- **Modules**: 8 (main, document, tree, editor, toc, search, storage, ui)
