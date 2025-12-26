mod document;
mod editor;
mod search;
mod storage;
mod toc;
mod tree;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use document::Document;
use editor::Editor;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use search::SearchDialog;
use std::io;
use storage::Storage;
use toc::TableOfContents;
use tree::DocumentTree;

enum FocusedPanel {
    Tree,
    Editor,
    Toc,
    Search,
}

enum AppMode {
    Normal,
    Insert,
    Search,
}

struct App {
    tree: DocumentTree,
    editor: Editor,
    toc: TableOfContents,
    search: SearchDialog,
    storage: Storage,
    focused_panel: FocusedPanel,
    mode: AppMode,
    should_quit: bool,
}

impl App {
    fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let mut tree = DocumentTree::new();
        
        // Load existing documents
        let documents = storage.load_all_documents()?;
        for doc in documents {
            tree.add_document(doc);
        }
        
        // If no documents exist, create a welcome document
        if tree.is_empty() {
            let mut welcome_doc = Document::new("Welcome to TUI Notion".to_string());
            welcome_doc.content = r#"# Welcome to TUI Notion

A terminal-based Notion clone built with Rust!

## Features

- **Three-Panel Layout**: Navigate documents, edit content, view outline
- **Markdown Support**: Full markdown editing with syntax highlighting
- **Live Table of Contents**: Auto-generated from your headings
- **Keyboard Navigation**: Vi-style keybindings (j/k) and arrow keys

## Quick Start

1. Press `i` to enter INSERT mode
2. Type your markdown content
3. Press `Esc` to save and return to NORMAL mode
4. Use `Tab` to cycle between panels

### Keyboard Shortcuts

- **Ctrl+K**: Quick search across all documents
- **Ctrl+N**: Create new document
- **Ctrl+S**: Save current document
- **Ctrl+D**: Delete current document
- **q**: Quit application

### Navigation

- **Arrow keys** or **j/k**: Navigate lists and scroll
- **Enter**: Open document or jump to heading
- **Tab**: Switch between panels

## Try It Out

Create headings with `#` symbols and watch them appear in the outline panel on the right!

### Markdown Syntax

The editor highlights:
- `# Headings` at different levels
- ``` Code blocks ```
- `- Lists` and bullet points

### Document Management

- Create multiple documents with Ctrl+N
- Switch between them using the tree on the left
- Use Ctrl+K for quick navigation

Happy note-taking!
"#.to_string();
            tree.add_document(welcome_doc);
        }

        let editor = Editor::new();
        let toc = TableOfContents::new();
        let search = SearchDialog::new();

        let mut app = Self {
            tree,
            editor,
            toc,
            search,
            storage,
            focused_panel: FocusedPanel::Tree,
            mode: AppMode::Normal,
            should_quit: false,
        };

        // Load the first document if available
        if let Some(doc_id) = app.tree.selected_document() {
            if let Some(doc) = app.tree.get_document(doc_id) {
                app.editor.set_content(doc.content.clone());
                app.toc.update_from_content(&doc.content);
            }
        }

