mod document;
mod editor;
mod search;
mod storage;
mod toc;
mod tree;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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
use tui_input::backend::crossterm::EventHandler;

enum FocusedPanel {
    Editor,
    Toc,
    Search,
}

enum AppMode {
    Normal,
    Insert,
    Search,
    ConfirmDelete,
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
    pending_delete_doc_id: Option<uuid::Uuid>,
    recently_accessed_docs: Vec<uuid::Uuid>,
}

impl App {
    async fn new() -> Result<Self> {
        let storage = Storage::new().await?;
        let mut tree = DocumentTree::new();
        
        // Load existing documents
        let documents = storage.load_all_documents().await?;
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
3. Press `Esc` to return to NORMAL mode (auto-saves)
4. Use `Tab` to cycle between panels

### Keyboard Shortcuts

- **Ctrl+K**: Quick search across all documents
- **Ctrl+N**: Create new document
- **Ctrl+D**: Delete current document (with confirmation)
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
- Documents are auto-saved as you edit!

Happy note-taking!
"#.to_string();
            tree.add_document(welcome_doc.clone());
            storage.save_document(&welcome_doc).await?;
        }

        let editor = Editor::new();
        let toc = TableOfContents::new();
        let search = SearchDialog::new();

        // Load recently accessed documents
        let limit = Storage::default_recently_accessed_limit();
        let recently_accessed_docs = storage
            .get_recently_accessed_documents(limit)
            .await
            .unwrap_or_else(|err| {
                eprintln!("Warning: Failed to load recently accessed documents: {}", err);
                Vec::new()
            });

        let mut app = Self {
            tree,
            editor,
            toc,
            search,
            storage,
            focused_panel: FocusedPanel::Editor,
            mode: AppMode::Normal,
            should_quit: false,
            pending_delete_doc_id: None,
            recently_accessed_docs,
        };

        // Try to load the last opened document
        if let Ok(Some(last_doc_id)) = app.storage.get_last_opened_document().await {
            if app.tree.get_document(last_doc_id).is_some() {
                app.tree.select_document(last_doc_id);
            }
        }

        // Load the first document if available
        if let Some(doc_id) = app.tree.selected_document() {
            if let Some(doc) = app.tree.get_document(doc_id) {
                app.editor.set_content(doc.content.clone());
                app.toc.update_from_content(&doc.content);
                // Record this as an access
                let _ = app.storage.record_document_access(doc_id).await;
                let limit = Storage::default_recently_accessed_limit();
                match app.storage.get_recently_accessed_documents(limit).await {
                    Ok(docs) => app.recently_accessed_docs = docs,
                    Err(err) => eprintln!("Warning: Failed to refresh recently accessed documents: {}", err),
                }
            }
        }

