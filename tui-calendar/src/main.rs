use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

const MIN_YEAR: i32 = 1900;
const MAX_YEAR: i32 = 3000;

#[derive(Clone, Debug)]
struct CalendarEvent {
    id: usize,
    title: String,
    description: String,
    date: NaiveDate,
    time: Option<NaiveTime>,
}

enum AppMode {
    Normal,
    CreateEvent,
    EditEvent,
    ViewEvent,
    ConfirmDelete,
}

enum CreateEventField {
    Title,
    Description,
    Date,
    Time,
}

struct App {
    mode: AppMode,
    events: Vec<CalendarEvent>,
    next_event_id: usize,
    current_date: NaiveDate,
    selected_date: NaiveDate,
    selected_event_index: Option<usize>,
    
    // Create event state
    new_event_title: Input,
    new_event_description: Input,
    new_event_date: Input,
    new_event_time: Input,
    create_event_field: CreateEventField,
    
    // Edit event state
    edit_event_id: Option<usize>,
    edit_event_title: Input,
    edit_event_description: Input,
    edit_event_date: Input,
    edit_event_time: Input,
    edit_event_field: CreateEventField,
    
    // Event list state
    event_list_state: ListState,
    
    // Vim-style number prefix support
    number_buffer: String,
}

impl App {
    fn new() -> Self {
        let today = Local::now().date_naive();
        let date_str = today.format("%Y-%m-%d").to_string();
        Self {
            mode: AppMode::Normal,
            events: Vec::new(),
            next_event_id: 1,
            current_date: today,
            selected_date: today,
            selected_event_index: None,
            new_event_title: Input::default(),
            new_event_description: Input::default(),
            new_event_date: Input::new(date_str.clone()),
            new_event_time: Input::default(),
            create_event_field: CreateEventField::Title,
            edit_event_id: None,
            edit_event_title: Input::default(),
            edit_event_description: Input::default(),
            edit_event_date: Input::new(date_str),
            edit_event_time: Input::default(),
            edit_event_field: CreateEventField::Title,
            event_list_state: ListState::default(),
            number_buffer: String::new(),
        }
    }

