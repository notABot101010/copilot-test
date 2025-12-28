mod models;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::{MindMap, Node, NodeColor};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use ui::{Canvas, DocumentDialog, SearchBox};
use uuid::Uuid;

const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(500);
const ZOOM_STEP: f64 = 0.1;
const MIN_ZOOM: f64 = 0.5;
const MAX_ZOOM: f64 = 3.0;

#[derive(Debug, Clone)]
struct TextInput {
    value: String,
    cursor: usize,
}

impl TextInput {
    fn new(value: String) -> Self {
        let cursor = value.chars().count();
        Self { value, cursor }
    }

    fn default() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
        }
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn cursor(&self) -> usize {
        self.cursor
    }

    fn reset(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    fn insert(&mut self, c: char) {
        let byte_idx = self.value
            .char_indices()
            .nth(self.cursor)
            .map(|(idx, _)| idx)
            .unwrap_or(self.value.len());
        self.value.insert(byte_idx, c);
        self.cursor += 1;
    }

    fn delete_before_cursor(&mut self) {
        if self.cursor > 0 {
            let byte_idx = self.value
                .char_indices()
                .nth(self.cursor - 1)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            self.value.remove(byte_idx);
            self.cursor -= 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor < self.value.chars().count() {
            self.cursor += 1;
        }
    }

    fn move_cursor_to_start(&mut self) {
        self.cursor = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.cursor = self.value.chars().count();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    ViewingDocument,
    EditingDocument,
    Searching,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    mindmap: MindMap,
}

struct App {
    mindmap: MindMap,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    selected_node: Option<Uuid>,
    mode: Mode,
    should_quit: bool,
    dragging_node: Option<Uuid>,
    dragging_canvas: bool,
    drag_start_x: f64,
    drag_start_y: f64,
    connecting_from: Option<Uuid>,
    last_click_time: Option<Instant>,
    last_click_node: Option<Uuid>,
    edit_title: TextInput,
    edit_body: TextInput,
    editing_title: bool,
    history: Vec<HistoryEntry>,
    history_index: usize,
    search_query: String,
    search_results: Vec<Uuid>,
}

impl App {
    fn new() -> Self {
        let mut mindmap = MindMap::new();
        
        // Add some initial nodes
        mindmap.add_node(Node::new("Welcome to MindMap!".to_string(), 40.0, 10.0));
        mindmap.add_node(Node::new("Getting Started".to_string(), 20.0, 20.0));
        mindmap.add_node(Node::new("Features".to_string(), 60.0, 20.0));
        mindmap.add_node(Node::new("Press N for new node".to_string(), 10.0, 35.0));
        mindmap.add_node(Node::new("Click & drag to move".to_string(), 40.0, 35.0));
        mindmap.add_node(Node::new("Press C to connect".to_string(), 70.0, 35.0));

        // Add some connections
        if mindmap.nodes.len() >= 6 {
            mindmap.add_connection(mindmap.nodes[0].id, mindmap.nodes[1].id);
            mindmap.add_connection(mindmap.nodes[0].id, mindmap.nodes[2].id);
            mindmap.add_connection(mindmap.nodes[1].id, mindmap.nodes[3].id);
            mindmap.add_connection(mindmap.nodes[1].id, mindmap.nodes[4].id);
            mindmap.add_connection(mindmap.nodes[2].id, mindmap.nodes[5].id);
        }

        let history_entry = HistoryEntry { mindmap: mindmap.clone() };

        Self {
            mindmap,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            selected_node: None,
            mode: Mode::Normal,
            should_quit: false,
            dragging_node: None,
            dragging_canvas: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            connecting_from: None,
            last_click_time: None,
            last_click_node: None,
            edit_title: TextInput::default(),
            edit_body: TextInput::default(),
            editing_title: true,
            history: vec![history_entry],
            history_index: 0,
            search_query: String::new(),
            search_results: Vec::new(),
        }
    }

    fn save_to_history(&mut self) {
        // Remove any redo history
        self.history.truncate(self.history_index + 1);
        
        // Add new history entry
        let entry = HistoryEntry {
            mindmap: self.mindmap.clone(),
        };
        self.history.push(entry);
        self.history_index += 1;

        // Limit history size
        if self.history.len() > 50 {
            self.history.remove(0);
            self.history_index = self.history_index.saturating_sub(1);
        }
    }

    fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.mindmap = self.history[self.history_index].mindmap.clone();
            self.selected_node = None;
            self.connecting_from = None;
        }
    }

    fn redo(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.mindmap = self.history[self.history_index].mindmap.clone();
            self.selected_node = None;
            self.connecting_from = None;
        }
    }

    fn zoom_in(&mut self) {
        self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM);
    }

    fn zoom_out(&mut self) {
        self.zoom = (self.zoom - ZOOM_STEP).max(MIN_ZOOM);
    }

    fn screen_to_world(&self, screen_x: u16, screen_y: u16) -> (f64, f64) {
        let world_x = screen_x as f64 / self.zoom + self.pan_x;
        let world_y = screen_y as f64 / self.zoom + self.pan_y;
        (world_x, world_y)
    }

    fn create_new_node(&mut self, x: f64, y: f64) {
        let node = Node::new("New Node".to_string(), x, y);
        self.mindmap.add_node(node);
        self.save_to_history();
    }

    fn delete_selected_node(&mut self) {
        if let Some(node_id) = self.selected_node {
            self.mindmap.remove_node(node_id);
            self.selected_node = None;
            self.save_to_history();
        }
    }

    fn disconnect_from_selected(&mut self) {
        if let Some(from_id) = self.selected_node {
            // Remove all connections from the selected node
            self.mindmap.connections.retain(|c| c.from != from_id && c.to != from_id);
            self.save_to_history();
        }
    }

    fn cycle_node_color(&mut self) {
        if let Some(node_id) = self.selected_node {
            if let Some(node) = self.mindmap.get_node_by_id_mut(node_id) {
                node.color = match node.color {
                    NodeColor::Default => NodeColor::Red,
                    NodeColor::Red => NodeColor::Green,
                    NodeColor::Green => NodeColor::Blue,
                    NodeColor::Blue => NodeColor::Yellow,
                    NodeColor::Yellow => NodeColor::Magenta,
                    NodeColor::Magenta => NodeColor::Cyan,
                    NodeColor::Cyan => NodeColor::Default,
                };
                self.save_to_history();
            }
        }
    }

    fn start_connecting(&mut self) {
        if let Some(node_id) = self.selected_node {
            self.connecting_from = Some(node_id);
        }
    }

    fn finish_connecting(&mut self, target_id: Uuid) {
        if let Some(from_id) = self.connecting_from {
            if from_id != target_id {
                self.mindmap.add_connection(from_id, target_id);
                self.save_to_history();
            }
            self.connecting_from = None;
        }
    }

    fn cancel_connecting(&mut self) {
        self.connecting_from = None;
    }

    fn open_document(&mut self, node_id: Uuid) {
        if let Some(node) = self.mindmap.get_node_by_id(node_id) {
            self.edit_title = TextInput::new(node.document.title.clone());
            self.edit_body = TextInput::new(node.document.body.clone());
            self.editing_title = true;
            self.mode = Mode::ViewingDocument;
        }
    }

    fn start_editing(&mut self) {
        if self.mode == Mode::ViewingDocument {
            self.mode = Mode::EditingDocument;
        }
    }

    fn save_document(&mut self) {
        if let Some(node_id) = self.selected_node {
            if let Some(node) = self.mindmap.get_node_by_id_mut(node_id) {
                node.document.title = self.edit_title.value().to_string();
                node.document.body = self.edit_body.value().to_string();
                self.save_to_history();
            }
        }
        self.mode = Mode::Normal;
        self.edit_title.reset();
        self.edit_body.reset();
    }

    fn cancel_editing(&mut self) {
        self.mode = Mode::Normal;
        self.edit_title.reset();
        self.edit_body.reset();
    }

    fn start_search(&mut self) {
        self.mode = Mode::Searching;
        self.search_query.clear();
        self.search_results.clear();
    }

    fn perform_search(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();
        
        if query.is_empty() {
            return;
        }

        for node in &self.mindmap.nodes {
            if node.document.title.to_lowercase().contains(&query) 
                || node.document.body.to_lowercase().contains(&query) {
                self.search_results.push(node.id);
            }
        }

        // If there are results, select the first one
        if !self.search_results.is_empty() {
            self.selected_node = Some(self.search_results[0]);
            // Pan to the selected node
            if let Some(node) = self.mindmap.get_node_by_id(self.search_results[0]) {
                self.pan_x = node.x - 50.0;
                self.pan_y = node.y - 25.0;
            }
        }
    }

    fn next_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let current_idx = self.selected_node
            .and_then(|id| self.search_results.iter().position(|&rid| rid == id))
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % self.search_results.len();
        self.selected_node = Some(self.search_results[next_idx]);

        // Pan to the selected node
        if let Some(node) = self.mindmap.get_node_by_id(self.search_results[next_idx]) {
            self.pan_x = node.x - 50.0;
            self.pan_y = node.y - 25.0;
        }
    }

    fn cancel_search(&mut self) {
        self.mode = Mode::Normal;
        self.search_query.clear();
        self.search_results.clear();
    }

    fn save_to_file(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.mindmap)?;
        std::fs::write("mindmap.json", json)?;
        Ok(())
    }

    fn load_from_file(&mut self) -> Result<()> {
        let json = std::fs::read_to_string("mindmap.json")?;
        self.mindmap = serde_json::from_str(&json)?;
        self.save_to_history();
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse_event: event::MouseEvent) {
        let (world_x, world_y) = self.screen_to_world(mouse_event.column, mouse_event.row);

        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.connecting_from.is_some() {
                    // In connecting mode, finish the connection
                    if let Some(node_idx) = self.mindmap.find_node_at(world_x, world_y) {
                        let target_id = self.mindmap.nodes[node_idx].id;
                        self.finish_connecting(target_id);
                    }
                } else {
                    // Normal mode - check for node selection
                    if let Some(node_idx) = self.mindmap.find_node_at(world_x, world_y) {
                        let node_id = self.mindmap.nodes[node_idx].id;
                        self.selected_node = Some(node_id);

                        // Check for double-click
                        let now = Instant::now();
                        if let (Some(last_time), Some(last_node)) = (self.last_click_time, self.last_click_node) {
                            if last_node == node_id && now.duration_since(last_time) < DOUBLE_CLICK_THRESHOLD {
                                self.open_document(node_id);
                                self.last_click_time = None;
                                self.last_click_node = None;
                                return;
                            }
                        }

                        self.last_click_time = Some(now);
                        self.last_click_node = Some(node_id);

                        // Start dragging
                        self.dragging_node = Some(node_id);
                        self.drag_start_x = world_x;
                        self.drag_start_y = world_y;
                    } else {
                        // Clicked outside any node - start canvas drag
                        self.selected_node = None;
                        self.last_click_time = None;
                        self.last_click_node = None;
                        self.dragging_canvas = true;
                        self.drag_start_x = mouse_event.column as f64;
                        self.drag_start_y = mouse_event.row as f64;
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging_node = None;
                self.dragging_canvas = false;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(node_id) = self.dragging_node {
                    if let Some(node) = self.mindmap.get_node_by_id_mut(node_id) {
                        let dx = world_x - self.drag_start_x;
                        let dy = world_y - self.drag_start_y;
                        node.x += dx;
                        node.y += dy;
                        self.drag_start_x = world_x;
                        self.drag_start_y = world_y;
                    }
                } else if self.dragging_canvas {
                    // Drag canvas - update pan based on screen coordinates
                    let dx = (mouse_event.column as f64 - self.drag_start_x) / self.zoom;
                    let dy = (mouse_event.row as f64 - self.drag_start_y) / self.zoom;
                    self.pan_x -= dx;
                    self.pan_y -= dy;
                    self.drag_start_x = mouse_event.column as f64;
                    self.drag_start_y = mouse_event.row as f64;
                }
            }
            MouseEventKind::ScrollUp => {
                self.zoom_in();
            }
            MouseEventKind::ScrollDown => {
                self.zoom_out();
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match self.mode {
            Mode::Normal => {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        self.should_quit = true;
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        if self.selected_node.is_none() {
                            self.zoom_in();
                        }
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        if self.selected_node.is_none() {
                            self.zoom_out();
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        // Create new node at center of screen
                        let center_x = 50.0 + self.pan_x;
                        let center_y = 25.0 + self.pan_y;
                        self.create_new_node(center_x, center_y);
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        self.delete_selected_node();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        self.start_connecting();
                    }
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        self.disconnect_from_selected();
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.cycle_node_color();
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        self.start_search();
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        if let Err(err) = self.save_to_file() {
                            eprintln!("Error saving: {:?}", err);
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        if let Err(err) = self.load_from_file() {
                            eprintln!("Error loading: {:?}", err);
                        }
                    }
                    KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.undo();
                    }
                    KeyCode::Char('y') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.redo();
                    }
                    KeyCode::Esc => {
                        if self.connecting_from.is_some() {
                            self.cancel_connecting();
                        } else {
                            self.selected_node = None;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(node_id) = self.selected_node {
                            self.open_document(node_id);
                        }
                    }
                    KeyCode::Left => {
                        self.pan_x -= 5.0;
                    }
                    KeyCode::Right => {
                        self.pan_x += 5.0;
                    }
                    KeyCode::Up => {
                        self.pan_y -= 5.0;
                    }
                    KeyCode::Down => {
                        self.pan_y += 5.0;
                    }
                    _ => {}
                }
            }
            Mode::ViewingDocument => {
                match key_event.code {
                    KeyCode::Enter => {
                        self.start_editing();
                    }
                    KeyCode::Esc => {
                        self.cancel_editing();
                    }
                    _ => {}
                }
            }
            Mode::EditingDocument => {
                match key_event.code {
                    KeyCode::Enter => {
                        self.save_document();
                    }
                    KeyCode::Esc => {
                        self.cancel_editing();
                    }
                    KeyCode::Tab => {
                        self.editing_title = !self.editing_title;
                    }
                    KeyCode::Char(c) => {
                        if self.editing_title {
                            self.edit_title.insert(c);
                        } else {
                            self.edit_body.insert(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if self.editing_title {
                            self.edit_title.delete_before_cursor();
                        } else {
                            self.edit_body.delete_before_cursor();
                        }
                    }
                    KeyCode::Left => {
                        if self.editing_title {
                            self.edit_title.move_cursor_left();
                        } else {
                            self.edit_body.move_cursor_left();
                        }
                    }
                    KeyCode::Right => {
                        if self.editing_title {
                            self.edit_title.move_cursor_right();
                        } else {
                            self.edit_body.move_cursor_right();
                        }
                    }
                    KeyCode::Home => {
                        if self.editing_title {
                            self.edit_title.move_cursor_to_start();
                        } else {
                            self.edit_body.move_cursor_to_start();
                        }
                    }
                    KeyCode::End => {
                        if self.editing_title {
                            self.edit_title.move_cursor_to_end();
                        } else {
                            self.edit_body.move_cursor_to_end();
                        }
                    }
                    _ => {}
                }
            }
            Mode::Searching => {
                match key_event.code {
                    KeyCode::Enter => {
                        self.perform_search();
                        self.cancel_search();
                    }
                    KeyCode::Char(c) => {
                        self.search_query.push(c);
                    }
                    KeyCode::Backspace => {
                        self.search_query.pop();
                    }
                    KeyCode::Down => {
                        self.next_search_result();
                    }
                    KeyCode::Esc => {
                        self.cancel_search();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.area();

            // Render canvas
            let canvas = Canvas::new(&app.mindmap, app.zoom, app.pan_x, app.pan_y)
                .selected(app.selected_node)
                .connecting(app.connecting_from);

            f.render_widget(canvas, area);

            // Render document dialog if in viewing/editing mode
            if app.mode == Mode::ViewingDocument || app.mode == Mode::EditingDocument {
                let dialog = DocumentDialog {
                    title_value: app.edit_title.value(),
                    title_cursor: app.edit_title.cursor(),
                    body_value: app.edit_body.value(),
                    body_cursor: app.edit_body.cursor(),
                    editing: app.mode == Mode::EditingDocument,
                    editing_title: app.editing_title,
                };
                f.render_widget(dialog, area);
            }

            // Render search box if in searching mode
            if app.mode == Mode::Searching {
                let search_box = SearchBox {
                    query: &app.search_query,
                    results_count: app.search_results.len(),
                };
                f.render_widget(search_box, area);
            }
        })?;

        if app.should_quit {
            return Ok(());
        }

        match event::read()? {
            Event::Key(key) => app.handle_key_event(key),
            Event::Mouse(mouse) => app.handle_mouse_event(mouse),
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}
