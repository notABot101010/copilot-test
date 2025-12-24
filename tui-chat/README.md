# TUI Chat

A terminal-based chat application built with Ratatui, inspired by Telegram's interface.

## Features

- **Conversation List**: Browse through your conversations on the left panel
- **Message View**: Read messages from the selected conversation on the right panel
- **Multi-line Input**: Compose and send messages with support for multi-line text
- **Keyboard Navigation**: Navigate efficiently using keyboard shortcuts

## Keyboard Shortcuts

### Navigation Mode (Default)
- `↑` or `k`: Move up in conversation list
- `↓` or `j`: Move down in conversation list
- `Enter`: Select conversation and start typing
- `Esc`: Deselect current conversation
- `q` or `Q`: Quit the application

### Editing Mode (When typing)
- `Enter`: Send message
- `Shift+Enter`: Add new line
- `Backspace`: Delete character
- `Esc`: Cancel and return to navigation mode

## Building and Running

```bash
# Build the project
cargo build --release

# Run the application
cargo run --release
```

## Mock Data

The application uses mock conversations and messages for demonstration purposes. No server connection is required.

## Dependencies

- `ratatui`: Terminal UI framework
- `crossterm`: Terminal manipulation library
- `chrono`: Date and time handling
- `uuid`: Unique identifiers for messages
- `serde`: Serialization framework
- `anyhow`: Error handling