    fn get_events_for_date(&self, date: NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.date == date)
            .collect()
    }

    fn get_selected_date_events(&self) -> Vec<&CalendarEvent> {
        self.get_events_for_date(self.selected_date)
    }

    fn move_to_previous_month(&mut self) {
        self.selected_date = self
            .selected_date
            .with_day(1)
            .and_then(|d| d.pred_opt())
            .unwrap_or(self.selected_date);
    }

    fn move_to_next_month(&mut self) {
        self.selected_date = self
            .selected_date
            .with_day(1)
            .and_then(|d| d
                .with_month((d.month() % 12) + 1)
                .and_then(|nd| if nd.month() == 1 {
                    nd.with_year(nd.year() + 1)
                } else {
                    Some(nd)
                }))
            .unwrap_or(self.selected_date);
    }

    fn move_selection_up(&mut self) {
        self.move_selection_up_by(1);
    }

    fn move_selection_up_by(&mut self, count: usize) {
        // Move up by weeks (7 days per count)
        let days_to_move = count * 7;
        let mut new_date = self.selected_date;
        
        for _ in 0..days_to_move {
            if let Some(pred) = new_date.pred_opt() {
                if pred.year() >= MIN_YEAR {
                    new_date = pred;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        self.selected_date = new_date;
    }

    fn move_selection_down(&mut self) {
        self.move_selection_down_by(1);
    }

    fn move_selection_down_by(&mut self, count: usize) {
        // Move down by weeks (7 days per count)
        let days_to_move = count * 7;
        let mut new_date = self.selected_date;
        
        for _ in 0..days_to_move {
            if let Some(succ) = new_date.succ_opt() {
                if succ.year() <= MAX_YEAR {
                    new_date = succ;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        self.selected_date = new_date;
    }

    fn move_selection_left(&mut self) {
        self.move_selection_left_by(1);
    }

    fn move_selection_left_by(&mut self, count: usize) {
        let mut new_date = self.selected_date;
        
        for _ in 0..count {
            if let Some(pred) = new_date.pred_opt() {
                if pred.year() >= MIN_YEAR {
                    new_date = pred;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        self.selected_date = new_date;
    }

    fn move_selection_right(&mut self) {
        self.move_selection_right_by(1);
    }

    fn move_selection_right_by(&mut self, count: usize) {
        let mut new_date = self.selected_date;
        
        for _ in 0..count {
            if let Some(succ) = new_date.succ_opt() {
                if succ.year() <= MAX_YEAR {
                    new_date = succ;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        self.selected_date = new_date;
    }

    fn start_create_event(&mut self) {
        self.new_event_title = Input::default();
        self.new_event_description = Input::default();
        self.new_event_date = Input::new(self.selected_date.format("%Y-%m-%d").to_string());
        self.new_event_time = Input::default();
        self.create_event_field = CreateEventField::Title;
        self.mode = AppMode::CreateEvent;
    }

    fn create_event(&mut self) -> Result<()> {
        if self.new_event_title.value().trim().is_empty() {
            return Ok(());
        }

        let date = NaiveDate::parse_from_str(self.new_event_date.value(), "%Y-%m-%d")
            .unwrap_or(self.selected_date);

        let time = if self.new_event_time.value().trim().is_empty() {
            None
        } else {
            NaiveTime::parse_from_str(self.new_event_time.value(), "%H:%M").ok()
        };

        let event = CalendarEvent {
            id: self.next_event_id,
            title: self.new_event_title.value().trim().to_string(),
            description: self.new_event_description.value().trim().to_string(),
            date,
            time,
        };

        self.events.push(event);
        self.events.sort_by(|a, b| {
            a.date.cmp(&b.date).then_with(|| {
                match (a.time, b.time) {
                    (Some(at), Some(bt)) => at.cmp(&bt),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            })
        });
        self.next_event_id += 1;

        self.mode = AppMode::Normal;
        Ok(())
    }

    fn cancel_create_event(&mut self) {
        self.mode = AppMode::Normal;
    }

    fn start_edit_event(&mut self) {
        if let Some(idx) = self.event_list_state.selected() {
            let events = self.get_selected_date_events();
            if idx < events.len() {
                let event = events[idx];
                let event_id = event.id;
                let title = event.title.clone();
                let description = event.description.clone();
                let date = event.date.format("%Y-%m-%d").to_string();
                let time = event.time.map(|t| t.format("%H:%M").to_string()).unwrap_or_default();
                
                self.edit_event_id = Some(event_id);
                self.edit_event_title = Input::new(title);
                self.edit_event_description = Input::new(description);
                self.edit_event_date = Input::new(date);
                self.edit_event_time = Input::new(time);
                self.edit_event_field = CreateEventField::Title;
                self.mode = AppMode::EditEvent;
            }
        }
    }

    fn save_edited_event(&mut self) -> Result<()> {
        if let Some(event_id) = self.edit_event_id {
            if self.edit_event_title.value().trim().is_empty() {
                return Ok(());
            }

            let date = NaiveDate::parse_from_str(self.edit_event_date.value(), "%Y-%m-%d")
                .unwrap_or(self.selected_date);

            let time = if self.edit_event_time.value().trim().is_empty() {
                None
            } else {
                NaiveTime::parse_from_str(self.edit_event_time.value(), "%H:%M").ok()
            };

            if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
                event.title = self.edit_event_title.value().trim().to_string();
                event.description = self.edit_event_description.value().trim().to_string();
                event.date = date;
                event.time = time;
            }

            self.events.sort_by(|a, b| {
                a.date.cmp(&b.date).then_with(|| {
                    match (a.time, b.time) {
                        (Some(at), Some(bt)) => at.cmp(&bt),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                })
            });

            self.mode = AppMode::Normal;
            self.edit_event_id = None;
        }
        Ok(())
    }

    fn cancel_edit_event(&mut self) {
        self.mode = AppMode::Normal;
        self.edit_event_id = None;
    }

    fn show_event_details(&mut self) {
        if let Some(idx) = self.event_list_state.selected() {
            let events = self.get_selected_date_events();
            if idx < events.len() {
                self.selected_event_index = Some(events[idx].id);
                self.mode = AppMode::ViewEvent;
            }
        }
    }

    fn close_event_details(&mut self) {
        self.mode = AppMode::Normal;
        self.selected_event_index = None;
    }

    fn start_delete_event(&mut self) {
        if self.event_list_state.selected().is_some() {
            self.mode = AppMode::ConfirmDelete;
        }
    }

    fn confirm_delete_event(&mut self) {
        if let Some(idx) = self.event_list_state.selected() {
            let events = self.get_selected_date_events();
            if idx < events.len() {
                let event_id = events[idx].id;
                self.events.retain(|e| e.id != event_id);
                
                // Adjust selection
                if idx > 0 {
                    self.event_list_state.select(Some(idx - 1));
                } else if !self.get_selected_date_events().is_empty() {
                    self.event_list_state.select(Some(0));
                } else {
                    self.event_list_state.select(None);
                }
            }
        }
        self.mode = AppMode::Normal;
    }

    fn cancel_delete_event(&mut self) {
        self.mode = AppMode::Normal;
    }

    fn next_event_in_list(&mut self) {
        let events = self.get_selected_date_events();
        if events.is_empty() {
            return;
        }

        let i = match self.event_list_state.selected() {
            Some(i) => {
                if i >= events.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.event_list_state.select(Some(i));
    }

    fn previous_event_in_list(&mut self) {
        let events = self.get_selected_date_events();
        if events.is_empty() {
            return;
        }

        let i = match self.event_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    events.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.event_list_state.select(Some(i));
    }

    fn get_count(&self) -> usize {
        if self.number_buffer.is_empty() {
            1
        } else {
            // Cap at 9999 to prevent performance issues with very large numbers
            self.number_buffer.parse().unwrap_or(1).max(1).min(9999)
        }
    }

    fn handle_create_event_input(&mut self, key_event: &Event) {
        match self.create_event_field {
            CreateEventField::Title => {
                self.new_event_title.handle_event(key_event);
            }
            CreateEventField::Description => {
                self.new_event_description.handle_event(key_event);
            }
            CreateEventField::Date => {
                self.new_event_date.handle_event(key_event);
            }
            CreateEventField::Time => {
                self.new_event_time.handle_event(key_event);
            }
        }
    }

    fn handle_edit_event_input(&mut self, key_event: &Event) {
        match self.edit_event_field {
            CreateEventField::Title => {
                self.edit_event_title.handle_event(key_event);
            }
            CreateEventField::Description => {
                self.edit_event_description.handle_event(key_event);
            }
            CreateEventField::Date => {
                self.edit_event_date.handle_event(key_event);
            }
            CreateEventField::Time => {
                self.edit_event_time.handle_event(key_event);
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.area());

    render_calendar(f, app, chunks[0]);
    render_event_list(f, app, chunks[1]);

    match app.mode {
        AppMode::CreateEvent => render_create_event_modal(f, app),
        AppMode::EditEvent => render_edit_event_modal(f, app),
        AppMode::ViewEvent => render_view_event_modal(f, app),
        AppMode::ConfirmDelete => render_confirm_delete_modal(f, app),
        _ => {}
    }
}

fn render_calendar(f: &mut Frame, app: &App, area: Rect) {
    let first_of_month = app
        .selected_date
        .with_day(1)
        .unwrap_or(app.selected_date);

    let year = first_of_month.year();
    let month = first_of_month.month();

    let title = format!(
        " {} {} ",
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        },
        year
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Calculate layout for calendar grid
    if inner.height < 10 || inner.width < 30 {
        return;
    }

    let weekday_of_first = first_of_month.weekday().num_days_from_sunday() as usize;
    let days_in_month = days_in_month(year, month);

    // Render weekday headers
    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };

    let weekdays = vec![
        Span::styled("Su ", Style::default().fg(Color::Yellow)),
        Span::styled("Mo ", Style::default().fg(Color::Yellow)),
        Span::styled("Tu ", Style::default().fg(Color::Yellow)),
        Span::styled("We ", Style::default().fg(Color::Yellow)),
        Span::styled("Th ", Style::default().fg(Color::Yellow)),
        Span::styled("Fr ", Style::default().fg(Color::Yellow)),
        Span::styled("Sa ", Style::default().fg(Color::Yellow)),
    ];
    let header_line = Line::from(weekdays);
    f.render_widget(Paragraph::new(header_line), header_area);

    // Render days
    let grid_area = Rect {
        x: inner.x,
        y: inner.y + 2,
        width: inner.width,
        height: inner.height.saturating_sub(2),
    };

    let mut lines = Vec::new();
    let mut current_line = Vec::new();

    // Add empty cells for days before the first of the month
    for _ in 0..weekday_of_first {
        current_line.push(Span::raw("   "));
    }

    // Add days of the month
    for day in 1..=days_in_month {
        let date = match NaiveDate::from_ymd_opt(year, month, day) {
            Some(d) => d,
            None => continue, // Skip invalid dates
        };
        let is_today = date == app.current_date;
        let is_selected = date == app.selected_date;
        let has_events = !app.get_events_for_date(date).is_empty();

        let day_str = if has_events {
            format!("{:2}*", day)
        } else {
            format!("{:2} ", day)
        };

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_today {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if has_events {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default()
        };

        current_line.push(Span::styled(day_str, style));

        if (weekday_of_first + day as usize) % 7 == 0 {
            lines.push(Line::from(current_line.clone()));
            current_line.clear();
        }
    }

    // Add the last line if it has content
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }

    let calendar_text = Text::from(lines);
    f.render_widget(Paragraph::new(calendar_text), grid_area);

    // Render help text
    let help_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(3),
        width: inner.width,
        height: 2,
    };

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Arrows", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("Ctrl+N", Style::default().fg(Color::Yellow)),
            Span::raw(": New Event  "),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    f.render_widget(Paragraph::new(help_text), help_area);
}

fn render_event_list(f: &mut Frame, app: &mut App, area: Rect) {
    let events = app.get_selected_date_events();

    let title = format!(
        " Events - {} ",
        app.selected_date.format("%Y-%m-%d")
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center);

    if events.is_empty() {
        let inner = block.inner(area);
        f.render_widget(block, area);
        let no_events = Paragraph::new("No events for this day")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(no_events, inner);
        return;
    }

    let items: Vec<ListItem> = events
        .iter()
        .map(|event| {
            let time_str = event
                .time
                .map(|t| format!("{} - ", t.format("%H:%M")))
                .unwrap_or_default();
            let content = format!("{}{}", time_str, event.title);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.event_list_state);

    // Render help text at bottom
    let help_area = Rect {
        x: area.x + 2,
        y: area.y + area.height.saturating_sub(2),
        width: area.width.saturating_sub(4),
        height: 1,
    };

    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(": View  "),
        Span::styled("E", Style::default().fg(Color::Yellow)),
        Span::raw(": Edit  "),
        Span::styled("Del", Style::default().fg(Color::Yellow)),
        Span::raw(": Delete"),
    ]);
    f.render_widget(Paragraph::new(help_text), help_area);
}

fn render_create_event_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.area());

    let block = Block::default()
        .title(" Create New Event ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);

    // Title field
    let title_style = if matches!(app.create_event_field, CreateEventField::Title) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let title_text = vec![
        Line::from(Span::styled("Title:", title_style)),
        Line::from(app.new_event_title.value()),
    ];
    let title_para = Paragraph::new(title_text);
    f.render_widget(title_para, chunks[0]);
    
    // Render cursor for title field
    if matches!(app.create_event_field, CreateEventField::Title) {
        let cursor_pos = app.new_event_title.visual_cursor().min(chunks[0].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[0].x + cursor_pos as u16, chunks[0].y + 1));
    }

    // Description field
    let desc_style = if matches!(app.create_event_field, CreateEventField::Description) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_text = vec![
        Line::from(Span::styled("Description:", desc_style)),
        Line::from(app.new_event_description.value()),
    ];
    let desc_para = Paragraph::new(desc_text);
    f.render_widget(desc_para, chunks[1]);
    
    // Render cursor for description field
    if matches!(app.create_event_field, CreateEventField::Description) {
        let cursor_pos = app.new_event_description.visual_cursor().min(chunks[1].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[1].x + cursor_pos as u16, chunks[1].y + 1));
    }

    // Date field
    let date_style = if matches!(app.create_event_field, CreateEventField::Date) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let date_text = vec![
        Line::from(Span::styled("Date (YYYY-MM-DD):", date_style)),
        Line::from(app.new_event_date.value()),
    ];
    let date_para = Paragraph::new(date_text);
    f.render_widget(date_para, chunks[2]);
    
    // Render cursor for date field
    if matches!(app.create_event_field, CreateEventField::Date) {
        let cursor_pos = app.new_event_date.visual_cursor().min(chunks[2].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[2].x + cursor_pos as u16, chunks[2].y + 1));
    }

    // Time field
    let time_style = if matches!(app.create_event_field, CreateEventField::Time) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let time_text = vec![
        Line::from(Span::styled("Time (HH:MM, optional):", time_style)),
        Line::from(app.new_event_time.value()),
    ];
    let time_para = Paragraph::new(time_text);
    f.render_widget(time_para, chunks[3]);
    
    // Render cursor for time field
    if matches!(app.create_event_field, CreateEventField::Time) {
        let cursor_pos = app.new_event_time.visual_cursor().min(chunks[3].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[3].x + cursor_pos as u16, chunks[3].y + 1));
    }

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": Next Field  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Save  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": Cancel"),
        ]),
    ];
    f.render_widget(Paragraph::new(help_text), chunks[4]);
}

fn render_edit_event_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.area());

    let block = Block::default()
        .title(" Edit Event ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);

    // Title field
    let title_style = if matches!(app.edit_event_field, CreateEventField::Title) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let title_text = vec![
        Line::from(Span::styled("Title:", title_style)),
        Line::from(app.edit_event_title.value()),
    ];
    let title_para = Paragraph::new(title_text);
    f.render_widget(title_para, chunks[0]);
    
    // Render cursor for title field
    if matches!(app.edit_event_field, CreateEventField::Title) {
        let cursor_pos = app.edit_event_title.visual_cursor().min(chunks[0].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[0].x + cursor_pos as u16, chunks[0].y + 1));
    }

    // Description field
    let desc_style = if matches!(app.edit_event_field, CreateEventField::Description) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_text = vec![
        Line::from(Span::styled("Description:", desc_style)),
        Line::from(app.edit_event_description.value()),
    ];
    let desc_para = Paragraph::new(desc_text);
    f.render_widget(desc_para, chunks[1]);
    
    // Render cursor for description field
    if matches!(app.edit_event_field, CreateEventField::Description) {
        let cursor_pos = app.edit_event_description.visual_cursor().min(chunks[1].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[1].x + cursor_pos as u16, chunks[1].y + 1));
    }

    // Date field
    let date_style = if matches!(app.edit_event_field, CreateEventField::Date) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let date_text = vec![
        Line::from(Span::styled("Date (YYYY-MM-DD):", date_style)),
        Line::from(app.edit_event_date.value()),
    ];
    let date_para = Paragraph::new(date_text);
    f.render_widget(date_para, chunks[2]);
    
    // Render cursor for date field
    if matches!(app.edit_event_field, CreateEventField::Date) {
        let cursor_pos = app.edit_event_date.visual_cursor().min(chunks[2].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[2].x + cursor_pos as u16, chunks[2].y + 1));
    }

    // Time field
    let time_style = if matches!(app.edit_event_field, CreateEventField::Time) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let time_text = vec![
        Line::from(Span::styled("Time (HH:MM, optional):", time_style)),
        Line::from(app.edit_event_time.value()),
    ];
    let time_para = Paragraph::new(time_text);
    f.render_widget(time_para, chunks[3]);
    
    // Render cursor for time field
    if matches!(app.edit_event_field, CreateEventField::Time) {
        let cursor_pos = app.edit_event_time.visual_cursor().min(chunks[3].width.saturating_sub(1) as usize);
        f.set_cursor_position((chunks[3].x + cursor_pos as u16, chunks[3].y + 1));
    }

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(": Next Field  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(": Save  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(": Cancel"),
        ]),
    ];
    f.render_widget(Paragraph::new(help_text), chunks[4]);
}