        Ok(app)
    }

    async fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode(key).await?,
            AppMode::Insert => self.handle_insert_mode(key).await?,
            AppMode::Search => self.handle_search_mode(key).await?,
            AppMode::ConfirmDelete => self.handle_confirm_delete_mode(key).await?,
        }
        Ok(())
    }

    async fn handle_normal_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        // Global shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('k') => {
                    self.mode = AppMode::Search;
                    self.focused_panel = FocusedPanel::Search;
                    self.search.reset();
                    self.search.update_results_with_recent(&self.tree, &self.recently_accessed_docs);
                    return Ok(());
                }
                KeyCode::Char('n') => {
                    // Create new document
                    let new_doc = Document::new("New Document".to_string());
                    let doc_id = new_doc.id;
                    self.storage.save_document(&new_doc).await?;
                    self.tree.add_document(new_doc);
                    self.tree.select_document(doc_id);
                    self.load_selected_document().await?;
                    return Ok(());
                }
                KeyCode::Char('d') => {
                    // Show delete confirmation dialog
                    if let Some(doc_id) = self.tree.selected_document() {
                        self.pending_delete_doc_id = Some(doc_id);
                        self.mode = AppMode::ConfirmDelete;
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
                    FocusedPanel::Editor => self.handle_editor_navigation(key)?,
                    FocusedPanel::Toc => self.handle_toc_navigation(key).await?,
                    FocusedPanel::Search => {}
                }
            }
        }
        Ok(())
    }

    async fn handle_insert_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.auto_save_current_document().await?;
            }
            KeyCode::Char(c) => {
                self.editor.insert_char(c);
                self.update_toc();
                self.auto_save_current_document().await?;
            }
            KeyCode::Backspace => {
                self.editor.delete_char();
                self.update_toc();
                self.auto_save_current_document().await?;
            }
            KeyCode::Enter => {
                self.editor.insert_newline();
                self.update_toc();
                self.auto_save_current_document().await?;
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

    async fn handle_search_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.focused_panel = FocusedPanel::Editor;
            }
            KeyCode::Enter => {
                if let Some(doc_id) = self.search.selected_document() {
                    self.tree.select_document(doc_id);
                    self.load_selected_document().await?;
                    self.mode = AppMode::Normal;
                    self.focused_panel = FocusedPanel::Editor;
                }
            }
            KeyCode::Down => {
                self.search.next_result();
            }
            KeyCode::Up => {
                self.search.previous_result();
            }
            _ => {
                // Handle input using tui-input
                use crossterm::event::Event;
                let input_event = Event::Key(key);
                self.search.input_mut().handle_event(&input_event);
                self.search.update_results_with_recent(&self.tree, &self.recently_accessed_docs);
            }
        }
        Ok(())
    }

    async fn handle_confirm_delete_mode(&mut self, key: event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Confirm deletion
                if let Some(doc_id) = self.pending_delete_doc_id {
                    self.tree.delete_document(doc_id);
                    self.storage.delete_document(doc_id).await?;
                    self.editor.clear();
                    self.toc.clear();
                    
                    // Load the next available document
                    if self.tree.selected_document().is_some() {
                        self.load_selected_document().await?;
                    }
                }
                self.pending_delete_doc_id = None;
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                // Cancel deletion
                self.pending_delete_doc_id = None;
                self.mode = AppMode::Normal;
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

    async fn handle_toc_navigation(&mut self, key: event::KeyEvent) -> Result<()> {
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
            FocusedPanel::Editor => FocusedPanel::Toc,
            FocusedPanel::Toc => FocusedPanel::Editor,
            FocusedPanel::Search => FocusedPanel::Editor,
        };
    }

    async fn load_selected_document(&mut self) -> Result<()> {
        if let Some(doc_id) = self.tree.selected_document() {
            if let Some(doc) = self.tree.get_document(doc_id) {
                self.editor.set_content(doc.content.clone());
                self.update_toc();
                // Save the last opened document
                self.storage.set_last_opened_document(doc_id).await?;
                // Record document access
                self.storage.record_document_access(doc_id).await?;
                // Update recently accessed documents list
                self.refresh_recently_accessed_docs().await?;
            }
        }
        Ok(())
    }

    async fn auto_save_current_document(&mut self) -> Result<()> {
        if let Some(doc_id) = self.tree.selected_document() {
            let content = self.editor.get_content();
            if let Some(doc) = self.tree.get_document_mut(doc_id) {
                doc.content = content;
                self.storage.save_document(doc).await?;
            }
        }
        Ok(())
    }

    fn update_toc(&mut self) {
        let content = self.editor.get_content();
        self.toc.update_from_content(&content);
    }

    async fn refresh_recently_accessed_docs(&mut self) -> Result<()> {
        let limit = Storage::default_recently_accessed_limit();
        match self.storage.get_recently_accessed_documents(limit).await {
            Ok(docs) => {
                self.recently_accessed_docs = docs;
            }
            Err(err) => {
                // Log error but don't fail - fall back to empty list
                eprintln!("Warning: Failed to refresh recently accessed documents: {}", err);
                self.recently_accessed_docs.clear();
            }
        }
        Ok(())
    }
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        // Update viewport height before rendering
        let terminal_height = terminal.size()?.height;
        let editor_height = (terminal_height as f32 * 0.8) as u16;
        let inner_height = editor_height.saturating_sub(2) as usize;
        app.editor.set_viewport_height(inner_height);
        
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(80),
                ])
                .split(f.area());

            // Render table of contents (left panel - moved from right)
            let toc_focused = matches!(app.focused_panel, FocusedPanel::Toc);
            ui::render_toc(chunks[0], f.buffer_mut(), &app.toc, toc_focused);

            // Render editor (right panel - now takes more space)
            let editor_focused = matches!(app.focused_panel, FocusedPanel::Editor);
            let editor_mode = match app.mode {
                AppMode::Insert => "INSERT",
                AppMode::Normal => "NORMAL",
                AppMode::Search => "SEARCH",
                AppMode::ConfirmDelete => "CONFIRM DELETE",
            };
            ui::render_editor(
                chunks[1],
                f.buffer_mut(),
                &app.editor,
                editor_focused,
                editor_mode,
            );

            // Render search dialog if in search mode
            if matches!(app.mode, AppMode::Search) {
                ui::render_search_dialog(f.area(), f.buffer_mut(), &app.search, &app.tree);
            }

            // Render confirmation dialog if in confirm delete mode
            if matches!(app.mode, AppMode::ConfirmDelete) {
                ui::render_confirm_dialog(f.area(), f.buffer_mut(), "Delete this document? (y/n)");
            }
        })?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            app.handle_key_event(key).await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = &res {
        eprintln!("Error: {:?}", err);
    }

    res
}
