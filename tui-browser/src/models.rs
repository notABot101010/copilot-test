use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Link {
    pub text: String,
    pub url: String,
    pub line_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub content: String,
    pub loading: bool,
    pub scroll_offset: usize,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "New Tab".to_string(),
            url: String::new(),
            content: String::new(),
            loading: false,
            scroll_offset: 0,
        }
    }

    #[allow(dead_code)]
    pub fn with_url(url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Loading...".to_string(),
            url,
            content: String::new(),
            loading: true,
            scroll_offset: 0,
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty() && self.url.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub added_at: DateTime<Utc>,
}

impl Bookmark {
    pub fn new(title: String, url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            url,
            added_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub url: String,
    pub title: String,
    pub visited_at: DateTime<Utc>,
}

impl HistoryEntry {
    pub fn new(url: String, title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            title,
            visited_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    entries: Vec<HistoryEntry>,
    current_index: Option<usize>,
}

impl NavigationHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
        }
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) {
        // Remove any forward history when navigating to a new page
        if let Some(idx) = self.current_index {
            self.entries.truncate(idx + 1);
        }
        
        self.entries.push(entry);
        self.current_index = Some(self.entries.len() - 1);
    }

    pub fn can_go_back(&self) -> bool {
        self.current_index.map_or(false, |idx| idx > 0)
    }

    pub fn can_go_forward(&self) -> bool {
        self.current_index.map_or(false, |idx| idx < self.entries.len() - 1)
    }

    pub fn go_back(&mut self) -> Option<&HistoryEntry> {
        if let Some(idx) = self.current_index {
            if idx > 0 {
                self.current_index = Some(idx - 1);
                return self.entries.get(idx - 1);
            }
        }
        None
    }

    pub fn go_forward(&mut self) -> Option<&HistoryEntry> {
        if let Some(idx) = self.current_index {
            if idx < self.entries.len() - 1 {
                self.current_index = Some(idx + 1);
                return self.entries.get(idx + 1);
            }
        }
        None
    }
}
