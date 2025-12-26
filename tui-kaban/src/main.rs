use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::io;
use uuid::Uuid;

const CARD_HEIGHT: u16 = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Card {
    id: Uuid,
    title: String,
    description: String,
}

impl Card {
    fn new(title: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Column {
    id: Uuid,
    name: String,
    cards: Vec<Card>,
}

impl Column {
    fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            cards: Vec::new(),
        }
    }

    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn remove_card(&mut self, card_id: Uuid) -> Option<Card> {
        let pos = self.cards.iter().position(|c| c.id == card_id)?;
        Some(self.cards.remove(pos))
    }
}

enum InputMode {
    Normal,
    CreatingCard,
    EditingCard,
}

struct App {
    columns: Vec<Column>,
    selected_column: usize,
    selected_card: Option<usize>,
    input_mode: InputMode,
    input_title: String,
    input_description: String,
    editing_field: usize, // 0 for title, 1 for description
    should_quit: bool,
    column_areas: Vec<Rect>,
    card_areas: Vec<Vec<Rect>>,
    dragging_card: Option<(usize, usize)>, // (column_idx, card_idx)
    drag_start_pos: Option<(u16, u16)>,
}

impl App {
    fn new() -> Self {
        let mut columns = Vec::new();
        columns.push(Column::new("To Do".to_string()));
        columns.push(Column::new("In Progress".to_string()));
        columns.push(Column::new("Done".to_string()));

        // Add some sample cards
        columns[0].add_card(Card::new(
            "Welcome to TUI Kanban".to_string(),
            "Press 'n' to create a new card, 'e' to edit, 'd' to delete".to_string(),
        ));
        columns[0].add_card(Card::new(
            "Mouse Support".to_string(),
            "Click and drag cards to move them between columns".to_string(),
        ));
        columns[1].add_card(Card::new(
            "Task Example".to_string(),
            "This is a sample task in progress".to_string(),
        ));

        Self {
            columns,
            selected_column: 0,
            selected_card: Some(0),
            input_mode: InputMode::Normal,
            input_title: String::new(),
            input_description: String::new(),
            editing_field: 0,
            should_quit: false,
            column_areas: Vec::new(),
            card_areas: Vec::new(),
            dragging_card: None,
            drag_start_pos: None,
        }
    }

