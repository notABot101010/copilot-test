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
    new_event_title: String,
    new_event_description: String,
    new_event_date: String,
    new_event_time: String,
    create_event_field: CreateEventField,
    
    // Event list state
    event_list_state: ListState,
}

impl App {
    fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            mode: AppMode::Normal,
            events: Vec::new(),
            next_event_id: 1,
            current_date: today,
            selected_date: today,
            selected_event_index: None,
            new_event_title: String::new(),
            new_event_description: String::new(),
            new_event_date: today.format("%Y-%m-%d").to_string(),
            new_event_time: String::new(),
            create_event_field: CreateEventField::Title,
            event_list_state: ListState::default(),
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
        if let Some(new_date) = self.selected_date.pred_opt() {
            if new_date.year() >= 1900 {
                self.selected_date = new_date;
            }
        }
    }

    fn move_selection_down(&mut self) {
        if let Some(new_date) = self.selected_date.succ_opt() {
            if new_date.year() <= 3000 {
                self.selected_date = new_date;
            }
        }
    }

    fn move_selection_left(&mut self) {
        self.selected_date = self
            .selected_date
            .pred_opt()
            .filter(|d| d.year() >= 1900)
            .unwrap_or(self.selected_date);
    }

    fn move_selection_right(&mut self) {
        self.selected_date = self
            .selected_date
            .succ_opt()
            .filter(|d| d.year() <= 3000)
            .unwrap_or(self.selected_date);
    }

    fn start_create_event(&mut self) {
        self.new_event_title.clear();
        self.new_event_description.clear();
        self.new_event_date = self.selected_date.format("%Y-%m-%d").to_string();
        self.new_event_time.clear();
        self.create_event_field = CreateEventField::Title;
        self.mode = AppMode::CreateEvent;
    }

    fn create_event(&mut self) -> Result<()> {
        if self.new_event_title.trim().is_empty() {
            return Ok(());
        }

        let date = NaiveDate::parse_from_str(&self.new_event_date, "%Y-%m-%d")
            .unwrap_or(self.selected_date);

        let time = if self.new_event_time.trim().is_empty() {
            None
        } else {
            NaiveTime::parse_from_str(&self.new_event_time, "%H:%M").ok()
        };

        let event = CalendarEvent {
            id: self.next_event_id,
            title: self.new_event_title.trim().to_string(),
            description: self.new_event_description.trim().to_string(),
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

    fn handle_create_event_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => {
                match self.create_event_field {
                    CreateEventField::Title => self.new_event_title.push(c),
                    CreateEventField::Description => self.new_event_description.push(c),
                    CreateEventField::Date => self.new_event_date.push(c),
                    CreateEventField::Time => self.new_event_time.push(c),
                }
            }
            KeyCode::Backspace => {
                match self.create_event_field {
                    CreateEventField::Title => {
                        self.new_event_title.pop();
                    }
                    CreateEventField::Description => {
                        self.new_event_description.pop();
                    }
                    CreateEventField::Date => {
                        self.new_event_date.pop();
                    }
                    CreateEventField::Time => {
                        self.new_event_time.pop();
                    }
                }
            }
            KeyCode::Tab => {
                self.create_event_field = match self.create_event_field {
                    CreateEventField::Title => CreateEventField::Description,
                    CreateEventField::Description => CreateEventField::Date,
                    CreateEventField::Date => CreateEventField::Time,
                    CreateEventField::Time => CreateEventField::Title,
                };
            }
            KeyCode::Enter => {
                let _ = self.create_event();
            }
            KeyCode::Esc => {
                self.cancel_create_event();
            }
            _ => {}
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
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
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
        Line::from(app.new_event_title.as_str()),
    ];
    f.render_widget(Paragraph::new(title_text), chunks[0]);

    // Description field
    let desc_style = if matches!(app.create_event_field, CreateEventField::Description) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_text = vec![
        Line::from(Span::styled("Description:", desc_style)),
        Line::from(app.new_event_description.as_str()),
    ];
    f.render_widget(Paragraph::new(desc_text), chunks[1]);

    // Date field
    let date_style = if matches!(app.create_event_field, CreateEventField::Date) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let date_text = vec![
        Line::from(Span::styled("Date (YYYY-MM-DD):", date_style)),
        Line::from(app.new_event_date.as_str()),
    ];
    f.render_widget(Paragraph::new(date_text), chunks[2]);

    // Time field
    let time_style = if matches!(app.create_event_field, CreateEventField::Time) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let time_text = vec![
        Line::from(Span::styled("Time (HH:MM, optional):", time_style)),
        Line::from(app.new_event_time.as_str()),
    ];
    f.render_widget(Paragraph::new(time_text), chunks[3]);

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
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt()
        .map(|d| d.day())
        .unwrap_or(31)
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
                    KeyCode::Left => {
                        app.move_selection_left();
                        app.event_list_state.select(None);
                    }
                    KeyCode::Right => {
                        app.move_selection_right();
                        app.event_list_state.select(None);
                    }
                    KeyCode::Up => {
                        if app.event_list_state.selected().is_some() {
                            app.previous_event_in_list();
                        } else {
                            app.move_selection_up();
                        }
                    }
                    KeyCode::Down => {
                        if app.event_list_state.selected().is_some() {
                            app.next_event_in_list();
                        } else {
                            app.move_selection_down();
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
                    }
                    KeyCode::Enter => {
                        if app.event_list_state.selected().is_some() {
                            app.show_event_details();
                        }
                    }
                    KeyCode::Delete => {
                        app.start_delete_event();
                    }
                    KeyCode::Char(',') => {
                        app.move_to_previous_month();
                    }
                    KeyCode::Char('.') => {
                        app.move_to_next_month();
                    }
                    _ => {}
                },
                AppMode::CreateEvent => {
                    app.handle_create_event_input(key.code);
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
