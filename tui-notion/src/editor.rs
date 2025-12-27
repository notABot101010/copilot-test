use crate::history::{EditorState, History};

pub struct Editor {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
    viewport_height: usize,
    history: History,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            viewport_height: 20, // Default viewport height
            history: History::new(),
        }
    }

    pub fn set_content(&mut self, content: String) {
        if content.is_empty() {
            self.lines = vec![String::new()];
        } else {
            self.lines = content.lines().map(|s| s.to_string()).collect();
        }
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    pub fn insert_char(&mut self, c: char) {
        // Ensure we have at least one line
        if self.lines.is_empty() {
            self.lines.push(String::new());
            self.cursor_line = 0;
            self.cursor_col = 0;
        }

        if self.cursor_line >= self.lines.len() {
            self.cursor_line = self.lines.len().saturating_sub(1);
        }
        self.lines[self.cursor_line].insert(self.cursor_col, c);
        self.cursor_col += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_col > 0 {
            self.lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
        }
    }

    pub fn insert_newline(&mut self) {
        let current_line = &self.lines[self.cursor_line];
        let remaining = current_line[self.cursor_col..].to_string();
        self.lines[self.cursor_line].truncate(self.cursor_col);
        self.cursor_line += 1;
        self.lines.insert(self.cursor_line, remaining);
        self.cursor_col = 0;

        // Ensure cursor stays visible after newline
        self.ensure_cursor_visible();
    }

    fn ensure_cursor_visible(&mut self) {
        // Adjust scroll offset to keep cursor visible
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = self.cursor_line.saturating_sub(self.viewport_height - 1);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor left without wrapping to previous line
    pub fn move_cursor_left_no_wrap(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// Move cursor right without wrapping to next line
    pub fn move_cursor_right_no_wrap(&mut self) {
        // Safety check: ensure cursor_line is within bounds
        if self.cursor_line < self.lines.len() {
            let line_len = self.lines[self.cursor_line].len();
            if self.cursor_col < line_len {
                self.cursor_col += 1;
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let line_len = self.lines[self.cursor_line].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
            self.ensure_cursor_visible();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            let line_len = self.lines[self.cursor_line].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
            self.ensure_cursor_visible();
        }
    }

    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height.max(1);
        self.ensure_cursor_visible();
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_line].len();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
        // Move cursor up if it's below the visible area
        if self.cursor_line >= self.scroll_offset + self.viewport_height {
            self.cursor_line = (self.scroll_offset + self.viewport_height - 1)
                .min(self.lines.len().saturating_sub(1));
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset + 1 < self.lines.len() {
            self.scroll_offset += 1;
            // Move cursor down if it's above the visible area
            if self.cursor_line < self.scroll_offset {
                self.cursor_line = self.scroll_offset;
            }
        }
    }

    pub fn page_up(&mut self) {
        let scroll_amount = self.viewport_height.min(10);
        self.scroll_offset = self.scroll_offset.saturating_sub(scroll_amount);
        // Move cursor up if it's below the visible area
        if self.cursor_line >= self.scroll_offset + self.viewport_height {
            self.cursor_line = (self.scroll_offset + self.viewport_height - 1)
                .min(self.lines.len().saturating_sub(1));
        }
    }

    pub fn page_down(&mut self) {
        let scroll_amount = self.viewport_height.min(10);
        let new_offset = self.scroll_offset + scroll_amount;
        if new_offset < self.lines.len() {
            self.scroll_offset = new_offset;
        } else {
            self.scroll_offset = self.lines.len().saturating_sub(1);
        }
        // Move cursor down if it's above the visible area
        if self.cursor_line < self.scroll_offset {
            self.cursor_line = self.scroll_offset;
        }
    }

    pub fn jump_to_line(&mut self, line: usize) {
        if line < self.lines.len() {
            self.cursor_line = line;
            self.cursor_col = 0;
            // Center the line in the view
            self.scroll_offset = line.saturating_sub(10);
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_line, self.cursor_col)
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Save current state to history before making changes
    pub fn save_state(&mut self) {
        let state = EditorState {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        };
        self.history.push(state);
    }

    /// Restore editor state from a saved state
    fn restore_state(&mut self, state: EditorState) {
        self.lines = state.lines;
        self.cursor_line = state.cursor_line;
        self.cursor_col = state.cursor_col;
        self.ensure_cursor_visible();
    }

    /// Undo the last change
    pub fn undo(&mut self) -> bool {
        let current_state = EditorState {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        };

        if let Some(previous_state) = self.history.undo(current_state) {
            self.restore_state(previous_state);
            true
        } else {
            false
        }
    }

    /// Redo a previously undone change
    pub fn redo(&mut self) -> bool {
        let current_state = EditorState {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        };

        if let Some(next_state) = self.history.redo(current_state) {
            self.restore_state(next_state);
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Clear undo/redo history (useful when loading a new document)
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_down_moves_cursor() {
        let mut editor = Editor::new();
        editor.set_viewport_height(10);

        // Create a document with many lines
        let content = (0..50)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        editor.set_content(content);

        // Initial state: cursor at line 0, scroll at 0
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.scroll_offset, 0);

        // Scroll down 5 times - cursor should move to stay visible
        for _ in 0..5 {
            editor.scroll_down();
        }
        assert_eq!(editor.scroll_offset, 5);
        assert_eq!(editor.cursor_line, 5); // Moved to stay at top of viewport

        // Scroll down 10 more times - cursor should now move to stay visible
        for _ in 0..10 {
            editor.scroll_down();
        }
        assert_eq!(editor.scroll_offset, 15);
        // Cursor should have moved to at least scroll_offset (15)
        assert!(editor.cursor_line >= editor.scroll_offset);
    }

    #[test]
    fn test_scroll_up_moves_cursor() {
        let mut editor = Editor::new();
        editor.set_viewport_height(10);

        // Create a document with many lines
        let content = (0..50)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        editor.set_content(content);

        // Position cursor and scroll at line 30
        editor.cursor_line = 30;
        editor.scroll_offset = 25;

        // Scroll up - cursor should stay at 30 (still within viewport 25-34)
        editor.scroll_up();
        assert_eq!(editor.scroll_offset, 24);
        assert_eq!(editor.cursor_line, 30);

        // Scroll up 10 more times
        for _ in 0..10 {
            editor.scroll_up();
        }
        assert_eq!(editor.scroll_offset, 14);
        // Cursor should be within visible range
        assert!(editor.cursor_line >= editor.scroll_offset);
        assert!(editor.cursor_line < editor.scroll_offset + editor.viewport_height);
    }

    #[test]
    fn test_page_down_keeps_cursor_visible() {
        let mut editor = Editor::new();
        editor.set_viewport_height(10);

        // Create a document with many lines
        let content = (0..50)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        editor.set_content(content);

        // Initial state
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.scroll_offset, 0);

        // Page down once
        editor.page_down();

        // Cursor should be visible
        assert!(editor.cursor_line >= editor.scroll_offset);
        assert!(editor.cursor_line < editor.scroll_offset + editor.viewport_height);
    }

    #[test]
    fn test_page_up_keeps_cursor_visible() {
        let mut editor = Editor::new();
        editor.set_viewport_height(10);

        // Create a document with many lines
        let content = (0..50)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        editor.set_content(content);

        // Position cursor and scroll far down
        editor.cursor_line = 40;
        editor.scroll_offset = 35;

        // Page up
        editor.page_up();

        // Cursor should be visible
        assert!(editor.cursor_line >= editor.scroll_offset);
        assert!(editor.cursor_line < editor.scroll_offset + editor.viewport_height);
    }

    #[test]
    fn test_cursor_movement_in_normal_mode() {
        let mut editor = Editor::new();
        editor.set_viewport_height(10);

        // Create a simple document
        let content = "Line 0\nLine 1\nLine 2\nLine 3\nLine 4".to_string();
        editor.set_content(content);

        // Start at (0, 0)
        assert_eq!(editor.cursor_position(), (0, 0));

        // Move down
        editor.move_cursor_down();
        assert_eq!(editor.cursor_position(), (1, 0));

        // Move right
        editor.move_cursor_right();
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position(), (1, 2));

        // Move up
        editor.move_cursor_up();
        assert_eq!(editor.cursor_position(), (0, 2));

        // Move left
        editor.move_cursor_left();
        assert_eq!(editor.cursor_position(), (0, 1));

        // Move to line end
        editor.move_cursor_to_line_end();
        assert_eq!(editor.cursor_position(), (0, 6)); // "Line 0" has 6 chars

        // Move to line start
        editor.move_cursor_to_line_start();
        assert_eq!(editor.cursor_position(), (0, 0));
    }

    #[test]
    fn test_cursor_movement_scrolls_viewport() {
        let mut editor = Editor::new();
        editor.set_viewport_height(5);

        // Create a document with many lines
        let content = (0..20)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        editor.set_content(content);

        // Initial state
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.scroll_offset, 0);

        // Move cursor down beyond viewport
        for _ in 0..10 {
            editor.move_cursor_down();
        }

        // Cursor should be at line 10
        assert_eq!(editor.cursor_line, 10);

        // Scroll offset should have adjusted to keep cursor visible
        assert!(editor.scroll_offset > 0);
        assert!(editor.cursor_line >= editor.scroll_offset);
        assert!(editor.cursor_line < editor.scroll_offset + editor.viewport_height);
    }

    #[test]
    fn test_undo_redo_insert_char() {
        let mut editor = Editor::new();
        editor.set_content("hello".to_string());
        editor.cursor_col = 5;

        // Save state and insert a character
        editor.save_state();
        editor.insert_char('!');
        assert_eq!(editor.get_content(), "hello!");

        // Undo should restore previous state
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "hello");
        assert_eq!(editor.cursor_col, 5);

        // Redo should restore the change
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "hello!");
        assert_eq!(editor.cursor_col, 6);
    }

    #[test]
    fn test_undo_redo_delete_char() {
        let mut editor = Editor::new();
        editor.set_content("hello!".to_string());
        editor.cursor_col = 6;

        // Save state and delete a character
        editor.save_state();
        editor.delete_char();
        assert_eq!(editor.get_content(), "hello");

        // Undo should restore the deleted character
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "hello!");
        assert_eq!(editor.cursor_col, 6);

        // Redo should delete again
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "hello");
    }

    #[test]
    fn test_undo_redo_multiple_changes() {
        let mut editor = Editor::new();
        editor.set_content("".to_string());

        // Make several changes
        editor.save_state();
        editor.insert_char('h');

        editor.save_state();
        editor.insert_char('i');

        editor.save_state();
        editor.insert_char('!');

        assert_eq!(editor.get_content(), "hi!");

        // Undo all changes
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "hi");

        assert!(editor.undo());
        assert_eq!(editor.get_content(), "h");

        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");

        // Can't undo anymore
        assert!(!editor.undo());

        // Redo all changes
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "h");

        assert!(editor.redo());
        assert_eq!(editor.get_content(), "hi");

        assert!(editor.redo());
        assert_eq!(editor.get_content(), "hi!");

        // Can't redo anymore
        assert!(!editor.redo());
    }

    #[test]
    fn test_new_change_clears_redo() {
        let mut editor = Editor::new();
        editor.set_content("hello".to_string());
        editor.cursor_col = 5;

        // Make a change
        editor.save_state();
        editor.insert_char('!');

        // Undo it
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "hello");

        // Make a different change
        editor.save_state();
        editor.insert_char('?');

        // Now redo should not bring back the '!'
        assert!(!editor.redo());
        assert_eq!(editor.get_content(), "hello?");
    }

    #[test]
    fn test_clear_history() {
        let mut editor = Editor::new();
        editor.set_content("hello".to_string());
        editor.cursor_col = 5;

        // Make some changes
        editor.save_state();
        editor.insert_char('!');

        // Clear history
        editor.clear_history();

        // Undo should not work
        assert!(!editor.undo());
        assert_eq!(editor.get_content(), "hello!");
    }

    #[test]
    fn test_cursor_right_stops_at_line_end() {
        let mut editor = Editor::new();
        editor.set_content("hello".to_string());

        // Try to move right 10 times, should stop at end of line (position 5)
        for _ in 0..10 {
            editor.move_cursor_right();
        }

        assert_eq!(editor.cursor_position(), (0, 5));

        // Verify we can't go beyond the end
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position(), (0, 5));
    }

    #[test]
    fn test_cursor_down_stops_at_last_line() {
        let mut editor = Editor::new();
        editor.set_content("Line 1\nLine 2\nLine 3".to_string());

        // Try to move down 10 times, should stop at last line
        for _ in 0..10 {
            editor.move_cursor_down();
        }

        assert_eq!(editor.cursor_position().0, 2); // Line 2 is the last (0-indexed)
    }

    #[test]
    fn test_cursor_left_stops_at_line_start() {
        let mut editor = Editor::new();
        editor.set_content("hello".to_string());
        editor.cursor_col = 3;

        // Try to move left 10 times, should stop at beginning
        for _ in 0..10 {
            editor.move_cursor_left();
        }

        assert_eq!(editor.cursor_position().1, 0); // Column should be 0
    }

    #[test]
    fn test_cursor_up_stops_at_first_line() {
        let mut editor = Editor::new();
        editor.set_content("Line 1\nLine 2\nLine 3".to_string());
        editor.cursor_line = 2;

        // Try to move up 10 times, should stop at first line
        for _ in 0..10 {
            editor.move_cursor_up();
        }

        assert_eq!(editor.cursor_position().0, 0); // Line should be 0
    }

    #[test]
    fn test_cursor_right_wraps_to_next_line() {
        let mut editor = Editor::new();
        editor.set_content("Line 1\nLine 2".to_string());
        editor.cursor_col = 6; // At end of first line

        // Move right should wrap to next line
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position(), (1, 0));
    }

    #[test]
    fn test_cursor_left_wraps_to_previous_line() {
        let mut editor = Editor::new();
        editor.set_content("Line 1\nLine 2".to_string());
        editor.cursor_line = 1;
        editor.cursor_col = 0;

        // Move left should wrap to end of previous line
        editor.move_cursor_left();
        assert_eq!(editor.cursor_position(), (0, 6));
    }

    #[test]
    fn test_multiple_cursor_movements_respect_line_boundaries() {
        let mut editor = Editor::new();
        editor.set_content("Short\nThis is a longer line\nShort".to_string());

        // Start at beginning
        assert_eq!(editor.cursor_position(), (0, 0));

        // Move down to longer line
        editor.move_cursor_down();
        assert_eq!(editor.cursor_position(), (1, 0));

        // Move right 21 times to reach end of line
        for _ in 0..21 {
            editor.move_cursor_right();
        }

        // Should be at end of line (21 chars)
        assert_eq!(editor.cursor_position(), (1, 21));

        // One more right movement should wrap to next line
        editor.move_cursor_right();
        assert_eq!(editor.cursor_position(), (2, 0));
    }

    #[test]
    fn test_cursor_left_no_wrap_stops_at_line_start() {
        let mut editor = Editor::new();
        editor.set_content("hello\nworld".to_string());
        editor.cursor_line = 1;
        editor.cursor_col = 3;

        // Move left 10 times with no_wrap - should stop at beginning of current line
        for _ in 0..10 {
            editor.move_cursor_left_no_wrap();
        }

        // Should be at start of line 1, NOT wrapped to previous line
        assert_eq!(editor.cursor_position(), (1, 0));
    }

    #[test]
    fn test_cursor_right_no_wrap_stops_at_line_end() {
        let mut editor = Editor::new();
        editor.set_content("hello\nworld".to_string());
        editor.cursor_line = 0;
        editor.cursor_col = 0;

        // Move right 10 times with no_wrap - should stop at end of current line
        for _ in 0..10 {
            editor.move_cursor_right_no_wrap();
        }

        // Should be at end of line 0 (5 chars), NOT wrapped to next line
        assert_eq!(editor.cursor_position(), (0, 5));
    }

    #[test]
    fn test_numeric_prefix_movements_dont_cross_lines() {
        let mut editor = Editor::new();
        editor.set_content("Line one\nLine two\nLine three".to_string());

        // Start at line 1, column 2
        editor.cursor_line = 1;
        editor.cursor_col = 2;

        // Move left 5 times with no wrap - should stop at column 0
        for _ in 0..5 {
            editor.move_cursor_left_no_wrap();
        }
        assert_eq!(editor.cursor_position(), (1, 0));

        // Move right 100 times with no wrap - should stop at end of line
        for _ in 0..100 {
            editor.move_cursor_right_no_wrap();
        }
        assert_eq!(editor.cursor_position(), (1, 8)); // "Line two" has 8 chars
    }
}
