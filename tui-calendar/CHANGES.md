# TUI Calendar - Recent Changes

## Summary of Improvements

This document describes the recent improvements made to the TUI Calendar application.

### 1. SQLite Database Storage

**Location**: `~/.tuicalendar/tuicalendar.db`

- Added persistent storage using SQLite with rusqlite
- Events are now automatically saved to and loaded from the database
- Database is created automatically on first run
- All event operations (create, edit, delete) now persist to disk

**Implementation**:
- New `database.rs` module with `Database` struct
- Schema includes all event fields: id, title, description, start_date, end_date, start_time, end_time, category
- Automatic loading of events on application startup
- Save operations on create/edit/delete

### 2. Improved Keyboard Shortcuts

**Changed shortcuts**:
- `t` (instead of `Ctrl+T`) - Jump to today's date
- `n` (instead of `Ctrl+N`) - Create new event
- `h` - Display help dialog with all shortcuts and relevant info
- `/` - Open search dialog

**New shortcuts**:
- `Tab` - Focus the event list panel (right side)
- `Enter` (in event list) - Edit selected event
- `Esc` (in event list) - Return to calendar view
- `Up/Down` (in event list) - Navigate through events

### 3. Help Dialog

Press `h` to display a comprehensive help dialog showing:
- All keyboard shortcuts organized by category
- Navigation commands
- Event management features
- Search functionality
- Event creation/editing shortcuts
- Event category colors

### 4. Search Functionality

Press `/` to open the search dialog featuring:
- Search events by title or description
- Real-time search results as you type
- Display of recently viewed events when search input is empty
- Navigate results with Up/Down arrows
- Press Enter to view selected event
- Tracks up to 10 recently viewed events

### 5. Event List Focus Mode

Press `Tab` to focus the event list panel:
- Highlighted border (cyan) indicates focus
- Navigate events with Up/Down arrows
- Press Enter to edit selected event
- Press Esc to return to calendar view
- Status bar shows "EVENT LIST" mode

### 6. Bottom Status Bar

A vim-style status bar at the bottom displays:
- Current mode (NORMAL, EVENT LIST, SEARCH, etc.)
- Current date and day of week
- Quick reminder: "Press 'h' for help"
- Color-coded by mode:
  - EVENT LIST: Cyan background
  - SEARCH: Yellow background
  - HELP: Green background
  - Default: Dark gray background

### 7. UI Improvements

- Updated help text in calendar and day view panels
- Visual indication when event list is focused
- Better visual feedback for different app modes
- Consistent color scheme throughout

## Technical Details

### New Dependencies

Added to `Cargo.toml`:
```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```

### New App Modes

- `Help` - Display help dialog
- `Search` - Search events dialog
- `EventListFocused` - Event list panel has focus

### Database Schema

```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    start_date TEXT NOT NULL,
    end_date TEXT,
    start_time TEXT,
    end_time TEXT,
    category TEXT
);
```

### File Structure

- `src/main.rs` - Main application logic with updated keyboard handling
- `src/database.rs` - New module for SQLite database operations

## Usage Examples

1. **Jump to Today**: Press `t`
2. **Create Event**: Press `n`
3. **Search Events**: Press `/`, type search query
4. **View Help**: Press `h`
5. **Focus Event List**: Press `Tab`
6. **Edit Event from List**: Press `Tab` to focus, navigate with arrows, press `Enter`
7. **View Recently Viewed**: Press `/` without typing anything

## Backward Compatibility

All existing functionality remains intact. The old keyboard shortcuts (`Ctrl+T`, `Ctrl+N`) have been replaced with simpler alternatives, making the application more accessible and easier to use.

Events created in previous versions (if stored in memory) will need to be re-created after this update since we now use persistent storage.
