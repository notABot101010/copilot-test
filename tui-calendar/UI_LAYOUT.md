# TUI Calendar - UI Layout

This document describes the visual layout of the TUI Calendar application.

## Main View Layout

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                     TUI Calendar Main View                                   │
├─────────────────────────────────────┬────────────────────────────────────────┤
│  Calendar View (70%)                │  Day View Panel (30%)                  │
│                                     │                                        │
│  ┌─────────────────────────────┐  │  ┌──────────────────────────────────┐ │
│  │   January 2024              │  │  │  Day View - 2024-01-15 (Monday)  │ │
│  ├─────────────────────────────┤  │  ├──────────────────────────────────┤ │
│  │ Su  Mo  Tu  We  Th  Fr  Sa  │  │  │  All Day                         │ │
│  ├─────────────────────────────┤  │  │    • Team Meeting                │ │
│  │ [1] [2] [3] [4] [5] [6] [7] │  │  │                                  │ │
│  │ [8] [9][10][11][12][13][14] │  │  │  09:00 Project Review            │ │
│  │[15][16][17][18][19][20][21] │  │  │  14:00-15:00 Client Call         │ │
│  │[22][23][24][25][26][27][28] │  │  │  16:00 Code Review               │ │
│  │[29][30][31]                 │  │  │                                  │ │
│  ├─────────────────────────────┤  │  ├──────────────────────────────────┤ │
│  │ Arrows: Navigate  W: Week   │  │  │ Tab: Focus Events  N: New        │ │
│  │ N: New  T: Today  H: Help   │  │  │ /: Search                        │ │
│  └─────────────────────────────┘  │  └──────────────────────────────────┘ │
│                                     │                                        │
└─────────────────────────────────────┴────────────────────────────────────────┘
 NORMAL | 2024-01-15 Monday | Press 'h' for help                               
```

## Help Dialog (Press 'h')

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                     Help - Keyboard Shortcuts                                │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Navigation                                                                  │
│    Arrow Keys: Navigate dates (Up/Down: weeks, Left/Right: days)           │
│    , (comma): Previous month                                                 │
│    . (period): Next month                                                    │
│    t: Jump to today                                                          │
│    w: Toggle week view                                                       │
│                                                                              │
│  Event Management                                                            │
│    n: Create new event                                                       │
│    /: Search events                                                          │
│    Tab: Focus event list panel                                              │
│    Enter: (in event list) Edit selected event                              │
│                                                                              │
│  Event List Panel (when focused with Tab)                                   │
│    Up/Down: Navigate events                                                  │
│    Enter: Edit selected event                                               │
│    Esc: Return to calendar                                                   │
│                                                                              │
│  [More sections...]                                                          │
│                                                                              │
│  Press Esc to close                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Search Dialog (Press '/')

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          Search Events                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Search:                                                                     │
│  meeting_                                                                    │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │ Search Results                                                         │ │
│  ├────────────────────────────────────────────────────────────────────────┤ │
│  │ > 09:00 2024-01-15 - Team Meeting                                      │ │
│  │   14:00-15:00 2024-01-20 - Client Meeting                              │ │
│  │   2024-01-25 - All Hands Meeting                                       │ │
│  │                                                                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ↑/↓: Navigate  Enter: View  Esc: Close                                     │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Event List Focused (Press Tab)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  Calendar View (70%)                │  Day View Panel (30%) - FOCUSED      │
│                                     │                                        │
│  ┌─────────────────────────────┐  │  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ │
│  │   January 2024              │  │  ┃  Day View - 2024-01-15 (Monday) ┃ │
│  ├─────────────────────────────┤  │  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫ │
│  │ Su  Mo  Tu  We  Th  Fr  Sa  │  │  ┃  All Day                        ┃ │
│  ├─────────────────────────────┤  │  ┃  > • Team Meeting               ┃ │
│  │ [1] [2] [3] [4] [5] [6] [7] │  │  ┃                                 ┃ │
│  │ [8] [9][10][11][12][13][14] │  │  ┃  09:00 Project Review           ┃ │
│  │[15][16][17][18][19][20][21] │  │  ┃  14:00-15:00 Client Call        ┃ │
│  │[22][23][24][25][26][27][28] │  │  ┃  16:00 Code Review              ┃ │
│  │[29][30][31]                 │  │  ┃                                 ┃ │
│  ├─────────────────────────────┤  │  ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫ │
│  │ Arrows: Navigate  W: Week   │  │  ┃ ↑/↓: Navigate  Enter: Edit     ┃ │
│  │ N: New  T: Today  H: Help   │  │  ┃ Esc: Back                      ┃ │
│  └─────────────────────────────┘  │  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                     │                                        │
└─────────────────────────────────────┴────────────────────────────────────────┘
 EVENT LIST | 2024-01-15 Monday | Press 'h' for help                           
```

## Status Bar States

The status bar at the bottom changes color based on the current mode:

- **NORMAL** (Dark Gray): Default calendar view
- **EVENT LIST** (Cyan): Event list panel is focused
- **SEARCH** (Yellow): Search dialog is open
- **HELP** (Green): Help dialog is displayed
- **CREATE EVENT** (Dark Gray): Creating a new event
- **EDIT EVENT** (Dark Gray): Editing an existing event

## Visual Indicators

### Calendar Cells
- **Green border**: Today's date
- **Cyan border + Blue background**: Selected date
- **Magenta border**: Date has events
- **Gray border**: Regular day

### Event Colors (by category)
- **Cyan**: Work
- **Green**: Personal
- **Yellow**: Meeting
- **Red**: Important
- **White**: No category

### Panel Focus
- **Double-line border (━)**: Panel is focused (event list)
- **Single-line border (─)**: Normal state

## Database Location

Events are persisted to: `~/.tuicalendar/tuicalendar.db`
- Created automatically on first run
- All events saved immediately on create/edit/delete
