# TUI Calendar

A terminal-based calendar application built with Rust and ratatui.

## Features

- **Monthly calendar view** with current day highlighting
- **Day view panel** showing hourly breakdown (00:00-23:00) with events
- **Dynamic vertical sizing** - calendar uses maximum available vertical space
- **Create events** with Ctrl+N
- **Navigate dates** using arrow keys
  - Up/Down arrows move by weeks (7 days)
  - Left/Right arrows move by days
  - Vim-style count prefix supported (e.g., "3→" moves 3 days right, "2↓" moves 2 weeks down)
- **Jump to today** with Ctrl+T
- **Event indicators** on dates with events
- **Cursor support** in input fields with tui-input

## Usage

```bash
cargo run
```

## Keyboard Shortcuts

### Calendar Navigation
- **Arrow Keys**: Navigate between dates
  - **Up/Down**: Move by weeks (7 days)
  - **Left/Right**: Move by days
  - **[count] + Arrow**: Move by count × direction (e.g., 5→ moves 5 days right, 2↑ moves 2 weeks up)
- **,** (comma): Previous month
- **.** (period): Next month
- **Ctrl+T**: Jump to today's date
- **Esc**: Clear number buffer

### Event Management
- **Ctrl+N**: Create new event

### General
- **Q**: Quit application
- **Esc**: Close modals/dialogs or clear number buffer

## Event Creation/Editing

When creating or editing an event:
- **Tab**: Move to next field
- **Enter**: Save event
- **Esc**: Cancel
- **Arrow Keys**: Move cursor within field
- **Home/End**: Jump to start/end of field

Fields:
- **Title**: Event title (required)
- **Description**: Event description (optional)
- **Date**: Event date in YYYY-MM-DD format
- **Time**: Event time in HH:MM format (optional)

## Vim-Style Count Prefix

You can enter a number before using arrow keys to move by that many units:
- **3→**: Move 3 days right
- **2←**: Move 2 days left
- **2↑**: Move 2 weeks up (14 days)
- **5↓**: Move 5 weeks down (35 days)

The number buffer is cleared after each movement or when pressing Esc.

## Visual Indicators

### Day Cell Styling
- **Green border**: Current day (today)
- **Cyan border with blue background**: Selected date
- **Magenta border**: Dates with events
- **Gray border**: Regular days

### Event Display in Calendar
- Each day cell shows event previews with time and title (more with larger cells)
- Event titles are truncated to fit within the cell
- **"+N more"** indicator appears when there are more events than fit in the cell
- Events are displayed in chronological order (sorted by time)

### Calendar Layout
- **Box-based design**: Each day is rendered as a bordered box (similar to GUI calendars)
- **Dynamic cell sizing**: Days automatically scale to use available vertical space
- **Weekday headers**: Two-letter abbreviations (Su, Mo, Tu, etc.) centered above columns
- **Event preview**: See event times and titles directly in the calendar grid

### Day View Panel (Right Side)
- **Hourly breakdown**: Shows 24 hours from 00:00 to 23:00
- **All-day events**: Listed at the top if they have no specific time
- **Timed events**: Displayed in their corresponding hour slot
- **Event details**: Shows event time and title for the selected date
