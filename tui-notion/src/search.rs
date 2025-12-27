use crate::tree::DocumentTree;
use tui_input::Input;
use uuid::Uuid;

pub struct SearchDialog {
    input: Input,
    results: Vec<Uuid>,
    selected_index: Option<usize>,
}

impl SearchDialog {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            results: Vec::new(),
            selected_index: None,
        }
    }

    pub fn reset(&mut self) {
        self.input.reset();
        self.results.clear();
        self.selected_index = None;
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn update_results(&mut self, tree: &DocumentTree) {
        self.update_results_with_recent(tree, &[]);
    }

    pub fn update_results_with_recent(&mut self, tree: &DocumentTree, recent_doc_ids: &[Uuid]) {
        self.results.clear();

        let query = self.input.value();

        if query.is_empty() {
            // Show recently accessed documents when query is empty
            // Only include documents that still exist in the tree
            self.results = recent_doc_ids
                .iter()
                .filter(|id| tree.get_document(**id).is_some())
                .copied()
                .collect();

            // If no recently accessed documents, show all documents
            if self.results.is_empty() {
                self.results = tree.documents().iter().map(|doc| doc.id).collect();
            }
        } else {
            // Search for matching documents
            let matches = tree.search(query);
            self.results = matches.iter().map(|doc| doc.id).collect();
        }

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
        self.input.value()
    }

    pub fn results(&self) -> &[Uuid] {
        &self.results
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }
}
