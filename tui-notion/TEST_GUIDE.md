#!/bin/bash

# TUI Notion Manual Test Script
# This script provides instructions for manually testing the application

cat << 'EOF'
╔════════════════════════════════════════════════════════════════════════════╗
║                      TUI NOTION - MANUAL TEST GUIDE                        ║
╚════════════════════════════════════════════════════════════════════════════╝

To test the TUI Notion application, follow these steps:

1. BUILD THE APPLICATION:
   $ cd tui-notion
   $ cargo build --release

2. RUN THE APPLICATION:
   $ cargo run --release

3. TEST DOCUMENT TREE (LEFT PANEL):
   - Use ↓/j and ↑/k to navigate documents
   - Press Enter to open a document
   - Observe that the selected document is highlighted

4. TEST EDITOR (CENTER PANEL):
   - Press Tab to focus on the editor
   - In NORMAL mode, use j/k/h/l or arrow keys to move cursor
   - Observe cursor moving as expected
   - Press 'i' to enter INSERT mode
   - Type some markdown content:
     # My First Heading
     ## A Subheading
     Some text here
     - List item 1
     - List item 2
   - Press Esc to return to NORMAL mode
   - Test cursor movement in NORMAL mode again
   - Observe syntax highlighting (headings in blue/cyan/green)

5. TEST TABLE OF CONTENTS (RIGHT PANEL):
   - Press Tab twice to focus on TOC
   - Observe that headings from the editor appear here
   - Use ↓/j and ↑/k to navigate headings
   - Press Enter on a heading to jump to it in the editor

6. TEST DOCUMENT CREATION:
   - Press Ctrl+N to create a new document
   - Observe new document appears in the tree
   - It's automatically selected and ready to edit

7. TEST SEARCH DIALOG:
   - Press Ctrl+K to open search
   - Type a search query
   - Use ↓/↑ to navigate results
   - Press Enter to open a document
   - Press Esc to close search dialog

8. TEST DOCUMENT DELETION:
   - Select a document in the tree
   - Press Ctrl+D to delete it
   - Confirm it's removed from the tree

9. TEST PERSISTENCE:
   - Create a document with some content
   - Press Ctrl+S to save (or press Esc to auto-save)
   - Quit with 'q'
   - Run the application again
   - Observe that your documents are still there

10. TEST KEYBOARD NAVIGATION:
    - Verify no mouse is needed for any operation
    - Test all shortcuts documented in README

╔════════════════════════════════════════════════════════════════════════════╗
║                            EXPECTED BEHAVIOR                               ║
╚════════════════════════════════════════════════════════════════════════════╝

✓ Three panels visible at all times
✓ Markdown syntax highlighting in editor
✓ Live TOC updates as you type
✓ Documents persist across sessions
✓ No mouse needed for any operation
✓ Vi-style and arrow key navigation work
✓ All Ctrl shortcuts function correctly

╔════════════════════════════════════════════════════════════════════════════╗
║                              QUICK REFERENCE                               ║
╚════════════════════════════════════════════════════════════════════════════╝

GLOBAL:
  q       - Quit
  Tab     - Cycle panels
  Ctrl+K  - Search dialog
  Ctrl+N  - New document
  Ctrl+S  - Save document
  Ctrl+D  - Delete document

EDITOR:
  i       - Enter INSERT mode
  Esc     - Exit INSERT mode
  j/k/h/l ↓/↑/←/→ - Move cursor (NORMAL mode)
  Home/End   - Line start/end (NORMAL mode)
  PgUp/Dn - Page scroll

INSERT MODE:
  Arrow keys - Move cursor
  Home/End   - Line start/end
  Enter      - New line
  Backspace  - Delete char

DATA LOCATION:
  ~/.tui-notion/*.json - Document storage

EOF
