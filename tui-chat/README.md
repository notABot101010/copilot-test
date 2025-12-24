# TUI Chat

A terminal-based chat application built with Ratatui, inspired by Telegram's interface.

## Features

- **Conversation List**: Browse through your conversations on the left panel
- **Message View**: Read messages from the selected conversation on the right panel
- **Multi-line Input**: Compose and send messages with support for multi-line text
- **Keyboard Navigation**: Navigate efficiently using keyboard shortcuts
- **Mouse Support**: Click to select conversations, focus input, and scroll messages

## Keyboard Shortcuts

### Navigation Mode (Default)
- `↑` or `k`: Move up in conversation list
- `↓` or `j`: Move down in conversation list
- `Enter`: Select conversation and start typing
- `Esc`: Deselect current conversation
- `q` or `Q`: Quit the application
- `Page Up`: Scroll messages up
- `Page Down`: Scroll messages down

### Editing Mode (When typing)
- `Enter`: Send message
- `Shift+Enter`: Add new line (Note: may not work in all terminals)
- `Backspace`: Delete character
- `Esc`: Cancel and return to navigation mode

## Mouse Support

The application supports mouse interactions for a more intuitive experience:

- **Click on a conversation**: Selects the conversation and marks it as read
- **Click on the input box**: Focuses the input box to start typing (requires a conversation to be selected)
- **Scroll in the message view**: Use mouse wheel to scroll through messages
  - Scroll up: View older messages
  - Scroll down: View newer messages

## Terminal Compatibility

The application has been tested with modern terminal emulators. Note that `Shift+Enter` for multi-line input may not work in all terminals due to terminal-specific key binding limitations. If `Shift+Enter` doesn't work in your terminal, you can compose multi-line messages by editing external text.

For the best mouse support experience, use a modern terminal emulator that fully supports mouse events (e.g., iTerm2, Windows Terminal, Alacritty, or Kitty).

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