    fn next_column(&mut self) {
        if self.selected_column < self.columns.len() - 1 {
            self.selected_column += 1;
            self.selected_card = if self.columns[self.selected_column].cards.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    fn previous_column(&mut self) {
        if self.selected_column > 0 {
            self.selected_column -= 1;
            self.selected_card = if self.columns[self.selected_column].cards.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    fn next_card(&mut self) {
        if let Some(idx) = self.selected_card {
            let cards_len = self.columns[self.selected_column].cards.len();
            if cards_len > 0 && idx < cards_len - 1 {
                self.selected_card = Some(idx + 1);
            }
        } else if !self.columns[self.selected_column].cards.is_empty() {
            self.selected_card = Some(0);
        }
    }

    fn previous_card(&mut self) {
        if let Some(idx) = self.selected_card {
            if idx > 0 {
                self.selected_card = Some(idx - 1);
            }
        } else if !self.columns[self.selected_column].cards.is_empty() {
            self.selected_card = Some(0);
        }
    }

    fn start_creating_card(&mut self) {
        self.input_mode = InputMode::CreatingCard;
        self.input_title.clear();
        self.input_description.clear();
        self.editing_field = 0;
    }

    fn start_editing_card(&mut self) {
        if let Some(card_idx) = self.selected_card {
            if let Some(card) = self.columns[self.selected_column].cards.get(card_idx) {
                self.input_mode = InputMode::EditingCard;
                self.input_title = card.title.clone();
                self.input_description = card.description.clone();
                self.editing_field = 0;
            }
        }
    }

    fn delete_card(&mut self) {
        if let Some(card_idx) = self.selected_card {
            if card_idx < self.columns[self.selected_column].cards.len() {
                self.columns[self.selected_column].cards.remove(card_idx);
                
                // Adjust selected_card
                if self.columns[self.selected_column].cards.is_empty() {
                    self.selected_card = None;
                } else if card_idx >= self.columns[self.selected_column].cards.len() {
                    self.selected_card = Some(self.columns[self.selected_column].cards.len() - 1);
                }
            }
        }
    }

    fn save_card(&mut self) {
        if self.input_title.trim().is_empty() {
            return;
        }

        match self.input_mode {
            InputMode::CreatingCard => {
                let card = Card::new(self.input_title.clone(), self.input_description.clone());
                self.columns[self.selected_column].add_card(card);
                let new_idx = self.columns[self.selected_column].cards.len() - 1;
                self.selected_card = Some(new_idx);
            }
            InputMode::EditingCard => {
                if let Some(card_idx) = self.selected_card {
                    if let Some(card) = self.columns[self.selected_column].cards.get_mut(card_idx) {
                        card.title = self.input_title.clone();
                        card.description = self.input_description.clone();
                    }
                }
            }
            _ => {}
        }

        self.input_mode = InputMode::Normal;
        self.input_title.clear();
        self.input_description.clear();
    }

    fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_title.clear();
        self.input_description.clear();
    }

    fn move_card_right(&mut self) {
        if let Some(card_idx) = self.selected_card {
            if self.selected_column < self.columns.len() - 1 {
                let card_id = self.columns[self.selected_column].cards[card_idx].id;
                if let Some(card) = self.columns[self.selected_column].remove_card(card_id) {
                    self.columns[self.selected_column + 1].add_card(card);
                    self.selected_column += 1;
                    self.selected_card = Some(self.columns[self.selected_column].cards.len() - 1);
                }
            }
        }
    }

    fn move_card_left(&mut self) {
        if let Some(card_idx) = self.selected_card {
            if self.selected_column > 0 {
                let card_id = self.columns[self.selected_column].cards[card_idx].id;
                if let Some(card) = self.columns[self.selected_column].remove_card(card_id) {
                    self.columns[self.selected_column - 1].add_card(card);
                    self.selected_column -= 1;
                    self.selected_card = Some(self.columns[self.selected_column].cards.len() - 1);
                }
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: event::MouseEvent) {
        let x = mouse_event.column;
        let y = mouse_event.row;

        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check which column and card was clicked
                for (col_idx, col_area) in self.column_areas.iter().enumerate() {
                    if self.is_in_rect(x, y, *col_area) {
                        self.selected_column = col_idx;
                        
                        // Check if a specific card was clicked
                        if col_idx < self.card_areas.len() {
                            let mut found_card = false;
                            for (card_idx, card_area) in self.card_areas[col_idx].iter().enumerate() {
                                if self.is_in_rect(x, y, *card_area) {
                                    self.selected_card = Some(card_idx);
                                    self.dragging_card = Some((col_idx, card_idx));
                                    self.drag_start_pos = Some((x, y));
                                    found_card = true;
                                    break;
                                }
                            }
                            if !found_card {
                                self.selected_card = None;
                                self.dragging_card = None;
                            }
                        }
                        break;
                    }
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging_card.is_some() {
                    // Visual feedback during drag (handled in rendering)
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some((drag_col_idx, drag_card_idx)) = self.dragging_card {
                    // Find the column where mouse was released
                    for (col_idx, col_area) in self.column_areas.iter().enumerate() {
                        if self.is_in_rect(x, y, *col_area) && col_idx != drag_col_idx {
                            // Move card to the new column
                            let card_id = self.columns[drag_col_idx].cards[drag_card_idx].id;
                            if let Some(card) = self.columns[drag_col_idx].remove_card(card_id) {
                                self.columns[col_idx].add_card(card);
                                self.selected_column = col_idx;
                                self.selected_card = Some(self.columns[col_idx].cards.len() - 1);
                            }
                            break;
                        }
                    }
                }
                self.dragging_card = None;
                self.drag_start_pos = None;
            }
            _ => {}
        }
    }

    fn is_in_rect(&self, x: u16, y: u16, rect: Rect) -> bool {
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.previous_column();
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.next_column();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous_card();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_card();
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.start_creating_card();
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    self.start_editing_card();
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    self.delete_card();
                }
                KeyCode::Char('L') => {
                    self.move_card_right();
                }
                KeyCode::Char('H') => {
                    self.move_card_left();
                }
                _ => {}
            },
            InputMode::CreatingCard | InputMode::EditingCard => match key_event.code {
                KeyCode::Esc => {
                    self.cancel_input();
                }
                KeyCode::Enter => {
                    self.save_card();
                }
                KeyCode::Tab => {
                    self.editing_field = 1 - self.editing_field;
                }
                KeyCode::Char(c) => {
                    if self.editing_field == 0 {
                        self.input_title.push(c);
                    } else {
                        self.input_description.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if self.editing_field == 0 {
                        self.input_title.pop();
                    } else {
                        self.input_description.pop();
                    }
                }
                _ => {}
            },
        }
    }
}

fn render_card(area: Rect, buf: &mut Buffer, card: &Card, is_selected: bool, is_dragging: bool) {
    let style = if is_dragging {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .style(style);

    let inner = block.inner(area);
    block.render(area, buf);

    let title = Paragraph::new(card.title.clone())
        .style(Style::default().add_modifier(Modifier::BOLD));
    title.render(
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
        buf,
    );

    if inner.height > 1 {
        let desc = Paragraph::new(card.description.clone())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Gray));
        desc.render(
            Rect {
                x: inner.x,
                y: inner.y + 1,
                width: inner.width,
                height: inner.height.saturating_sub(1),
            },
            buf,
        );
    }
}

fn render_column(
    area: Rect,
    buf: &mut Buffer,
    column: &Column,
    is_selected: bool,
    selected_card: Option<usize>,
    dragging_card: Option<usize>,
) -> Vec<Rect> {
    let style = if is_selected {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ({}) ", column.name, column.cards.len()))
        .style(style);

    let inner = block.inner(area);
    block.render(area, buf);

    let mut card_areas = Vec::new();
    let mut current_y = inner.y;
    
    for (idx, card) in column.cards.iter().enumerate() {
        if current_y >= inner.y + inner.height {
            break;
        }

        let card_height = CARD_HEIGHT.min(inner.height.saturating_sub(current_y - inner.y));
        let card_area = Rect {
            x: inner.x,
            y: current_y,
            width: inner.width,
            height: card_height,
        };

        let is_card_selected = is_selected && selected_card == Some(idx);
        let is_card_dragging = dragging_card == Some(idx);
        render_card(card_area, buf, card, is_card_selected, is_card_dragging);
        
        card_areas.push(card_area);
        current_y += card_height;
    }

    card_areas
}

fn render_input_dialog(area: Rect, buf: &mut Buffer, app: &App, title: &str) {
    // Create a centered dialog
    let dialog_width = area.width.min(60);
    let dialog_height = area.height.min(10);
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(dialog_area);
    block.render(dialog_area, buf);

    // Render title field
    let title_style = if app.editing_field == 0 {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };

    let title_label = Paragraph::new("Title:")
        .style(Style::default().fg(Color::Cyan));
    title_label.render(
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        },
        buf,
    );

    let title_input = Paragraph::new(app.input_title.clone())
        .style(title_style);
    title_input.render(
        Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: 1,
        },
        buf,
    );

    // Render description field
    let desc_style = if app.editing_field == 1 {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };

    let desc_label = Paragraph::new("Description:")
        .style(Style::default().fg(Color::Cyan));
    desc_label.render(
        Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: 1,
        },
        buf,
    );

