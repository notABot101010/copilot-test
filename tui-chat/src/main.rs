mod mock_data;
mod ui;

use anyhow::Result;
use chrono::Utc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mock_data::{generate_mock_data, Conversation, Message};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};
use std::io;
use ui::{ConversationList, InputBox, MessageView};
use uuid::Uuid;

enum InputMode {
    Normal,
    Editing,
}

struct App {
    conversations: Vec<Conversation>,
    selected_conversation: Option<usize>,
    input: String,
    input_mode: InputMode,
    should_quit: bool,
    message_scroll_offset: usize,
    conversation_list_area: Rect,
    message_view_area: Rect,
    input_box_area: Rect,
}

impl App {
    fn new() -> Self {
        Self {
            conversations: generate_mock_data(),
            selected_conversation: None,
            input: String::new(),
            input_mode: InputMode::Normal,
            should_quit: false,
            message_scroll_offset: 0,
            conversation_list_area: Rect::default(),
            message_view_area: Rect::default(),
            input_box_area: Rect::default(),
        }
    }

    fn next_conversation(&mut self) {
        if self.conversations.is_empty() {
            return;
        }

        self.selected_conversation = if let Some(i) = self.selected_conversation {
            if i >= self.conversations.len() - 1 {
                Some(0)
            } else {
                Some(i + 1)
            }
        } else {
            Some(0)
        };
        self.message_scroll_offset = 0;
    }

    fn previous_conversation(&mut self) {
        if self.conversations.is_empty() {
            return;
        }

        self.selected_conversation = if let Some(i) = self.selected_conversation {
            if i == 0 {
                Some(self.conversations.len() - 1)
            } else {
                Some(i - 1)
            }
        } else {
            Some(0)
        };
        self.message_scroll_offset = 0;
    }

    fn select_current_conversation(&mut self) {
        if let Some(idx) = self.selected_conversation {
            if idx < self.conversations.len() {
                self.conversations[idx].mark_as_read();
            }
        }
    }

    fn deselect_conversation(&mut self) {
        self.selected_conversation = None;
        self.input.clear();
        self.input_mode = InputMode::Normal;
        self.message_scroll_offset = 0;
    }

    fn send_message(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        if let Some(idx) = self.selected_conversation {
            if idx < self.conversations.len() {
                let message = Message {
                    id: Uuid::new_v4(),
                    sender: "You".to_string(),
                    content: self.input.clone(),
                    timestamp: Utc::now(),
                    is_own: true,
                };
                self.conversations[idx].add_message(message);
                self.conversations[idx].mark_as_read();
                self.input.clear();
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: event::MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let x = mouse_event.column;
                let y = mouse_event.row;

                // Check if click is in conversation list area
                if self.is_in_rect(x, y, self.conversation_list_area) {
                    self.handle_conversation_list_click(y);
                }
                // Check if click is in input box area
                else if self.is_in_rect(x, y, self.input_box_area) {
                    if self.selected_conversation.is_some() {
                        self.input_mode = InputMode::Editing;
                        self.select_current_conversation();
                    }
                }
                // Check if click is in message view area
                else if self.is_in_rect(x, y, self.message_view_area) {
                    // Focus on message view (could add additional functionality here)
                }
            }
            MouseEventKind::ScrollUp => {
                if self.is_in_rect(mouse_event.column, mouse_event.row, self.message_view_area) {
                    self.message_scroll_offset = self.message_scroll_offset.saturating_sub(3);
                }
            }
            MouseEventKind::ScrollDown => {
                if self.is_in_rect(mouse_event.column, mouse_event.row, self.message_view_area) {
                    self.message_scroll_offset = self.message_scroll_offset.saturating_add(3);
                }
            }
            _ => {}
        }
    }

    fn is_in_rect(&self, x: u16, y: u16, rect: Rect) -> bool {
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }

    fn handle_conversation_list_click(&mut self, y: u16) {
        // Calculate which conversation was clicked
        // The conversation list has a border (1 line at top)
        // Each conversation item takes 2 lines
        let inner_y = y.saturating_sub(self.conversation_list_area.y + 1);
        let clicked_index = (inner_y / 2) as usize;

        if clicked_index < self.conversations.len() {
            self.selected_conversation = Some(clicked_index);
            self.message_scroll_offset = 0;
            self.select_current_conversation();
        }
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_conversation();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous_conversation();
                }
                KeyCode::PageUp => {
                    self.message_scroll_offset = self.message_scroll_offset.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.message_scroll_offset = self.message_scroll_offset.saturating_add(10);
                }
                KeyCode::Enter => {
                    if self.selected_conversation.is_some() {
                        self.input_mode = InputMode::Editing;
                        self.select_current_conversation();
                    }
                }
                KeyCode::Esc => {
                    self.deselect_conversation();
                }
                _ => {}
            },
            InputMode::Editing => match key_event.code {
                KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
                    self.input.push('\n');
                }
                KeyCode::Enter => {
                    self.send_message();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                }
                _ => {}
            },
        }
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
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(f.area());

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(5)])
                .split(chunks[1]);

            // Store the areas for mouse event handling
            app.conversation_list_area = chunks[0];
            app.message_view_area = right_chunks[0];
            app.input_box_area = right_chunks[1];

            // Render conversation list
            ConversationList::render(
                chunks[0],
                f.buffer_mut(),
                &app.conversations,
                app.selected_conversation,
            );

            // Render message view
            let selected_conv = app
                .selected_conversation
                .and_then(|idx| app.conversations.get(idx));
            MessageView::render(right_chunks[0], f.buffer_mut(), selected_conv, app.message_scroll_offset);

            // Render input box
            let is_editing = matches!(app.input_mode, InputMode::Editing);
            InputBox::render(right_chunks[1], f.buffer_mut(), &app.input, is_editing);
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
