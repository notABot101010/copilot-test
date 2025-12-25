# Testing Guide for TUI Book Reader

## Manual Testing

### Prerequisites
1. Build the project:
   ```bash
   cargo build --package tui-book
   ```

2. Obtain a sample EPUB file (e.g., from Project Gutenberg):
   ```bash
   wget https://www.gutenberg.org/ebooks/11.epub.noimages -O alice.epub
   ```

### Test Scenarios

#### Test 1: Basic Launch
**Steps:**
1. Run `./target/debug/tui-book alice.epub`
2. Verify the application launches with:
   - TOC panel on the left
   - Content panel on the right
   - Content panel focused (cyan border)

**Expected:** Application displays correctly with no errors

#### Test 2: TOC Navigation
**Steps:**
1. Press `Tab` to focus TOC panel
2. Verify TOC panel has cyan border
3. Press `↓` arrow key several times
4. Verify selection moves down (yellow text with `>` marker)
5. Press `↑` arrow key
6. Verify selection moves up

**Expected:** Navigation works smoothly, selection is clearly visible

#### Test 3: Section Jumping
**Steps:**
1. With TOC focused, navigate to a chapter
2. Press `Enter`
3. Verify content panel updates to show selected chapter
4. Verify focus switches to content panel

**Expected:** Content changes to selected section, smooth transition

#### Test 4: Content Scrolling
**Steps:**
1. Ensure content panel is focused (press `Tab` if needed)
2. Press `↓` arrow multiple times
3. Verify content scrolls down
4. Press `↑` arrow
5. Verify content scrolls up

**Expected:** Content scrolls smoothly within bounds

#### Test 5: Toggle TOC
**Steps:**
1. Press `Ctrl+B`
2. Verify TOC panel disappears
3. Verify content panel expands to full width
4. Press `Ctrl+B` again
5. Verify TOC panel reappears

**Expected:** TOC toggles on/off correctly, layout adjusts

#### Test 6: Focus Switching
**Steps:**
1. With TOC visible, press `Tab` repeatedly
2. Verify focus alternates between TOC and content
3. Verify border color changes (cyan for focused, gray for unfocused)
4. Hide TOC with `Ctrl+B`
5. Press `Tab`
6. Verify nothing changes (Tab does nothing when TOC is hidden)

**Expected:** Focus switching works correctly, visual feedback is clear

#### Test 7: Quit Application
**Steps:**
1. Press `q`
2. Verify application exits cleanly
3. Verify terminal is restored to normal state

**Expected:** Clean exit with no terminal artifacts

### Edge Cases

#### Empty or Invalid EPUB
**Steps:**
1. Try running with a non-existent file
2. Try running with a corrupted EPUB

**Expected:** Clear error message, no crash

#### Very Long Content
**Steps:**
1. Navigate to a long chapter
2. Scroll to the end
3. Try to scroll further

**Expected:** Scrolling stops at content boundary, no crash

#### Small Terminal
**Steps:**
1. Resize terminal to minimum size
2. Verify application still renders

**Expected:** Graceful handling of small terminal sizes

## Automated Testing

Currently, the project focuses on manual testing due to the TUI nature. Future improvements could include:
- Integration tests for EPUB parsing
- Unit tests for text wrapping logic
- Mock terminal testing with ratatui's testing utilities

## Performance Testing

For large EPUB files:
1. Measure startup time
2. Check memory usage
3. Verify smooth scrolling
4. Test rapid navigation

**Expected:** Reasonable performance even with large books (>1MB)
