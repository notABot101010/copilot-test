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
}
