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
        let max_scroll = self.lines.len().saturating_sub(1);
        if self.scroll_offset + scroll_amount < max_scroll {
            self.scroll_offset += scroll_amount;
        } else {
            self.scroll_offset = max_scroll.saturating_sub(self.viewport_height.saturating_sub(1));
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
