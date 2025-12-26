use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub parent_id: Option<Uuid>,
}

impl Document {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content: String::new(),
            parent_id: None,
        }
    }

    pub fn with_content(title: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            parent_id: None,
        }
    }

    /// Extract the first heading (starting with #) from the content
    /// If no heading is found, returns None
    pub fn extract_first_heading(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                let title = trimmed.trim_start_matches('#').trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
        None
    }

    /// Update the document title from the first heading in the content
    /// If no heading is found, keeps the current title
    pub fn update_title_from_content(&mut self) {
        if let Some(heading) = Self::extract_first_heading(&self.content) {
            self.title = heading;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_first_heading_simple() {
        let content = "# My Title\n\nSome content here";
        assert_eq!(
            Document::extract_first_heading(content),
            Some("My Title".to_string())
        );
    }

    #[test]
    fn test_extract_first_heading_with_multiple_hashes() {
        let content = "## Second Level Heading\n\nContent";
        assert_eq!(
            Document::extract_first_heading(content),
            Some("Second Level Heading".to_string())
        );
    }

    #[test]
    fn test_extract_first_heading_with_text_before() {
        let content = "Some text\n\n# The Heading\n\nMore content";
        assert_eq!(
            Document::extract_first_heading(content),
            Some("The Heading".to_string())
        );
    }

    #[test]
    fn test_extract_first_heading_no_heading() {
        let content = "Just plain text\nNo headings here";
        assert_eq!(Document::extract_first_heading(content), None);
    }

    #[test]
    fn test_extract_first_heading_empty_heading() {
        let content = "#\nSome content";
        assert_eq!(Document::extract_first_heading(content), None);
    }

    #[test]
    fn test_extract_first_heading_with_spaces() {
        let content = "   #   Heading with spaces   \nContent";
        assert_eq!(
            Document::extract_first_heading(content),
            Some("Heading with spaces".to_string())
        );
    }

    #[test]
    fn test_update_title_from_content() {
        let mut doc = Document::new("Original Title".to_string());
        doc.content = "# New Title from Content\n\nSome text".to_string();
        doc.update_title_from_content();
        assert_eq!(doc.title, "New Title from Content");
    }

    #[test]
    fn test_update_title_from_content_no_heading() {
        let mut doc = Document::new("Original Title".to_string());
        doc.content = "Just some text without a heading".to_string();
        doc.update_title_from_content();
        // Title should remain unchanged
        assert_eq!(doc.title, "Original Title");
    }

    #[test]
    fn test_update_title_from_content_multiple_headings() {
        let mut doc = Document::new("Original Title".to_string());
        doc.content = "# First Heading\n\n## Second Heading\n\nText".to_string();
        doc.update_title_from_content();
        // Should use the first heading only
        assert_eq!(doc.title, "First Heading");
    }
}
