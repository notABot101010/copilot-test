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
    edit_buffer: String,
    scroll_row: usize,
    scroll_col: usize,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            cells: CellMap::new(),
            cursor_row: 0,
            cursor_col: 0,
            mode: Mode::View,
            edit_buffer: String::new(),
            scroll_row: 0,
            scroll_col: 0,
            should_quit: false,
        }
    }

    fn move_cursor(&mut self, row_delta: i32, col_delta: i32) {
        let new_row = (self.cursor_row as i32 + row_delta).clamp(0, ROWS as i32 - 1) as usize;
        let new_col = (self.cursor_col as i32 + col_delta).clamp(0, COLS as i32 - 1) as usize;
        
        self.cursor_row = new_row;
        self.cursor_col = new_col;
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
        self.edit_buffer = self.cells.get(&key).cloned().unwrap_or_default();
    }

    fn start_formula(&mut self) {
        self.mode = Mode::Edit;
        self.edit_buffer = "=".to_string();
    }

    fn save_edit(&mut self) {
        let key = get_cell_key(self.cursor_row, self.cursor_col);
        if self.edit_buffer.is_empty() {
            self.cells.remove(&key);
        } else {
            self.cells.insert(key, self.edit_buffer.clone());
        }
        self.mode = Mode::View;
        self.edit_buffer.clear();
    }

    fn cancel_edit(&mut self) {
        self.mode = Mode::View;
        self.edit_buffer.clear();
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
                _ => {}
            }
            return;
        }

        match key_code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Up => self.move_cursor(-1, 0),
            KeyCode::Down => self.move_cursor(1, 0),
            KeyCode::Left => self.move_cursor(0, -1),
            KeyCode::Right => self.move_cursor(0, 1),
            KeyCode::Char('=') => self.start_formula(),
            KeyCode::Char('e') | KeyCode::Enter => self.enter_edit_mode(),
            KeyCode::Delete | KeyCode::Backspace => {
                let key = get_cell_key(self.cursor_row, self.cursor_col);
                self.cells.remove(&key);
            }
            _ => {}
        }
    }

    fn handle_edit_mode_key(&mut self, key_code: KeyCode, _modifiers: KeyModifiers) {
        match key_code {
            KeyCode::Enter => {
                self.save_edit();
            }
            KeyCode::Esc => {
                self.cancel_edit();
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.edit_buffer.push(c);
            }
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

    // Create app
    let mut app = App::new();

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

    // Render top bar
    render_top_bar(f, app, chunks[0]);

    // Render grid
    render_grid(f, app, chunks[1], visible_rows, visible_cols);

    // Render status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let cell_ref = format_cell_reference(app.cursor_row, app.cursor_col);
    
    let display_value = if app.mode == Mode::Edit {
        &app.edit_buffer
    } else {
        let key = get_cell_key(app.cursor_row, app.cursor_col);
        app.cells.get(&key).map(|s| s.as_str()).unwrap_or("")
    };

    let text = format!("{} | fx: {}", cell_ref, display_value);
    
    let style = if app.mode == Mode::Edit {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(
            if app.mode == Mode::Edit {
                "Edit Mode (Enter=Save, Esc=Cancel)"
            } else {
                "View Mode (e/==Edit, Arrow keys=Navigate, q=Quit)"
            }
        ));

    f.render_widget(paragraph, area);
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
                app.edit_buffer.clone()
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
    
    let status = format!(
        " {} | Cell: {} | Functions: =SUM(A1:A10), =AVERAGE(A1:A10), =MIN, =MAX, =COUNT ",
        mode_text,
        format_cell_reference(app.cursor_row, app.cursor_col)
    );

    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}
