mod formula;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use formula::{
    get_cell_key, get_computed_value, format_cell_reference, index_to_column, CellMap,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Cell, Row, Table},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

const ROWS: usize = 100;
const COLS: usize = 26; // A-Z
const CELL_WIDTH: u16 = 12;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    View,
    Edit,
}

struct App {
    cells: CellMap,
    cursor_row: usize,
    cursor_col: usize,
    mode: Mode,
    input: Input,
    scroll_row: usize,
    scroll_col: usize,
    should_quit: bool,
    numeric_multiplier: String,
    undo_stack: Vec<CellMap>,
    redo_stack: Vec<CellMap>,
    csv_file: Option<PathBuf>,
}

impl App {
    fn new(csv_file: Option<PathBuf>) -> Self {
        Self {
            cells: CellMap::new(),
            cursor_row: 0,
            cursor_col: 0,
            mode: Mode::View,
            input: Input::default(),
            scroll_row: 0,
            scroll_col: 0,
            should_quit: false,
            numeric_multiplier: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            csv_file,
        }
    }

    fn move_cursor(&mut self, row_delta: i32, col_delta: i32) {
        // Apply numeric multiplier if present
        let multiplier = if !self.numeric_multiplier.is_empty() {
            // Parse multiplier, clamping to reasonable values
            self.numeric_multiplier.parse::<i32>().unwrap_or(1).clamp(1, 1000)
        } else {
            1
        };

        let new_row = (self.cursor_row as i32 + row_delta * multiplier).clamp(0, ROWS as i32 - 1) as usize;
        let new_col = (self.cursor_col as i32 + col_delta * multiplier).clamp(0, COLS as i32 - 1) as usize;

        self.cursor_row = new_row;
        self.cursor_col = new_col;

        // Clear the numeric multiplier after use
        self.numeric_multiplier.clear();
    }

    fn adjust_scroll(&mut self, visible_rows: usize, visible_cols: usize) {
        // Adjust vertical scroll
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row - visible_rows + 1;
        }

