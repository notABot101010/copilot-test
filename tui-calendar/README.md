# TUI Calendar

A terminal-based calendar application built with Rust and ratatui.

## Features

- **Monthly calendar view** with current day highlighting
- **Create events** with Ctrl+N
- **View event details** by selecting an event and pressing Enter
- **Delete events** with confirmation dialog
- **Navigate dates** using arrow keys
- **Event indicators** on dates with events

## Usage

```bash
cargo run
```

## Keyboard Shortcuts

### Calendar Navigation
- **Arrow Keys**: Navigate between dates
- **,** (comma): Previous month
- **.** (period): Next month
- **Tab**: Toggle between calendar and event list

### Event Management
- **Ctrl+N**: Create new event
- **Enter**: View details of selected event
- **Delete**: Delete selected event (with confirmation)

### General
- **Q**: Quit application
- **Esc**: Close modals/dialogs

## Event Creation

When creating an event (Ctrl+N):
- **Tab**: Move to next field
- **Enter**: Save event
- **Esc**: Cancel

Fields:
- **Title**: Event title (required)
- **Description**: Event description (optional)
- **Date**: Event date in YYYY-MM-DD format
- **Time**: Event time in HH:MM format (optional)

## Visual Indicators

- **Green**: Current day
- **Cyan (highlighted)**: Selected date
- **Magenta with asterisk (*)**: Dates with events