        Ok(app)
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode(key)?,
            AppMode::Insert => self.handle_insert_mode(key)?,
            AppMode::Search => self.handle_search_mode(key)?,
        }
        Ok(())
    }

    fn handle_normal_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        // Global shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('k') => {
                    self.mode = AppMode::Search;
                    self.focused_panel = FocusedPanel::Search;
                    self.search.reset();
                    return Ok(());
                }
                KeyCode::Char('n') => {
                    // Create new document
                    let new_doc = Document::new("New Document".to_string());
                    let doc_id = new_doc.id;
                    self.tree.add_document(new_doc);
                    self.tree.select_document(doc_id);
                    self.load_selected_document()?;
                    return Ok(());
                }
                KeyCode::Char('s') => {
                    // Save current document
                    self.save_current_document()?;
                    return Ok(());
                }
                KeyCode::Char('d') => {
                    // Delete current document
                    if let Some(doc_id) = self.tree.selected_document() {
                        self.tree.delete_document(doc_id);
                        self.storage.delete_document(doc_id)?;
                        self.editor.clear();
                        self.toc.clear();
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.cycle_focus();
            }
            KeyCode::Char('i') => {
                if matches!(self.focused_panel, FocusedPanel::Editor) {
                    self.mode = AppMode::Insert;
                }
            }
            _ => {
                match self.focused_panel {
                    FocusedPanel::Tree => self.handle_tree_navigation(key)?,
                    FocusedPanel::Editor => self.handle_editor_navigation(key)?,
                    FocusedPanel::Toc => self.handle_toc_navigation(key)?,
                    FocusedPanel::Search => {}
                }
            }
        }
        Ok(())
    }

    fn handle_insert_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.save_current_document()?;
            }
            KeyCode::Char(c) => {
                self.editor.insert_char(c);
                self.update_toc();
            }
            KeyCode::Backspace => {
                self.editor.delete_char();
                self.update_toc();
            }
            KeyCode::Enter => {
                self.editor.insert_newline();
                self.update_toc();
            }
            KeyCode::Left => {
                self.editor.move_cursor_left();
            }
            KeyCode::Right => {
                self.editor.move_cursor_right();
            }
            KeyCode::Up => {
                self.editor.move_cursor_up();
            }
            KeyCode::Down => {
                self.editor.move_cursor_down();
            }
            KeyCode::Home => {
                self.editor.move_cursor_to_line_start();
            }
            KeyCode::End => {
                self.editor.move_cursor_to_line_end();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.focused_panel = FocusedPanel::Tree;
            }
            KeyCode::Enter => {
                if let Some(doc_id) = self.search.selected_document() {
                    self.tree.select_document(doc_id);
                    self.load_selected_document()?;
                    self.mode = AppMode::Normal;
                    self.focused_panel = FocusedPanel::Editor;
                }
            }
            KeyCode::Char(c) => {
                self.search.add_char(c);
                self.search.update_results(&self.tree);
            }
            KeyCode::Backspace => {
                self.search.delete_char();
                self.search.update_results(&self.tree);
            }
            KeyCode::Down => {
                self.search.next_result();
            }
            KeyCode::Up => {
                self.search.previous_result();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_tree_navigation(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.tree.next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.tree.previous();
            }
            KeyCode::Enter => {
                self.load_selected_document()?;
                self.focused_panel = FocusedPanel::Editor;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_editor_navigation(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.editor.scroll_down();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.editor.scroll_up();
            }
            KeyCode::PageDown => {
                self.editor.page_down();
            }
            KeyCode::PageUp => {
                self.editor.page_up();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_toc_navigation(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.toc.next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.toc.previous();
            }
            KeyCode::Enter => {
                if let Some(line) = self.toc.selected_line() {
                    self.editor.jump_to_line(line);
                    self.focused_panel = FocusedPanel::Editor;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Tree => FocusedPanel::Editor,
            FocusedPanel::Editor => FocusedPanel::Toc,
            FocusedPanel::Toc => FocusedPanel::Tree,
            FocusedPanel::Search => FocusedPanel::Tree,
        };
    }

    fn load_selected_document(&mut self) -> Result<()> {
        if let Some(doc_id) = self.tree.selected_document() {
            if let Some(doc) = self.tree.get_document(doc_id) {
                self.editor.set_content(doc.content.clone());
                self.update_toc();
            }
        }
        Ok(())
    }

    fn save_current_document(&mut self) -> Result<()> {
        if let Some(doc_id) = self.tree.selected_document() {
            let content = self.editor.get_content();
            if let Some(doc) = self.tree.get_document_mut(doc_id) {
                doc.content = content;
                self.storage.save_document(doc)?;
            }
        }
        Ok(())
    }

    fn update_toc(&mut self) {
        let content = self.editor.get_content();
        self.toc.update_from_content(&content);
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(f.area());

            // Render document tree (left panel)
            let tree_focused = matches!(app.focused_panel, FocusedPanel::Tree);
            ui::render_tree(chunks[0], f.buffer_mut(), &app.tree, tree_focused);

            // Render editor (center panel)
            let editor_focused = matches!(app.focused_panel, FocusedPanel::Editor);
            let editor_mode = match app.mode {
                AppMode::Insert => "INSERT",
                AppMode::Normal => "NORMAL",
                AppMode::Search => "SEARCH",
            };
            ui::render_editor(
                chunks[1],
                f.buffer_mut(),
                &app.editor,
                editor_focused,
                editor_mode,
            );

            // Render table of contents (right panel)
            let toc_focused = matches!(app.focused_panel, FocusedPanel::Toc);
            ui::render_toc(chunks[2], f.buffer_mut(), &app.toc, toc_focused);

            // Render search dialog if in search mode
            if matches!(app.mode, AppMode::Search) {
                ui::render_search_dialog(f.area(), f.buffer_mut(), &app.search, &app.tree);
            }
        })?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            app.handle_key_event(key)?;
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new()?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = &res {
        eprintln!("Error: {:?}", err);
    }

    res
}
