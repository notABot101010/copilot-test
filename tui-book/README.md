# TUI Book Reader

A terminal-based EPUB book reader built with Rust and ratatui.

## Features

- Read EPUB books in your terminal
- Clean, centered text display
- Interactive table of contents sidebar
- Keyboard navigation
- Focus management between panels

## Installation

```bash
cargo build --release
```

## Usage

```bash
tui-book <path-to-book.epub>
```

Example:
```bash
tui-book alice.epub
```

## Keyboard Controls

- **Ctrl+B**: Toggle the table of contents panel
- **Tab**: Switch focus between the TOC panel and content panel
- **Arrow Up/Down**: Navigate within the focused panel
  - In TOC: Select different sections
  - In Content: Scroll through the book
- **Enter**: When in TOC, jump to the selected section
- **q**: Quit the application

## Features in Detail

### Table of Contents Panel
- Appears on the left side of the screen
- Shows the book's table of contents
- Navigate with arrow keys
- Press Enter to jump to a section
- Toggle visibility with Ctrl+B

### Content Panel
- Displays the book content with centered text
- Scrollable with arrow keys
- Clean, readable layout
- Shows current section content

### Focus Management
- The focused panel is highlighted with cyan borders
- Unfocused panels have gray borders
- Use Tab to switch focus between panels
- When TOC is hidden, focus automatically goes to content panel

## Technical Details

Built with:
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI library
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [epub](https://github.com/danigm/epub-rs) - EPUB parsing

## License

MIT
