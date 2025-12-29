# TUI Browser

A terminal-based web browser built with Rust, featuring ratatui for the UI, tokio for async runtime, and reqwest for HTTP requests.

## Features

### Core Features
- **Tab Management**: Multiple tabs with easy switching
- **URL Navigation**: Full URL bar with cursor support
- **Content Display**: HTML to text rendering with proper text wrapping (no cut-off text!)
- **Image Support**: Sixel/Kitty/iTerm2 inline image rendering for supported terminals
- **Keyboard-First Design**: All functionality accessible via keyboard
- **Link Navigation Mode**: Navigate and open links on the page using keyboard
- **Smart History Navigation**: Go back/forward through pages, or use Backspace for quick back navigation

### Modern Browser Features
- **History Navigation**: Go back and forward through visited pages
- **Loading Indicators**: Visual feedback when pages are loading (in content area and tab)
- **Status Bar**: Real-time status updates and help text
- **Scrollable Help Dialog**: Comprehensive keyboard shortcuts reference with scrolling support
- **Direct Link Navigation**: Type link number and press Enter to navigate, or Ctrl+Enter for new tab
- **Page Search**: Search within page content with live results (Ctrl+S)
- **Zoom Control**: Adjust content width for better readability (+/- keys)
- **Page Refresh**: Refresh current page with Ctrl+R
- **Centered Content**: Content displayed in 60% width with 20% margins for optimal reading

## User Interface

```
┌─────────────────────────────────────────────┐
│ Tabs                                        │
├─────────────────────────────────────────────┤
│ URL Bar                                     │
├─────────────────────────────────────────────┤
│                                             │
│ Content Area (Centered, 60% width)         │
│                                             │
├─────────────────────────────────────────────┤
│ Status Bar                                  │
└─────────────────────────────────────────────┘
```

## Keyboard Shortcuts

### Navigation
- `Tab` - Cycle between panels (Tab Bar → URL Bar → Content)
- `Ctrl+T` - Open new tab
- `Ctrl+W` - Close current tab
- `←/→` (in Tab Bar) - Switch between tabs
- `Ctrl+←` - Go back in history
- `Ctrl+→` - Go forward in history

### URL Bar
- `Enter` - Navigate to URL
- `Ctrl+L` - Focus URL bar
- `Ctrl+R` - Refresh current page
- `←/→` - Move cursor
- `Home/End` - Jump to start/end
- `Backspace/Delete` - Edit URL

### Search
- `Ctrl+S` - Start search in page
- `n` - Next search result
- `N` - Previous search result
- (In search mode):
  - Type to search as you type
  - `Enter` - Go to next result
  - `Esc` - Exit search

### Content
- `↑/↓` or `j/k` - Scroll line by line
- `PgUp/PgDn` - Scroll page by page
- `+/-` - Zoom in/out (adjust content width)
- `0-9` - Type link number
- `Enter` - Navigate to typed link number
- `Ctrl+Enter` - Open typed link in new tab
- `Backspace` - Clear link number or go back to previous page
- `Esc` - Clear link number
- **Link Navigation**: Links are displayed with numbers like `[1]`, `[2]`, etc. Type the number and press Enter to navigate
- **Note**: When pages are loading, a "Loading page, please wait..." message is displayed

### General
- `Ctrl+H` - Show/hide help dialog
  - In help dialog: `↑/↓` or `j/k` - Scroll through help text
  - In help dialog: `PgUp/PgDn` - Scroll page by page
- `Ctrl+Q` or `q` - Quit browser
- `Esc` - Close dialog or return to content

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo

### Build
```bash
cd tui-browser
cargo build --release
```

### Run
```bash
cargo run --release
```

## Usage

1. Launch the browser with `cargo run --release`
2. Press `Tab` to focus the URL bar (or `Ctrl+L`)
3. Type a URL (e.g., `example.com` or `https://example.com`)
4. Press `Enter` to navigate
5. Links on the page are displayed with numbers like `[1]`, `[2]`, etc.
6. Type a link number (e.g., `15`) and press `Enter` to navigate to that link
7. Type a link number and press `Ctrl+Enter` to open it in a new tab
8. Press `Ctrl+S` to search within the current page
9. Use `+/-` to zoom in/out and adjust content width
10. Press `Ctrl+H` to see all keyboard shortcuts

## Technical Details

### Dependencies
- **ratatui** (0.29) - Terminal UI framework
- **crossterm** (0.28) - Cross-platform terminal manipulation
- **tokio** (1.42) - Async runtime
- **reqwest** (0.12) - HTTP client
- **html2text** (0.12) - HTML to plain text conversion
- **ratatui-image** (1.0) - Multi-protocol terminal image rendering (sixel, kitty, iTerm2)
- **image** (0.25) - Image decoding and manipulation
- **serde** (1.0) - Serialization framework
- **chrono** (0.4) - Date and time handling
- **uuid** (1.0) - UUID generation
- **anyhow** (1.0) - Error handling

### Architecture
- **models.rs** - Core data structures (Tab, Bookmark, History)
- **http_client.rs** - HTTP fetching and HTML rendering
- **ui.rs** - UI components (TabBar, UrlBar, FavoritesBar, ContentArea, StatusBar, HelpDialog)
- **main.rs** - Application logic and event handling

### Limitations
- No JavaScript support (static content only)
- Basic HTML rendering (converted to plain text)
- No CSS styling
- Limited image rendering (first loaded image only, requires terminal with sixel/kitty/iTerm2 support)
- No form submission
- No cookie management

## Image Support

The browser now supports inline image rendering using the sixel protocol (and other protocols like Kitty and iTerm2). See [FEATURES.md](FEATURES.md) for detailed information about image support.

## Future Enhancements
- [ ] Download manager
- [ ] Cookie support
- [ ] Form handling
- [ ] Better HTML rendering
- [ ] Persistent history
- [ ] Configuration file
- [ ] Theme customization
- [x] ~~Sixel image rendering~~ (implemented!)
- [ ] Render all images inline (currently only first image)
- [ ] Image gallery view mode
- [ ] Lazy loading for images

## License

MIT
