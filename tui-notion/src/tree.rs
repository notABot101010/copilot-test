use crate::document::Document;
use uuid::Uuid;

pub struct DocumentTree {
    documents: Vec<Document>,
    selected_index: Option<usize>,
}

impl DocumentTree {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            selected_index: None,
        }
    }

    pub fn add_document(&mut self, document: Document) {
        self.documents.push(document);
        if self.selected_index.is_none() {
            self.selected_index = Some(0);
        }
    }

    pub fn delete_document(&mut self, doc_id: Uuid) {
        if let Some(index) = self.documents.iter().position(|d| d.id == doc_id) {
            self.documents.remove(index);
            if self.documents.is_empty() {
                self.selected_index = None;
            } else if let Some(selected) = self.selected_index {
                if selected >= self.documents.len() {
                    self.selected_index = Some(self.documents.len() - 1);
                }
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(index) = self.selected_index {
            if index < self.documents.len() - 1 {
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

    pub fn selected_document(&self) -> Option<Uuid> {
        self.selected_index
            .and_then(|idx| self.documents.get(idx))
            .map(|doc| doc.id)
    }

    pub fn select_document(&mut self, doc_id: Uuid) {
        if let Some(index) = self.documents.iter().position(|d| d.id == doc_id) {
            self.selected_index = Some(index);
        }
    }

    pub fn get_document(&self, doc_id: Uuid) -> Option<&Document> {
        self.documents.iter().find(|d| d.id == doc_id)
    }

    pub fn get_document_mut(&mut self, doc_id: Uuid) -> Option<&mut Document> {
        self.documents.iter_mut().find(|d| d.id == doc_id)
    }

    pub fn documents(&self) -> &[Document] {
        &self.documents
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn search(&self, query: &str) -> Vec<&Document> {
        let query_lower = query.to_lowercase();
        self.documents
            .iter()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&query_lower)
                    || doc.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}
