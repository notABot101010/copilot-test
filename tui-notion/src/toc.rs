pub struct TocEntry {
    pub level: usize,
    pub title: String,
    pub line: usize,
}

pub struct TableOfContents {
    entries: Vec<TocEntry>,
    selected_index: Option<usize>,
}

impl TableOfContents {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            selected_index: None,
        }
    }

    pub fn update_from_content(&mut self, content: &str) {
        self.entries.clear();
        
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let title = trimmed.trim_start_matches('#').trim().to_string();
                
                if !title.is_empty() && level <= 6 {
                    self.entries.push(TocEntry {
                        level,
                        title,
                        line: line_num,
                    });
                }
            }
        }

        if !self.entries.is_empty() && self.selected_index.is_none() {
            self.selected_index = Some(0);
        }
        
        if let Some(idx) = self.selected_index {
            if idx >= self.entries.len() {
                self.selected_index = if self.entries.is_empty() {
                    None
                } else {
                    Some(self.entries.len() - 1)
                };
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected_index = None;
    }

    pub fn next(&mut self) {
        if let Some(index) = self.selected_index {
            if index < self.entries.len() - 1 {
                self.selected_index = Some(index + 1);
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(index) = self.selected_index {
            if index > 0 {
                self.selected_index = Some(index - 1);
            }
        }
    }

    pub fn selected_line(&self) -> Option<usize> {
        self.selected_index
            .and_then(|idx| self.entries.get(idx))
            .map(|entry| entry.line)
    }

    pub fn entries(&self) -> &[TocEntry] {
        &self.entries
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Synchronize TOC selection based on the current cursor line in the editor
    /// Selects the heading that the cursor is currently at or below
    pub fn sync_with_cursor(&mut self, cursor_line: usize) {
        if self.entries.is_empty() {
            self.selected_index = None;
            return;
        }

        // Find the heading that the cursor is currently at or below
        // We want the last heading that starts before or at the cursor position
        let mut best_match_idx = 0;
        
        for (idx, entry) in self.entries.iter().enumerate() {
            if entry.line <= cursor_line {
                best_match_idx = idx;
            } else {
                // We've found a heading after the cursor, stop here
                break;
            }
        }
        
        self.selected_index = Some(best_match_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_with_cursor_selects_first_heading() {
        let mut toc = TableOfContents::new();
        let content = r#"# First Heading
Some content here
## Second Heading
More content
### Third Heading
Even more content"#;
        
        toc.update_from_content(content);
        
        // Cursor at line 0 (the first heading) - should select first heading
        toc.sync_with_cursor(0);
        assert_eq!(toc.selected_index(), Some(0));
        
        // Cursor at line 1 (after first heading, before second) - should select first heading
        toc.sync_with_cursor(1);
        assert_eq!(toc.selected_index(), Some(0));
    }

    #[test]
    fn test_sync_with_cursor_selects_current_section() {
        let mut toc = TableOfContents::new();
        let content = r#"# First Heading
Line 1
Line 2
## Second Heading
Line 4
Line 5
### Third Heading
Line 7
Line 8"#;
        
        toc.update_from_content(content);
        
        // Cursor at line 2 (within first section) - should select first heading
        toc.sync_with_cursor(2);
        assert_eq!(toc.selected_index(), Some(0));
        
        // Cursor at line 3 (the second heading itself) - should select second heading
        toc.sync_with_cursor(3);
        assert_eq!(toc.selected_index(), Some(1));
        
        // Cursor at line 5 (within second section) - should select second heading
        toc.sync_with_cursor(5);
        assert_eq!(toc.selected_index(), Some(1));
        
        // Cursor at line 6 (the third heading) - should select third heading
        toc.sync_with_cursor(6);
        assert_eq!(toc.selected_index(), Some(2));
        
        // Cursor at line 8 (within third section) - should select third heading
        toc.sync_with_cursor(8);
        assert_eq!(toc.selected_index(), Some(2));
    }

    #[test]
    fn test_sync_with_cursor_beyond_last_heading() {
        let mut toc = TableOfContents::new();
        let content = r#"# First Heading
## Second Heading
Some content
More content
Even more content"#;
        
        toc.update_from_content(content);
        
        // Cursor at line 10 (way beyond all headings) - should select last heading
        toc.sync_with_cursor(10);
        assert_eq!(toc.selected_index(), Some(1));
        
        // Cursor at line 100 - should still select last heading
        toc.sync_with_cursor(100);
        assert_eq!(toc.selected_index(), Some(1));
    }

    #[test]
    fn test_sync_with_cursor_empty_toc() {
        let mut toc = TableOfContents::new();
        let content = "No headings here\nJust plain text";
        
        toc.update_from_content(content);
        
        // Should have no selection when there are no headings
        toc.sync_with_cursor(0);
        assert_eq!(toc.selected_index(), None);
        
        toc.sync_with_cursor(5);
        assert_eq!(toc.selected_index(), None);
    }

    #[test]
    fn test_sync_preserves_manual_selection() {
        let mut toc = TableOfContents::new();
        let content = r#"# First Heading
Content
## Second Heading
Content
### Third Heading"#;
        
        toc.update_from_content(content);
        
        // Manually select second heading
        toc.next();
        assert_eq!(toc.selected_index(), Some(1));
        
        // Now sync with cursor at line 0 - should change to first heading
        toc.sync_with_cursor(0);
        assert_eq!(toc.selected_index(), Some(0));
    }

    #[test]
    fn test_sync_with_multiple_headings_same_line() {
        // This shouldn't happen in practice, but let's test it
        let mut toc = TableOfContents::new();
        let content = r#"# First Heading
## Second Heading
### Third Heading"#;
        
        toc.update_from_content(content);
        
        // All headings are on consecutive lines
        toc.sync_with_cursor(0);
        assert_eq!(toc.selected_index(), Some(0));
        
        toc.sync_with_cursor(1);
        assert_eq!(toc.selected_index(), Some(1));
        
        toc.sync_with_cursor(2);
        assert_eq!(toc.selected_index(), Some(2));
    }
}
