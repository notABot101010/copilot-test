use crate::tree::DocumentTree;
use uuid::Uuid;

pub struct SearchDialog {
    query: String,
    results: Vec<Uuid>,
    selected_index: Option<usize>,
}

impl SearchDialog {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: None,
        }
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected_index = None;
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn delete_char(&mut self) {
        self.query.pop();
    }

    pub fn update_results(&mut self, tree: &DocumentTree) {
        self.results.clear();
        
        if self.query.is_empty() {
            self.selected_index = None;
            return;
        }

        let matches = tree.search(&self.query);
        self.results = matches.iter().map(|doc| doc.id).collect();
        
        if !self.results.is_empty() {
            self.selected_index = Some(0);
        } else {
            self.selected_index = None;
        }
    }

    pub fn next_result(&mut self) {
        if let Some(index) = self.selected_index {
            if index < self.results.len() - 1 {
                self.selected_index = Some(index + 1);
            }
        }
    }

    pub fn previous_result(&mut self) {
        if let Some(index) = self.selected_index {
            if index > 0 {
                self.selected_index = Some(index - 1);
            }
        }
    }

    pub fn selected_document(&self) -> Option<Uuid> {
        self.selected_index
            .and_then(|idx| self.results.get(idx))
            .copied()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn results(&self) -> &[Uuid] {
        &self.results
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }
}
