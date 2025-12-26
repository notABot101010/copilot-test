/// History module for undo/redo functionality
/// Stores editor state snapshots for in-memory undo/redo operations

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

pub struct History {
    undo_stack: VecDeque<EditorState>,
    redo_stack: VecDeque<EditorState>,
    max_history_size: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history_size: 100, // Keep last 100 states
        }
    }

    /// Push a new state onto the undo stack
    /// This clears the redo stack as new changes invalidate redo history
    pub fn push(&mut self, state: EditorState) {
        self.undo_stack.push_back(state);
        
        // Limit history size to prevent unbounded memory growth
        if self.undo_stack.len() > self.max_history_size {
            self.undo_stack.pop_front();
        }
        
        // Clear redo stack when new changes are made
        self.redo_stack.clear();
    }

    /// Undo the last change
    /// Returns the previous state if available
    pub fn undo(&mut self, current_state: EditorState) -> Option<EditorState> {
        if let Some(previous_state) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(current_state);
            Some(previous_state)
        } else {
            None
        }
    }

    /// Redo a previously undone change
    /// Returns the next state if available
    pub fn redo(&mut self, current_state: EditorState) -> Option<EditorState> {
        if let Some(next_state) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(current_state);
            Some(next_state)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_state(lines: Vec<&str>, cursor_line: usize, cursor_col: usize) -> EditorState {
        EditorState {
            lines: lines.into_iter().map(|s| s.to_string()).collect(),
            cursor_line,
            cursor_col,
        }
    }

    #[test]
    fn test_push_and_undo() {
        let mut history = History::new();
        
        let state1 = create_state(vec!["hello"], 0, 5);
        let state2 = create_state(vec!["hello", "world"], 1, 5);
        
        history.push(state1.clone());
        history.push(state2.clone());
        
        let state3 = create_state(vec!["hello", "world", "!"], 2, 1);
        
        // Undo should return state2
        let undone = history.undo(state3.clone());
        assert!(undone.is_some());
        let undone = undone.unwrap();
        assert_eq!(undone.lines, vec!["hello", "world"]);
        assert_eq!(undone.cursor_line, 1);
        assert_eq!(undone.cursor_col, 5);
    }

    #[test]
    fn test_redo() {
        let mut history = History::new();
        
        let state1 = create_state(vec!["hello"], 0, 5);
        let state2 = create_state(vec!["hello", "world"], 1, 5);
        
        history.push(state1.clone());
        
        // Undo
        let undone = history.undo(state2.clone());
        assert!(undone.is_some());
        
        // Redo should return state2
        let redone = history.redo(state1.clone());
        assert!(redone.is_some());
        let redone = redone.unwrap();
        assert_eq!(redone.lines, vec!["hello", "world"]);
        assert_eq!(redone.cursor_line, 1);
        assert_eq!(redone.cursor_col, 5);
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let mut history = History::new();
        
        let state1 = create_state(vec!["a"], 0, 1);
        let state2 = create_state(vec!["ab"], 0, 2);
        let state3 = create_state(vec!["abc"], 0, 3);
        
        history.push(state1.clone());
        history.push(state2.clone());
        
        // Undo once
        history.undo(state3.clone());
        assert!(history.can_redo());
        
        // Push new state should clear redo
        let state4 = create_state(vec!["abcd"], 0, 4);
        history.push(state4);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_can_undo_can_redo() {
        let mut history = History::new();
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        
        let state1 = create_state(vec!["test"], 0, 4);
        history.push(state1);
        
        assert!(history.can_undo());
        assert!(!history.can_redo());
        
        let state2 = create_state(vec!["test2"], 0, 5);
        history.undo(state2);
        
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_max_history_size() {
        let mut history = History::new();
        history.max_history_size = 5;
        
        // Push 10 states
        for i in 0..10 {
            let state = create_state(vec![&format!("line{}", i)], 0, i);
            history.push(state);
        }
        
        // Should only keep last 5
        assert_eq!(history.undo_stack.len(), 5);
        
        // The oldest should be line5 (line0-4 were removed)
        assert_eq!(history.undo_stack[0].lines[0], "line5");
    }

    #[test]
    fn test_clear() {
        let mut history = History::new();
        
        let state1 = create_state(vec!["a"], 0, 1);
        let state2 = create_state(vec!["ab"], 0, 2);
        
        history.push(state1.clone());
        history.undo(state2);
        
        assert!(history.can_redo());
        
        history.clear();
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
