pub struct Editor {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    scroll_offset: usize,
    viewport_height: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
            viewport_height: 20, // Default viewport height
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
            self.cursor_line = (self.scroll_offset + self.viewport_height - 1).min(self.lines.len().saturating_sub(1));
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
            self.cursor_line = (self.scroll_offset + self.viewport_height - 1).min(self.lines.len().saturating_sub(1));
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_down_moves_cursor() {
        let mut editor = Editor::new();
        editor.set_viewport_height(10);
        
        // Create a document with many lines
        let content = (0..50).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
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
        let content = (0..50).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
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
        let content = (0..50).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
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
        let content = (0..50).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
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
        let content = (0..20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
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
}