fn render_view_event_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());

    if let Some(event_id) = app.selected_event_index {
        if let Some(event) = app.events.iter().find(|e| e.id == event_id) {
            let block = Block::default()
                .title(" Event Details ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));

            f.render_widget(Clear, area);
            f.render_widget(block.clone(), area);

            let inner = block.inner(area);

            let time_str = event
                .time
                .map(|t| format!("Time: {}\n", t.format("%H:%M")))
                .unwrap_or_default();

            let content = format!(
                "Title: {}\n\nDate: {}\n{}\nDescription:\n{}",
                event.title,
                event.date.format("%Y-%m-%d"),
                time_str,
                if event.description.is_empty() {
                    "(No description)"
                } else {
                    &event.description
                }
            );

            let text = Text::from(content);
            let paragraph = Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::White));

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(1), Constraint::Length(2)])
                .split(inner);

            f.render_widget(paragraph, chunks[0]);

            let help_text = Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(": Close"),
            ]);
            f.render_widget(Paragraph::new(help_text), chunks[1]);
        }
    }
}

fn render_confirm_delete_modal(f: &mut Frame, _app: &App) {
    let area = centered_rect(50, 30, f.area());

    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    let text = vec![
        Line::from(""),
        Line::from("Are you sure you want to delete this event?"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("Y", Style::default().fg(Color::Red)),
            Span::raw(": Yes, delete  "),
            Span::styled("N/Esc", Style::default().fg(Color::Green)),
            Span::raw(": No, cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn days_in_month(year: i32, month: u32) -> u32 {
    // Try to get the first day of next month, then subtract 1 to get last day of current month
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    
    next_month_date
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or_else(|| {
            // Fallback to known month lengths if date calculations fail
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    // Check for leap year
                    if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                        29
                    } else {
                        28
                    }
                }
                _ => 31, // Default fallback
            }
        })
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Ok(());
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.start_create_event();
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        // Build up number buffer for vim-style numeric prefixes
                        if app.number_buffer.len() < 4 {
                            app.number_buffer.push(c);
                        }
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if app.event_list_state.selected().is_some() {
                            app.start_edit_event();
                        }
                        app.number_buffer.clear();
                    }
                    KeyCode::Left => {
                        let count = app.get_count();
                        app.move_selection_left_by(count);
                        app.event_list_state.select(None);
                        app.number_buffer.clear();
                    }
                    KeyCode::Right => {
                        let count = app.get_count();
                        app.move_selection_right_by(count);
                        app.event_list_state.select(None);
                        app.number_buffer.clear();
                    }
                    KeyCode::Up => {
                        if app.event_list_state.selected().is_some() {
                            app.previous_event_in_list();
                            app.number_buffer.clear();
                        } else {
                            let count = app.get_count();
                            app.move_selection_up_by(count);
                            app.number_buffer.clear();
                        }
                    }
                    KeyCode::Down => {
                        if app.event_list_state.selected().is_some() {
                            app.next_event_in_list();
                            app.number_buffer.clear();
                        } else {
                            let count = app.get_count();
                            app.move_selection_down_by(count);
                            app.number_buffer.clear();
                        }
                    }
                    KeyCode::Tab => {
                        let events = app.get_selected_date_events();
                        if !events.is_empty() {
                            if app.event_list_state.selected().is_none() {
                                app.event_list_state.select(Some(0));
                            } else {
                                app.event_list_state.select(None);
                            }
                        }
                        app.number_buffer.clear();
                    }
                    KeyCode::Enter => {
                        if app.event_list_state.selected().is_some() {
                            app.show_event_details();
                        }
                        app.number_buffer.clear();
                    }
                    KeyCode::Delete => {
                        app.start_delete_event();
                        app.number_buffer.clear();
                    }
                    KeyCode::Char(',') => {
                        app.move_to_previous_month();
                        app.number_buffer.clear();
                    }
                    KeyCode::Char('.') => {
                        app.move_to_next_month();
                        app.number_buffer.clear();
                    }
                    KeyCode::Esc => {
                        app.number_buffer.clear();
                    }
                    _ => {}
                },
                AppMode::CreateEvent => {
                    let input_event = Event::Key(key);
                    match key.code {
                        KeyCode::Tab => {
                            app.create_event_field = match app.create_event_field {
                                CreateEventField::Title => CreateEventField::Description,
                                CreateEventField::Description => CreateEventField::Date,
                                CreateEventField::Date => CreateEventField::Time,
                                CreateEventField::Time => CreateEventField::Title,
                            };
                        }
                        KeyCode::Enter => {
                            let _ = app.create_event();
                        }
                        KeyCode::Esc => {
                            app.cancel_create_event();
                        }
                        _ => {
                            app.handle_create_event_input(&input_event);
                        }
                    }
                }
                AppMode::EditEvent => {
                    let input_event = Event::Key(key);
                    match key.code {
                        KeyCode::Tab => {
                            app.edit_event_field = match app.edit_event_field {
                                CreateEventField::Title => CreateEventField::Description,
                                CreateEventField::Description => CreateEventField::Date,
                                CreateEventField::Date => CreateEventField::Time,
                                CreateEventField::Time => CreateEventField::Title,
                            };
                        }
                        KeyCode::Enter => {
                            let _ = app.save_edited_event();
                        }
                        KeyCode::Esc => {
                            app.cancel_edit_event();
                        }
                        _ => {
                            app.handle_edit_event_input(&input_event);
                        }
                    }
                }
                AppMode::ViewEvent => match key.code {
                    KeyCode::Esc => {
                        app.close_event_details();
                    }
                    _ => {}
                },
                AppMode::ConfirmDelete => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.confirm_delete_event();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_delete_event();
                    }
                    _ => {}
                },
            }
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
