# TUI Browser - Visual Layout

## Main Interface

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Tabs (Ctrl+T: New | ←/→: Switch)                                           │
│  1 New Tab   2 Example ⟳   3 Rust-lang                                    │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ URL Bar (Enter: Navigate)                                                   │
│ https://example.com█                                                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ Favorites (Ctrl+F: Add | ←/→: Navigate | Enter: Open)                      │
│  ★ Example  ★ Rust Lang  ★ GitHub                                          │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ Content (↑/↓: Scroll | PgUp/PgDn: Page)                                    │
│                                                                              │
│  Example Domain                                                              │
│  ===============                                                             │
│                                                                              │
│  This domain is for use in illustrative examples in documents. You may      │
│  use this domain in literature without prior coordination or asking for     │
│  permission.                                                                 │
│                                                                              │
│  More information...                                                         │
│                                                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ Loaded: Example Domain              Ctrl+H: Help | Ctrl+Q: Quit            │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Help Dialog (Ctrl+H)

```
              ┌───────────────────────────────────────────────┐
              │ Keyboard Shortcuts (Esc: Close)               │
              ├───────────────────────────────────────────────┤
              │ Navigation:                                   │
              │   Tab          - Cycle between panels         │
              │   Ctrl+T       - Open new tab                 │
              │   Ctrl+W       - Close current tab            │
              │   ←/→ (tabs)   - Switch between tabs         │
              │                                               │
              │ URL Bar:                                      │
              │   Enter        - Navigate to URL              │
              │   Ctrl+L       - Focus URL bar                │
              │                                               │
              │ Favorites:                                    │
              │   Ctrl+F       - Add current page             │
              │   ←/→ (favs)   - Navigate favorites          │
              │   Enter        - Open selected favorite       │
              │                                               │
              │ Content:                                      │
              │   ↑/↓ or j/k   - Scroll line by line         │
              │   PgUp/PgDn    - Scroll page by page         │
              │   Ctrl+S       - Search in page (TODO)       │
              │                                               │
              │ General:                                      │
              │   Ctrl+H       - Show this help               │
              │   Ctrl+Q or q  - Quit browser                 │
              └───────────────────────────────────────────────┘
```

## Panel Focus Visualization

When a panel is focused, it has a **yellow border** instead of white.

### Example: URL Bar Focused
```
┌─────────────────────────────────────────────────────────────────────────────┐ (white)
│ Tabs (Ctrl+T: New | ←/→: Switch)                                           │
│  1 New Tab   2 Example   3 Rust-lang                                       │
└─────────────────────────────────────────────────────────────────────────────┘

╔═════════════════════════════════════════════════════════════════════════════╗ (yellow - focused)
║ URL Bar (Enter: Navigate)                                                   ║
║ https://example.com█                                                        ║
╚═════════════════════════════════════════════════════════════════════════════╝
```

## Features Demonstrated

### 1. Tab Management
- Multiple tabs open simultaneously
- Active tab highlighted in cyan
- Loading indicator (⟳) shown for pages being fetched
- Tab numbers for quick identification

### 2. URL Bar
- Full cursor support (█ shows cursor position)
- Auto-complete with https:// prefix
- Keyboard navigation (Home, End, ←, →)
- Character editing (Backspace, Delete)

### 3. Favorites/Bookmarks
- Star (★) icon for each bookmark
- Selected bookmark highlighted
- Horizontal scrolling for many bookmarks
- Quick access with arrow keys

### 4. Content Display
- HTML converted to readable text
- Scroll position maintained
- Line-by-line and page scrolling
- Clean formatting

### 5. Status Bar
- Left: Current page status/messages
- Right: Quick help reminder
- Color-coded (cyan for status, gray for help)

### 6. History Navigation
- Ctrl+← to go back
- Ctrl+→ to go forward
- History preserved per tab
- Status messages confirm navigation

## Color Scheme

- **Cyan**: Active elements, highlights, status text
- **Yellow**: Focused panel borders, section headers
- **White**: Normal text, inactive borders
- **Gray**: Placeholder text, help text
- **Red**: Error messages (when applicable)
- **Black on Cyan**: Selected items (tabs, bookmarks)

## Modern Browser Features Implemented

✅ **Multi-Tab Browsing**: Open multiple pages simultaneously
✅ **Bookmarks**: Save favorite pages for quick access
✅ **History**: Navigate back and forward through visited pages
✅ **Loading Indicators**: Visual feedback during page loads
✅ **Status Updates**: Real-time information about browser state
✅ **Help System**: Built-in keyboard shortcuts reference
✅ **Keyboard-First**: All features accessible via keyboard
✅ **URL Auto-completion**: Smart protocol handling
✅ **Panel Focus**: Tab key cycles through all UI elements
✅ **Responsive Layout**: Adapts to terminal size
