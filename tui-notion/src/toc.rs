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
}