    let desc_input = Paragraph::new(app.input_description.clone())
        .wrap(Wrap { trim: true })
        .style(desc_style);
    desc_input.render(
        Rect {
            x: inner.x,
            y: inner.y + 4,
            width: inner.width,
            height: inner.height.saturating_sub(6),
        },
        buf,
    );

    // Render help text
    let help = Paragraph::new("Tab: Switch field | Enter: Save | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    help.render(
        Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        },
        buf,
    );
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.area();

            // Create help bar at the bottom
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(1)])
                .split(area);

            // Split main area into columns
            let num_columns = app.columns.len();
            let constraints = vec![Constraint::Ratio(1, num_columns as u32); num_columns];
            
            let columns_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(main_chunks[0]);

            // Store column areas and render columns
            app.column_areas.clear();
            app.card_areas.clear();
            
            for (idx, column) in app.columns.iter().enumerate() {
                app.column_areas.push(columns_layout[idx]);
                
                let is_selected = idx == app.selected_column;
                let selected_card = if is_selected { app.selected_card } else { None };
                let dragging_card = if let Some((drag_col, drag_card)) = app.dragging_card {
                    if drag_col == idx {
                        Some(drag_card)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                let card_areas = render_column(
                    columns_layout[idx],
                    f.buffer_mut(),
                    column,
                    is_selected,
                    selected_card,
                    dragging_card,
                );
                app.card_areas.push(card_areas);
            }

            // Render help bar
            let help_text = match app.input_mode {
                InputMode::Normal => {
                    "q: Quit | n: New Card | e: Edit | d: Delete | h/l: Switch Column | k/j: Select Card | H/L: Move Card"
                }
                _ => "Creating/Editing card...",
            };
            
            let help = Paragraph::new(help_text)
                .style(Style::default().bg(Color::DarkGray).fg(Color::White));
            help.render(main_chunks[1], f.buffer_mut());

            // Render input dialog if in input mode
            match app.input_mode {
                InputMode::CreatingCard => {
                    render_input_dialog(area, f.buffer_mut(), &app, " Create New Card ");
                }
                InputMode::EditingCard => {
                    render_input_dialog(area, f.buffer_mut(), &app, " Edit Card ");
                }
                _ => {}
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
