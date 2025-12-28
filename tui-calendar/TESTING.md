# TUI Calendar - Manual Testing Guide

This guide outlines how to manually test all features of the TUI Calendar application.

## Prerequisites

Build the application:
```bash
cargo build --release
```

Run the application:
```bash
cargo run --release
```

## Test Cases

### 1. Calendar View (Monthly)
**Expected:** Monthly calendar grid showing December 2025 (current month)
- [ ] Calendar displays days of the week (Su-Sa)
- [ ] Calendar shows all days of current month
- [ ] Current day (today) is highlighted in green
- [ ] Selected date is highlighted with cyan background

### 2. Date Navigation
**Test:** Use arrow keys to navigate dates
- [ ] **Left Arrow**: Moves selection to previous day
- [ ] **Right Arrow**: Moves selection to next day  
- [ ] **Up Arrow**: Moves selection up one week (7 days back)
- [ ] **Down Arrow**: Moves selection down one week (7 days forward)
- [ ] **Comma (,)**: Navigate to previous month
- [ ] **Period (.)**: Navigate to next month
- [ ] **Arrow navigation automatically switches months** when crossing month boundaries

**Test:** Vim-style count prefix for navigation
- [ ] Press **3** then **→** (right arrow): Moves 3 days forward
- [ ] Press **2** then **←** (left arrow): Moves 2 days backward
- [ ] Press **2** then **↑** (up arrow): Moves 2 weeks up (14 days back)
- [ ] Press **3** then **↓** (down arrow): Moves 3 weeks down (21 days forward)
- [ ] Press **Esc**: Clears the number buffer
- [ ] Any movement clears the number buffer
- [ ] Number buffer is limited to 4 digits (9999 max)

### 3. Create New Event (Ctrl+N)
**Test:** Press Ctrl+N to open event creation modal
- [ ] Modal appears with "Create New Event" title
- [ ] Four input fields visible: Title, Description, Date, Time
- [ ] Title field is highlighted (active field)
- [ ] Date field is pre-filled with selected date

**Test:** Fill in event details
- [ ] Type text in Title field
- [ ] Cursor is visible and moves as you type
- [ ] Arrow keys move cursor within field
- [ ] Home/End keys jump to start/end of field
- [ ] Press Tab to move to Description field (field highlights in yellow)
- [ ] Type text in Description field
- [ ] Press Tab to move to Date field
- [ ] Date field accepts YYYY-MM-DD format
- [ ] Press Tab to move to Time field
- [ ] Time field accepts HH:MM format
- [ ] Tab wraps around from Time back to Title

**Test:** Save event
- [ ] Press Enter to save event
- [ ] Modal closes
- [ ] Event appears in event list for that date
- [ ] Date with event shows asterisk (*) in calendar

**Test:** Cancel event creation
- [ ] Press Ctrl+N to open modal
- [ ] Press Esc
- [ ] Modal closes without saving
- [ ] No event is created

### 4. View Events
**Test:** Navigate to a date with events
- [ ] Events list shows all events for selected date
- [ ] Events display in chronological order
- [ ] Events with time show time before title (e.g., "14:30 - Meeting")
- [ ] Events without time show only title

**Test:** Navigate event list
- [ ] Press Tab to focus on event list
- [ ] Use Up/Down arrows to navigate between events
- [ ] Selected event is highlighted with gray background and ">" symbol
- [ ] Press Tab again to return focus to calendar

### 5. View Event Details (Enter)
**Test:** Select an event and press Enter
- [ ] Modal appears with "Event Details" title
- [ ] Modal shows:
  - Event title
  - Event date
  - Event time (if set)
  - Event description (or "(No description)")
- [ ] Text wraps properly if description is long

**Test:** Close event details
- [ ] Press Esc to close modal
- [ ] Returns to calendar view
- [ ] Event remains selected in list

### 5. Edit Event (E Key)
**Test:** Select an event and press E
- [ ] Modal appears with "Edit Event" title
- [ ] Four input fields visible: Title, Description, Date, Time
- [ ] Title field is highlighted (active field)
- [ ] Fields are pre-filled with existing event data

**Test:** Edit event details
- [ ] Cursor is visible in current field
- [ ] Arrow keys move cursor within field
- [ ] Home/End keys jump to start/end of field
- [ ] Can modify Title field
- [ ] Press Tab to move to Description field
- [ ] Can modify Description field
- [ ] Press Tab to move to Date field
- [ ] Can change date (YYYY-MM-DD format)
- [ ] Press Tab to move to Time field
- [ ] Can change or add/remove time (HH:MM format)
- [ ] Tab wraps around from Time back to Title

**Test:** Save edited event
- [ ] Press Enter to save changes
- [ ] Modal closes
- [ ] Event appears in list with updated information
- [ ] Event is re-sorted by date/time if date or time was changed
- [ ] If date changed, event moves to new date

**Test:** Cancel editing
- [ ] Press E to edit an event
- [ ] Make some changes
- [ ] Press Esc
- [ ] Modal closes without saving
- [ ] Event data is unchanged

### 6. Delete Event (Delete Key)
**Test:** Select an event and press Delete
- [ ] Confirmation modal appears with "Confirm Delete" title
- [ ] Modal asks "Are you sure you want to delete this event?"
- [ ] Shows two options: Y (Yes) and N/Esc (No)

**Test:** Confirm deletion
- [ ] Press Y
- [ ] Modal closes
- [ ] Event is removed from list
- [ ] If date has no more events, asterisk (*) is removed from calendar
- [ ] Selection moves to previous event (or first event, or none if list is empty)

**Test:** Cancel deletion
- [ ] Press Delete on an event
- [ ] Press N or Esc
- [ ] Modal closes
- [ ] Event is NOT deleted

### 7. Edge Cases and Multi-Event Scenarios
**Test:** Multiple events on same date
- [ ] Create 3+ events on same date with different times
- [ ] Events appear sorted by time
- [ ] Events without time appear after timed events
- [ ] Can navigate through all events with Up/Down

**Test:** Events across different dates
- [ ] Create events on different dates
- [ ] Each date with events shows asterisk (*)
- [ ] Navigate to each date
- [ ] Correct events appear for each date

**Test:** Empty date
- [ ] Navigate to date without events
- [ ] Event list shows "No events for this day"
- [ ] Cannot press Enter or Delete (no events selected)

### 8. General UI
**Test:** Help text visibility
- [ ] Calendar shows help: "Arrows: Navigate  Ctrl+N: New Event  Q: Quit"
- [ ] Event list shows: "Enter: View  E: Edit  Del: Delete"

**Test:** Quit application
- [ ] Press Q
- [ ] Application exits cleanly
- [ ] Terminal returns to normal state

## Expected Visual Elements

### Colors:
- **Green**: Current day (today)
- **Cyan background**: Selected date
- **Magenta**: Dates with events (with asterisk)
- **Yellow**: Active input field in modals
- **Red**: "Yes, delete" option in confirmation
- **Gray**: Highlighted event in list

### Layout:
- Calendar occupies ~70% of screen width (left side)
- Event list occupies ~30% of screen width (right side)
- Modals are centered and sized appropriately

## Notes
- All modals have proper borders and titles
- Text input is responsive and immediate
- No crashes or panics should occur during normal operation
- Application handles invalid date/time formats gracefully
