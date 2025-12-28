# TUI RSS Reader

A powerful terminal-based RSS reader built with Rust and ratatui, inspired by Feedly.

## Features

- **Two-Panel Interface**: Article list on the left, reader on the right
- **Focus Mode**: Press `f` to hide the sidebar and focus on reading
- **Feed Management**: Add, delete, and manage RSS feeds
- **Search**: Search through articles with `/`
- **Keyboard Navigation**: Vim-style keybindings (j/k) and arrow keys
- **Read/Unread Tracking**: Keep track of which articles you've read
- **Responsive Scrolling**: Smooth scrolling with Page Up/Down support
- **Centered Reading**: Content is centered for better readability
- **Persistent Storage**: Feeds and read status saved locally

## Installation

```bash
cd tui-rss
cargo build --release
```

## Usage

Run the application:

```bash
cargo run --release
```

Or run the built binary:

```bash
./target/release/tui-rss
```

## Keyboard Shortcuts

### Navigation
- `↑/↓` or `j/k` - Navigate articles
- `Enter` - Select article / Open reader
- `Tab` - Switch between panels
- `Esc` - Return to article list

### Article Reader
- `↑/↓` or `j/k` - Scroll content
- `PageUp/PageDown` - Scroll by page
- `n` - Next article
- `p` - Previous article
- `t` - Toggle read/unread status

### Features
- `n` - Add new RSS feed
- `r` - Refresh all feeds
- `f` - Toggle focus mode (hide sidebar)
- `/` - Search articles
- `m` - Manage feeds (view, delete, refresh individual feeds)
- `u` - Toggle show unread only
- `?` - Show help
- `q` - Quit

## Feed Management

Press `m` to enter feed management mode:
- `↑/↓` or `j/k` - Navigate feeds
- `d` - Delete selected feed
- `r` - Refresh selected feed
- `Esc` - Return to normal mode

## Adding Feeds

1. Press `n` to open the "Add Feed" dialog
2. Enter the RSS feed URL (e.g., `https://example.com/feed.xml`)
3. Press `Enter` to add the feed
4. The feed will be fetched and articles will be added

## Storage

Feeds and read status are stored in:
- Linux/Mac: `~/.config/tui-rss/feeds.json`
- Windows: `%USERPROFILE%\.config\tui-rss\feeds.json`

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `feed-rs` - RSS/Atom feed parsing
- `reqwest` - HTTP client for fetching feeds
- `tui-input` - Text input handling with cursor support
- `html2text` - Convert HTML content to plain text
- `chrono` - Date/time handling
- `serde` & `serde_json` - Data serialization

## Tips

- Use focus mode (`f`) for distraction-free reading
- Search (`/`) works on both article titles and descriptions
- Toggle unread filter (`u`) to focus on new content
- Refresh regularly (`r`) to get the latest articles
- The reader centers content for optimal reading experience

## Example Feeds

Here are some popular RSS feeds to get started:
- Hacker News: `https://news.ycombinator.com/rss`
- Reddit Programming: `https://www.reddit.com/r/programming/.rss`
- Lobsters: `https://lobste.rs/rss`

## License

MIT