        // Adjust horizontal scroll
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor_col - visible_cols + 1;
        }
    }

    fn enter_edit_mode(&mut self) {
        self.mode = Mode::Edit;
        let key = get_cell_key(self.cursor_row, self.cursor_col);
        let cell_value = self.cells.get(&key).cloned().unwrap_or_default();
        self.input = Input::new(cell_value);
        // Clear numeric multiplier when entering edit mode
        self.numeric_multiplier.clear();
    }

    fn start_formula(&mut self) {
        self.mode = Mode::Edit;
        self.input = Input::new("=".to_string());
        // Clear numeric multiplier when entering edit mode
        self.numeric_multiplier.clear();
    }

    fn save_edit(&mut self) {
        // Save current state to undo stack before making changes
        self.push_undo_state();
        
        let key = get_cell_key(self.cursor_row, self.cursor_col);
        let value = self.input.value().to_string();
        if value.is_empty() {
            self.cells.remove(&key);
        } else {
            self.cells.insert(key, value);
        }
        self.mode = Mode::View;
        self.input.reset();
    }

    fn cancel_edit(&mut self) {
        self.mode = Mode::View;
        self.input.reset();
    }

    fn push_undo_state(&mut self) {
        self.undo_stack.push(self.cells.clone());
        // Clear redo stack when a new action is performed
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev_state) = self.undo_stack.pop() {
            self.redo_stack.push(self.cells.clone());
            self.cells = prev_state;
        }
    }

    fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(self.cells.clone());
            self.cells = next_state;
        }
    }

    fn load_csv(&mut self) -> Result<()> {
        if let Some(path) = &self.csv_file {
            if path.exists() {
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .flexible(true)  // Allow variable number of fields per record
                    .from_path(path)?;

                for (row_idx, result) in reader.records().enumerate() {
                    if row_idx >= ROWS {
                        break;
                    }
                    let record = result?;
                    for (col_idx, field) in record.iter().enumerate() {
                        if col_idx >= COLS {
                            break;
                        }
                        if !field.is_empty() {
                            let key = get_cell_key(row_idx, col_idx);
                            self.cells.insert(key, field.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn save_csv(&self) -> Result<()> {
        if let Some(path) = &self.csv_file {
            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .flexible(true)  // Allow variable number of fields per record
                .from_path(path)?;

            let mut has_any_data = false;
            
            for row_idx in 0..ROWS {
                let mut row_data = Vec::new();
                let mut last_non_empty_col = None;
                
                // Collect row data and track last non-empty cell
                for col_idx in 0..COLS {
                    let key = get_cell_key(row_idx, col_idx);
                    let value = self.cells.get(&key).map(|s| s.as_str()).unwrap_or("");
                    row_data.push(value);
                    if !value.is_empty() {
                        last_non_empty_col = Some(col_idx);
                    }
                }
                
                // Only write rows that have at least one non-empty cell
                if let Some(last_col) = last_non_empty_col {
                    // Write only up to the last non-empty cell
                    writer.write_record(&row_data[..=last_col])?;
                    has_any_data = true;
                }
            }
            
            // If no data was written at all, write an empty row to create a valid CSV file
            if !has_any_data {
                writer.write_record(&[""])?;
            }
            
            writer.flush()?;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            Mode::View => self.handle_view_mode_key(key_code, modifiers),
            Mode::Edit => self.handle_edit_mode_key(key_code, modifiers),
        }
    }

    fn handle_view_mode_key(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        if modifiers.contains(KeyModifiers::CONTROL) {
            match key_code {
                KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('z') => {
                    self.undo();
                }
                KeyCode::Char('y') => {
                    self.redo();
                }
                KeyCode::Char('s') => {
                    let _ = self.save_csv();
                }
                _ => {}
            }
            return;
        }

        match key_code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Up => {
                self.move_cursor(-1, 0);
            }
            KeyCode::Down => {
                self.move_cursor(1, 0);
            }
            KeyCode::Left => {
                self.move_cursor(0, -1);
            }
            KeyCode::Right => {
                self.move_cursor(0, 1);
            }
            KeyCode::Char('=') => {
                self.start_formula();
            }
            KeyCode::Char('e') | KeyCode::Char('i') | KeyCode::Enter => {
                self.enter_edit_mode();
            }
            KeyCode::Delete | KeyCode::Backspace => {
                self.push_undo_state();
                let key = get_cell_key(self.cursor_row, self.cursor_col);
                self.cells.remove(&key);
                // Clear numeric multiplier on delete
                self.numeric_multiplier.clear();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                // Accumulate numeric multiplier (limit to 4 digits to prevent overflow)
                if self.numeric_multiplier.len() < 4 {
                    self.numeric_multiplier.push(c);
                }
            }
            _ => {
                // Clear numeric multiplier on any other key
                self.numeric_multiplier.clear();
            }
        }
    }

    fn handle_edit_mode_key(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        match key_code {
            KeyCode::Enter => {
                self.save_edit();
            }
            KeyCode::Esc => {
                self.cancel_edit();
            }
            _ => {
                // Use tui-input's event handler for all other keys
                // This handles cursor navigation, character input, deletion, etc.
                self.input.handle_event(&Event::Key(event::KeyEvent::new(key_code, modifiers)));
            }
        }
    }
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let csv_file = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and load CSV if provided
    let mut app = App::new(csv_file.clone());
    if csv_file.is_some() {
        let _ = app.load_csv();
    }

    // Run app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key_event(key.code, key.modifiers);
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top bar with formula input
            Constraint::Min(0),    // Grid
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Calculate visible dimensions
    let grid_height = chunks[1].height.saturating_sub(2) as usize; // Account for borders
    let grid_width = chunks[1].width.saturating_sub(4) as usize; // Account for borders and row numbers
    let visible_rows = grid_height;
    let visible_cols = (grid_width / CELL_WIDTH as usize).min(COLS);

    // Adjust scroll position
    app.adjust_scroll(visible_rows, visible_cols);

    // Render top bar and get cursor position if in edit mode
    let cursor_pos = render_top_bar(f, app, chunks[0]);
    
    // Set cursor position if in edit mode
    if let Some((x, y)) = cursor_pos {
        f.set_cursor_position((x, y));
    }

    // Render grid
    render_grid(f, app, chunks[1], visible_rows, visible_cols);

    // Render status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_top_bar(f: &mut Frame, app: &App, area: Rect) -> Option<(u16, u16)> {
    let cell_ref = format_cell_reference(app.cursor_row, app.cursor_col);

    let multiplier_text = if !app.numeric_multiplier.is_empty() {
        format!(" [{}x]", app.numeric_multiplier)
    } else {
        String::new()
    };

    let block = Block::default().borders(Borders::ALL).title(
        if app.mode == Mode::Edit {
            "Edit Mode (Enter=Save, Esc=Cancel, Arrows=Move Cursor)"
        } else {
            "View Mode (e/==Edit, Ctrl+Z=Undo, Ctrl+Y=Redo, Ctrl+S=Save, q=Quit)"
        }
    );

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render the prefix and input value
    let prefix = format!("{}{} | fx: ", cell_ref, multiplier_text);
    
    if app.mode == Mode::Edit {
        // In edit mode, render with cursor position calculation
        let input_value = app.input.value();
        let text = format!("{}{}", prefix, input_value);
        
        let style = Style::default().fg(Color::Yellow);
        
        // Calculate scroll for the input portion
        // Note: prefix is always short (cell ref + multiplier + " | fx: "), so casting is safe
        let prefix_len_u16 = prefix.len() as u16;
        let available_width = if inner.width > prefix_len_u16 {
            (inner.width - prefix_len_u16) as usize
        } else {
            0
        };
        let scroll = if available_width > 0 {
            app.input.visual_scroll(available_width)
        } else {
            0
        };
        
        let paragraph = Paragraph::new(text)
            .style(style)
            .scroll((0, scroll as u16));
        
        f.render_widget(paragraph, inner);
        
        // Calculate cursor position
        let cursor_offset = app.input.visual_cursor();
        
        // The cursor position accounts for scroll affecting the entire text (prefix + input)
        // When scroll > 0, both prefix and input are shifted left
        let cursor_x = inner.x + (prefix_len_u16 + cursor_offset as u16).saturating_sub(scroll as u16);
        let cursor_y = inner.y;
        
        // Make sure cursor is within bounds (only need to check x-coordinate)
        if cursor_x >= inner.x && cursor_x < inner.x + inner.width {
            return Some((cursor_x, cursor_y));
        }
    } else {
        // In view mode, just render the text normally
        let key = get_cell_key(app.cursor_row, app.cursor_col);
        let display_value = app.cells.get(&key).map(|s| s.as_str()).unwrap_or("");
        let text = format!("{}{}", prefix, display_value);
        
        let style = Style::default().fg(Color::White);
        let paragraph = Paragraph::new(text).style(style);
        
        f.render_widget(paragraph, inner);
    }
    
    None
}

fn render_grid(f: &mut Frame, app: &App, area: Rect, visible_rows: usize, visible_cols: usize) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Spreadsheet");

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create header row with column labels
    let mut header_cells = vec![Cell::from("")]; // Empty cell for row number column
    for col in app.scroll_col..(app.scroll_col + visible_cols).min(COLS) {
        header_cells.push(
            Cell::from(index_to_column(col))
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        );
    }
    let header = Row::new(header_cells).height(1);

    // Create data rows
    let mut rows = vec![header];

    for row_idx in app.scroll_row..(app.scroll_row + visible_rows).min(ROWS) {
        let mut row_cells = vec![
            Cell::from((row_idx + 1).to_string())
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ];

        for col_idx in app.scroll_col..(app.scroll_col + visible_cols).min(COLS) {
            let is_cursor = row_idx == app.cursor_row && col_idx == app.cursor_col;

            let display_value = if is_cursor && app.mode == Mode::Edit {
                app.input.value().to_string()
            } else {
                get_computed_value(row_idx, col_idx, &app.cells)
            };

            let style = if is_cursor {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if display_value.starts_with('=') {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            row_cells.push(Cell::from(display_value).style(style));
        }

        rows.push(Row::new(row_cells).height(1));
    }

    // Create column widths
    let mut widths = vec![Constraint::Length(5)]; // Row number column
    for _ in 0..visible_cols {
        widths.push(Constraint::Length(CELL_WIDTH));
    }

    let table = Table::new(rows, widths)
        .style(Style::default().fg(Color::White));

    f.render_widget(table, inner);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode_text = match app.mode {
        Mode::View => "VIEW",
        Mode::Edit => "EDIT",
    };

    let file_info = if let Some(path) = &app.csv_file {
        format!(" | File: {}", path.display())
    } else {
        String::new()
    };

    let status = format!(
        " {} | Cell: {}{} | Functions: =SUM(A1:A10), =AVERAGE(A1:A10), =MIN, =MAX, =COUNT ",
        mode_text,
        format_cell_reference(app.cursor_row, app.cursor_col),
        file_info
    );

    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_multiplier_basic() {
        let mut app = App::new(None);

        // Set up initial position
        app.cursor_row = 5;
        app.cursor_col = 5;

        // Type numeric multiplier
        app.numeric_multiplier = "3".to_string();

        // Move down with multiplier
        app.move_cursor(1, 0);

        assert_eq!(app.cursor_row, 8); // 5 + 3*1 = 8
        assert_eq!(app.numeric_multiplier, ""); // Should be cleared
    }

    #[test]
    fn test_numeric_multiplier_large() {
        let mut app = App::new(None);

        // Set up initial position
        app.cursor_row = 10;
        app.cursor_col = 0;

        // Type large numeric multiplier
        app.numeric_multiplier = "50".to_string();

        // Move down with multiplier
        app.move_cursor(1, 0);

        assert_eq!(app.cursor_row, 60); // 10 + 50*1 = 60
        assert_eq!(app.numeric_multiplier, ""); // Should be cleared
    }

    #[test]
    fn test_numeric_multiplier_horizontal() {
        let mut app = App::new(None);

        // Set up initial position
        app.cursor_row = 0;
        app.cursor_col = 2;

        // Type numeric multiplier
        app.numeric_multiplier = "10".to_string();

        // Move right with multiplier
        app.move_cursor(0, 1);

        assert_eq!(app.cursor_col, 12); // 2 + 10*1 = 12
        assert_eq!(app.numeric_multiplier, ""); // Should be cleared
    }

    #[test]
    fn test_numeric_multiplier_boundary() {
        let mut app = App::new(None);

        // Set up initial position at bottom
        app.cursor_row = 95;
        app.cursor_col = 20;

        // Type numeric multiplier that would exceed bounds
        app.numeric_multiplier = "100".to_string();

        // Move down with multiplier - should clamp to max
        app.move_cursor(1, 0);

        assert_eq!(app.cursor_row, ROWS - 1); // Should be clamped to max
        assert_eq!(app.numeric_multiplier, ""); // Should be cleared
    }

    #[test]
    fn test_no_multiplier() {
        let mut app = App::new(None);

        // Set up initial position
        app.cursor_row = 5;
        app.cursor_col = 5;

        // Move without multiplier
        app.move_cursor(1, 0);

        assert_eq!(app.cursor_row, 6); // 5 + 1*1 = 6
    }

    #[test]
    fn test_input_widget_integration() {
        let mut app = App::new(None);

        // Enter edit mode
        app.enter_edit_mode();

        assert_eq!(app.mode, Mode::Edit);
        assert_eq!(app.input.value(), "");

        // Set a value in input
        app.input = Input::new("test value".to_string());

        // Save edit
        app.save_edit();

        assert_eq!(app.mode, Mode::View);
        let key = get_cell_key(0, 0);
        assert_eq!(app.cells.get(&key).unwrap(), "test value");
    }

    #[test]
    fn test_start_formula_clears_multiplier() {
        let mut app = App::new(None);

        // Set numeric multiplier
        app.numeric_multiplier = "123".to_string();

        // Start formula
        app.start_formula();

        assert_eq!(app.mode, Mode::Edit);
        assert_eq!(app.input.value(), "=");
        assert_eq!(app.numeric_multiplier, ""); // Should be cleared
    }

    #[test]
    fn test_enter_edit_mode_clears_multiplier() {
        let mut app = App::new(None);

        // Set numeric multiplier
        app.numeric_multiplier = "456".to_string();

        // Enter edit mode
        app.enter_edit_mode();

        assert_eq!(app.mode, Mode::Edit);
        assert_eq!(app.numeric_multiplier, ""); // Should be cleared
    }

    #[test]
    fn test_numeric_multiplier_max_value() {
        let mut app = App::new(None);

        // Set up initial position
        app.cursor_row = 0;
        app.cursor_col = 0;

        // Type a very large numeric multiplier (should be clamped to 1000)
        app.numeric_multiplier = "5000".to_string();

        // Move down with multiplier
        app.move_cursor(1, 0);

        // Should move by max 1000 cells, clamped to ROWS-1
        assert_eq!(app.cursor_row, ROWS - 1);
    }
    
    #[test]
    fn test_cursor_position_in_edit_mode() {
        let mut app = App::new(None);
        
        // Enter edit mode
        app.enter_edit_mode();
        assert_eq!(app.mode, Mode::Edit);
        
        // Type some text
        app.input = Input::new("Hello".to_string());
        
        // The cursor should be at the end of the text
        let cursor_pos = app.input.visual_cursor();
        assert_eq!(cursor_pos, 5); // "Hello" has 5 characters
        
        // Test that visual_scroll returns 0 for short text
        let scroll = app.input.visual_scroll(50);
        assert_eq!(scroll, 0);
    }
    
    #[test]
    fn test_cursor_position_with_long_text() {
        let mut app = App::new(None);
        
        // Enter edit mode properly
        app.enter_edit_mode();
        
        // Set long text
        let long_text = "This is a very long text that will require scrolling";
        app.input = Input::new(long_text.to_string());
        
        // The cursor should be at the end
        let cursor_pos = app.input.visual_cursor();
        assert_eq!(cursor_pos, long_text.len());
        
        // With a narrow width, it should scroll
        let scroll = app.input.visual_scroll(20);
        assert!(scroll > 0);
        
        // The visible cursor position should be within the width
        let visible_cursor = cursor_pos.saturating_sub(scroll);
        assert!(visible_cursor <= 20);
    }

    #[test]
    fn test_undo_redo() {
        let mut app = App::new(None);
        
        // Initially, undo and redo should do nothing
        app.undo();
        assert_eq!(app.cells.len(), 0);
        app.redo();
        assert_eq!(app.cells.len(), 0);
        
        // Save initial empty state
        app.push_undo_state();
        
        // Add a value
        let key1 = get_cell_key(0, 0);
        app.cells.insert(key1.clone(), "10".to_string());
        app.push_undo_state();
        
        // Add another value
        let key2 = get_cell_key(1, 1);
        app.cells.insert(key2.clone(), "20".to_string());
        
        // Current state: cells = [key1, key2]
        // Undo stack: [empty, [key1]]
        // Redo stack: []
        
        // Undo should restore previous state (with just key1)
        app.undo();
        // Now: cells = [key1], undo_stack = [empty], redo_stack = [[key1, key2]]
        assert_eq!(app.cells.len(), 1);
        assert_eq!(app.cells.get(&key1).unwrap(), "10");
        assert!(!app.cells.contains_key(&key2));
        
        // Undo again should restore initial empty state
        app.undo();
        // Now: cells = empty, undo_stack = [], redo_stack = [[key1, key2], [key1]]
        assert_eq!(app.cells.len(), 0);
        
        // Redo should restore [key1]
        app.redo();
        // Now: cells = [key1], undo_stack = [empty], redo_stack = [[key1, key2]]
        assert_eq!(app.cells.len(), 1);
        assert_eq!(app.cells.get(&key1).unwrap(), "10");
        assert!(!app.cells.contains_key(&key2));
        
        // Redo again should restore [key1, key2]
        app.redo();
        // Now: cells = [key1, key2], undo_stack = [empty, [key1]], redo_stack = []
        assert_eq!(app.cells.len(), 2);
        assert_eq!(app.cells.get(&key1).unwrap(), "10");
        assert_eq!(app.cells.get(&key2).unwrap(), "20");
    }

    #[test]
    fn test_undo_clears_redo_stack() {
        let mut app = App::new(None);
        
        // Add a value
        let key1 = get_cell_key(0, 0);
        app.cells.insert(key1.clone(), "10".to_string());
        app.push_undo_state();
        
        // Add another value
        let key2 = get_cell_key(1, 1);
        app.cells.insert(key2.clone(), "20".to_string());
        app.push_undo_state();
        
        // Undo once
        app.undo();
        assert_eq!(app.redo_stack.len(), 1);
        
        // Make a new change - should clear redo stack
        let key3 = get_cell_key(2, 2);
        app.cells.insert(key3, "30".to_string());
        app.push_undo_state();
        
        assert_eq!(app.redo_stack.len(), 0);
    }

    #[test]
    fn test_csv_save_load() {
        use std::env;
        use std::fs;
        
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("test_spreadsheet.csv");
        
        // Clean up if file exists
        let _ = fs::remove_file(&temp_file);
        
        // Create app with CSV file
        let mut app = App::new(Some(temp_file.clone()));
        
        // Add some data
        app.cells.insert(get_cell_key(0, 0), "10".to_string());
        app.cells.insert(get_cell_key(0, 1), "20".to_string());
        app.cells.insert(get_cell_key(1, 0), "30".to_string());
        app.cells.insert(get_cell_key(1, 1), "=A1+B1".to_string());
        
        // Save CSV
        let result = app.save_csv();
        assert!(result.is_ok());
        assert!(temp_file.exists());
        
        // Create a new app and load the CSV
        let mut app2 = App::new(Some(temp_file.clone()));
        let load_result = app2.load_csv();
        assert!(load_result.is_ok());
        
        // Verify data was loaded correctly
        assert_eq!(app2.cells.get(&get_cell_key(0, 0)).unwrap(), "10");
        assert_eq!(app2.cells.get(&get_cell_key(0, 1)).unwrap(), "20");
        assert_eq!(app2.cells.get(&get_cell_key(1, 0)).unwrap(), "30");
        assert_eq!(app2.cells.get(&get_cell_key(1, 1)).unwrap(), "=A1+B1");
        
        // Clean up
        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_csv_save_sparse_rows() {
        use std::env;
        use std::fs;
        
        let temp_dir = env::temp_dir();
        let temp_file = temp_dir.join("test_sparse.csv");
        
        // Clean up if file exists
        let _ = fs::remove_file(&temp_file);
        
        // Create app with sparse data (empty first cell, data in middle)
        let mut app = App::new(Some(temp_file.clone()));
        app.cells.insert(get_cell_key(0, 1), "B1".to_string()); // Empty A1, data in B1
        app.cells.insert(get_cell_key(1, 0), "A2".to_string()); // Data in A2
        app.cells.insert(get_cell_key(2, 2), "C3".to_string()); // Empty A3, B3, data in C3
        
        // Save CSV
        let result = app.save_csv();
        if let Err(err) = &result {
            eprintln!("Save error: {}", err);
        }
        assert!(result.is_ok());
        
        // Verify the file was created
        assert!(temp_file.exists());
        
        // Read the CSV file content
        let content = fs::read_to_string(&temp_file).unwrap();
        eprintln!("CSV content:\n{}", content);
        
        // Load it back
        let mut app2 = App::new(Some(temp_file.clone()));
        let load_result = app2.load_csv();
        if let Err(err) = &load_result {
            eprintln!("Load error: {}", err);
        }
        assert!(load_result.is_ok());
        
        // Verify sparse data was preserved correctly
        // Note: CSV format doesn't distinguish between empty string and missing cell,
        // so we check that the non-empty cells are loaded correctly
        assert_eq!(app2.cells.get(&get_cell_key(0, 1)).unwrap(), "B1");
        assert_eq!(app2.cells.get(&get_cell_key(1, 0)).unwrap(), "A2");
        assert_eq!(app2.cells.get(&get_cell_key(2, 2)).unwrap(), "C3");
        
        // Clean up
        let _ = fs::remove_file(&temp_file);
    }
}
