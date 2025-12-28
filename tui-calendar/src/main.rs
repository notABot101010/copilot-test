use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};
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
    widgets::{Block, Borders, Clear, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

const MIN_YEAR: i32 = 1900;
const MAX_YEAR: i32 = 3000;
const MAX_COUNT: usize = 9999;
const MAX_BUFFER_LEN: usize = 4;

#[derive(Clone, Debug)]
struct CalendarEvent {
    id: usize,
    title: String,
    description: String,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>, // For multi-day events
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    category: Option<String>,
}

enum AppMode {
    Normal,
    CreateEvent,
    EditEvent,
    ViewEvent,
    ConfirmDelete,
    WeekView,
}

enum CreateEventField {
    Title,
    Description,
    StartDate,
    EndDate,
    StartTime,
    EndTime,
    Category,
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
    new_event_start_date: Input,
    new_event_end_date: Input,
    new_event_start_time: Input,
    new_event_end_time: Input,
    new_event_category: Input,
    create_event_field: CreateEventField,
    
    // Edit event state
    edit_event_id: Option<usize>,
    edit_event_title: Input,
    edit_event_description: Input,
    edit_event_start_date: Input,
    edit_event_end_date: Input,
    edit_event_start_time: Input,
    edit_event_end_time: Input,
    edit_event_category: Input,
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
            new_event_start_date: Input::new(date_str.clone()),
            new_event_end_date: Input::default(),
            new_event_start_time: Input::default(),
            new_event_end_time: Input::default(),
            new_event_category: Input::default(),
            create_event_field: CreateEventField::Title,
            edit_event_id: None,
            edit_event_title: Input::default(),
            edit_event_description: Input::default(),
            edit_event_start_date: Input::new(date_str),
            edit_event_end_date: Input::default(),
            edit_event_start_time: Input::default(),
            edit_event_end_time: Input::default(),
            edit_event_category: Input::default(),
            edit_event_field: CreateEventField::Title,
            event_list_state: ListState::default(),
            number_buffer: String::new(),
        }
    }

    fn get_events_for_date(&self, date: NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| {
                // Check if the event occurs on this date
                let end_date = e.end_date.unwrap_or(e.start_date);
                date >= e.start_date && date <= end_date
            })
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
        let days_to_move = (count * 7) as u64;
        
        if let Some(new_date) = self.selected_date.checked_sub_days(chrono::Days::new(days_to_move)) {
            if new_date.year() >= MIN_YEAR {
                self.selected_date = new_date;
            }
        }
    }

    fn move_selection_down(&mut self) {
        self.move_selection_down_by(1);
    }

    fn move_selection_down_by(&mut self, count: usize) {
        // Move down by weeks (7 days per count)
        let days_to_move = (count * 7) as u64;
        
        if let Some(new_date) = self.selected_date.checked_add_days(chrono::Days::new(days_to_move)) {
            if new_date.year() <= MAX_YEAR {
                self.selected_date = new_date;
            }
        }
    }

    fn move_selection_left(&mut self) {
        self.move_selection_left_by(1);
    }

    fn move_selection_left_by(&mut self, count: usize) {
        let days_to_move = count as u64;
        
        if let Some(new_date) = self.selected_date.checked_sub_days(chrono::Days::new(days_to_move)) {
            if new_date.year() >= MIN_YEAR {
                self.selected_date = new_date;
            }
        }
    }

    fn move_selection_right(&mut self) {
        self.move_selection_right_by(1);
    }

    fn move_selection_right_by(&mut self, count: usize) {
        let days_to_move = count as u64;
        
        if let Some(new_date) = self.selected_date.checked_add_days(chrono::Days::new(days_to_move)) {
            if new_date.year() <= MAX_YEAR {
                self.selected_date = new_date;
            }
        }
    }

    fn start_create_event(&mut self) {
        self.new_event_title = Input::default();
        self.new_event_description = Input::default();
        self.new_event_start_date = Input::new(self.selected_date.format("%Y-%m-%d").to_string());
        self.new_event_end_date = Input::default();
        self.new_event_start_time = Input::default();
        self.new_event_end_time = Input::default();
        self.new_event_category = Input::default();
        self.create_event_field = CreateEventField::Title;
        self.mode = AppMode::CreateEvent;
    }

    fn create_event(&mut self) -> Result<()> {
        if self.new_event_title.value().trim().is_empty() {
            return Ok(());
        }

        let start_date = NaiveDate::parse_from_str(self.new_event_start_date.value(), "%Y-%m-%d")
            .unwrap_or(self.selected_date);

        let end_date = if self.new_event_end_date.value().trim().is_empty() {
            None
        } else {
            NaiveDate::parse_from_str(self.new_event_end_date.value(), "%Y-%m-%d").ok()
        };

        let start_time = if self.new_event_start_time.value().trim().is_empty() {
            None
        } else {
            NaiveTime::parse_from_str(self.new_event_start_time.value(), "%H:%M").ok()
        };

        let end_time = if self.new_event_end_time.value().trim().is_empty() {
            None
        } else {
            NaiveTime::parse_from_str(self.new_event_end_time.value(), "%H:%M").ok()
        };

        let category = if self.new_event_category.value().trim().is_empty() {
            None
        } else {
            Some(self.new_event_category.value().trim().to_string())
        };

        let event = CalendarEvent {
            id: self.next_event_id,
            title: self.new_event_title.value().trim().to_string(),
            description: self.new_event_description.value().trim().to_string(),
            start_date,
            end_date,
            start_time,
            end_time,
            category,
        };

        self.events.push(event);
        self.events.sort_by(|a, b| {
            a.start_date.cmp(&b.start_date).then_with(|| {
                match (a.start_time, b.start_time) {
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
                let start_date = event.start_date.format("%Y-%m-%d").to_string();
                let end_date = event.end_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                let start_time = event.start_time.map(|t| t.format("%H:%M").to_string()).unwrap_or_default();
                let end_time = event.end_time.map(|t| t.format("%H:%M").to_string()).unwrap_or_default();
                let category = event.category.clone().unwrap_or_default();
                
                self.edit_event_id = Some(event_id);
                self.edit_event_title = Input::new(title);
                self.edit_event_description = Input::new(description);
                self.edit_event_start_date = Input::new(start_date);
                self.edit_event_end_date = Input::new(end_date);
                self.edit_event_start_time = Input::new(start_time);
                self.edit_event_end_time = Input::new(end_time);
                self.edit_event_category = Input::new(category);
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

            let start_date = NaiveDate::parse_from_str(self.edit_event_start_date.value(), "%Y-%m-%d")
                .unwrap_or(self.selected_date);

            let end_date = if self.edit_event_end_date.value().trim().is_empty() {
                None
            } else {
                NaiveDate::parse_from_str(self.edit_event_end_date.value(), "%Y-%m-%d").ok()
            };

            let start_time = if self.edit_event_start_time.value().trim().is_empty() {
                None
            } else {
                NaiveTime::parse_from_str(self.edit_event_start_time.value(), "%H:%M").ok()
            };

            let end_time = if self.edit_event_end_time.value().trim().is_empty() {
                None
            } else {
                NaiveTime::parse_from_str(self.edit_event_end_time.value(), "%H:%M").ok()
            };

            let category = if self.edit_event_category.value().trim().is_empty() {
                None
            } else {
                Some(self.edit_event_category.value().trim().to_string())
            };

            if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
                event.title = self.edit_event_title.value().trim().to_string();
                event.description = self.edit_event_description.value().trim().to_string();
                event.start_date = start_date;
                event.end_date = end_date;
                event.start_time = start_time;
                event.end_time = end_time;
                event.category = category;
            }

            self.events.sort_by(|a, b| {
                a.start_date.cmp(&b.start_date).then_with(|| {
                    match (a.start_time, b.start_time) {
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
            return 1;
        }
        
        // Try to parse the buffer, cap at MAX_COUNT to prevent performance issues
        self.number_buffer
            .parse::<usize>()
            .unwrap_or(1)
            .max(1)
            .min(MAX_COUNT)
    }

    fn toggle_week_view(&mut self) {
        match self.mode {
            AppMode::Normal => {
                self.mode = AppMode::WeekView;
            }
            AppMode::WeekView => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn get_week_dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let weekday = self.selected_date.weekday().num_days_from_sunday();
        
        // Get the start of the week (Sunday)
        let week_start = self.selected_date
            .checked_sub_days(chrono::Days::new(weekday as u64))
            .unwrap_or(self.selected_date);
        
        // Get all 7 days of the week
        for i in 0..7 {
            if let Some(date) = week_start.checked_add_days(chrono::Days::new(i)) {
                dates.push(date);
            }
        }
        
        dates
    }

    fn handle_create_event_input(&mut self, key_event: &Event) {
        match self.create_event_field {
            CreateEventField::Title => {
                self.new_event_title.handle_event(key_event);
            }
            CreateEventField::Description => {
                self.new_event_description.handle_event(key_event);
            }
            CreateEventField::StartDate => {
                self.new_event_start_date.handle_event(key_event);
            }
            CreateEventField::EndDate => {
                self.new_event_end_date.handle_event(key_event);
            }
            CreateEventField::StartTime => {
                self.new_event_start_time.handle_event(key_event);
            }
            CreateEventField::EndTime => {
                self.new_event_end_time.handle_event(key_event);
            }
            CreateEventField::Category => {
                self.new_event_category.handle_event(key_event);
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
            CreateEventField::StartDate => {
                self.edit_event_start_date.handle_event(key_event);
            }
            CreateEventField::EndDate => {
                self.edit_event_end_date.handle_event(key_event);
            }
            CreateEventField::StartTime => {
                self.edit_event_start_time.handle_event(key_event);
            }
            CreateEventField::EndTime => {
                self.edit_event_end_time.handle_event(key_event);
            }
            CreateEventField::Category => {
                self.edit_event_category.handle_event(key_event);
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    match app.mode {
        AppMode::WeekView => {
            render_week_view(f, app, f.area());
        }
        _ => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(f.area());

            render_calendar(f, app, chunks[0]);
            render_day_view(f, app, chunks[1]);
        }
    }

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

    // Calculate grid dimensions
    let cell_width = (inner.width / 7).max(12); // At least 12 chars wide per cell
    let header_height = 1;
    let help_height = 2;
    
    // Calculate number of weeks to display
    let total_cells = weekday_of_first + days_in_month as usize;
    let num_weeks = ((total_cells + 6) / 7).min(6); // Round up, max 6 weeks
    
    // Calculate cell height dynamically based on available space
    let available_grid_height = inner.height.saturating_sub(header_height + help_height + 1);
    let cell_height = if num_weeks > 0 {
        (available_grid_height / num_weeks as u16).max(3) // At least 3 lines per cell
    } else {
        3
    };
    
    // Render weekday headers
    let header_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: header_height,
    };

    let mut weekday_spans = Vec::new();
    let weekday_labels = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    for label in weekday_labels.iter() {
        let shortened = &label[0..2.min(label.len())];
        let padded = format!("{:^width$}", shortened, width = cell_width as usize);
        weekday_spans.push(Span::styled(padded, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }
    let header_line = Line::from(weekday_spans);
    f.render_widget(Paragraph::new(header_line), header_area);

    // Render calendar grid with boxes
    let grid_area = Rect {
        x: inner.x,
        y: inner.y + header_height + 1,
        width: inner.width,
        height: inner.height.saturating_sub(header_height + help_height + 1),
    };

    let mut day_counter = 1;
    let mut current_weekday = weekday_of_first;

    // Render each week row
    for week in 0..num_weeks {
        let week_y = grid_area.y + (week as u16 * cell_height);
        if week_y + cell_height > grid_area.y + grid_area.height {
            break;
        }

        // Render each day in the week
        for weekday in 0..7 {
            let cell_x = grid_area.x + (weekday as u16 * cell_width);
            
            // Check if we should render a day
            let should_render_day = if week == 0 {
                weekday >= weekday_of_first && day_counter <= days_in_month
            } else {
                day_counter <= days_in_month
            };

            if should_render_day {
                let date = match NaiveDate::from_ymd_opt(year, month, day_counter) {
                    Some(d) => d,
                    None => {
                        day_counter += 1;
                        continue;
                    }
                };

                let is_today = date == app.current_date;
                let is_selected = date == app.selected_date;
                let events = app.get_events_for_date(date);

                // Render day cell box
                let cell_area = Rect {
                    x: cell_x,
                    y: week_y,
                    width: cell_width.min(grid_area.x + grid_area.width - cell_x),
                    height: cell_height,
                };

                render_day_cell(f, cell_area, day_counter, is_today, is_selected, &events);
                day_counter += 1;
            }
            
            current_weekday = (current_weekday + 1) % 7;
        }
    }

    // Render help text
    let help_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(help_height),
        width: inner.width,
        height: help_height,
    };

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Arrows", Style::default().fg(Color::Yellow)),
            Span::raw(": Navigate  "),
            Span::styled("W", Style::default().fg(Color::Yellow)),
            Span::raw(": Week View  "),
            Span::styled("Ctrl+N", Style::default().fg(Color::Yellow)),
            Span::raw(": New  "),
            Span::styled("Ctrl+T", Style::default().fg(Color::Yellow)),
            Span::raw(": Today  "),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    f.render_widget(Paragraph::new(help_text), help_area);
}

fn render_day_cell(f: &mut Frame, area: Rect, day: u32, is_today: bool, is_selected: bool, events: &[&CalendarEvent]) {
    if area.width < 3 || area.height < 2 {
        return;
    }

    // Determine box style based on state
    let (border_style, bg_color) = if is_selected {
        (Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD), Some(Color::Blue))
    } else if is_today {
        (Style::default().fg(Color::Green).add_modifier(Modifier::BOLD), None)
    } else if !events.is_empty() {
        (Style::default().fg(Color::Magenta), None)
    } else {
        (Style::default().fg(Color::Gray), None)
    };

    // Create block for day cell
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    
    if let Some(bg) = bg_color {
        block = block.style(Style::default().bg(bg));
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render day number at the top
    let day_text = format!("{:>2}", day);
    let day_style = if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else if is_today {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    if inner.height > 0 {
        let day_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(day_text).style(day_style).alignment(Alignment::Right),
            day_area
        );
    }

    // Render event previews (can show more events with more height)
    if !events.is_empty() && inner.height > 1 {
        let events_area = Rect {
            x: inner.x,
            y: inner.y + 1,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };

        let mut event_lines = Vec::new();
        let max_events_to_show = events_area.height.saturating_sub(1) as usize;
        
        for event in events.iter().take(max_events_to_show) {
            // Show time range if both start and end times are present
            let time_str = if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                format!("{}-{} ", start.format("%H:%M"), end.format("%H:%M"))
            } else if let Some(start) = event.start_time {
                format!("{} ", start.format("%H:%M"))
            } else {
                String::new()
            };
            
            let available_width = events_area.width as usize;
            let max_title_len = available_width.saturating_sub(time_str.len()).max(1);
            
            // Use char-based truncation to handle multi-byte UTF-8 characters safely
            let title = if event.title.chars().count() > max_title_len {
                format!("{}…", event.title.chars().take(max_title_len.saturating_sub(1)).collect::<String>())
            } else {
                event.title.clone()
            };
            
            let event_text = format!("{}{}", time_str, title);
            
            // Choose color based on category
            let event_color = event.category.as_ref().and_then(|cat| {
                match cat.to_lowercase().as_str() {
                    "work" => Some(Color::Cyan),
                    "personal" => Some(Color::Green),
                    "meeting" => Some(Color::Yellow),
                    "important" => Some(Color::Red),
                    _ => None,
                }
            }).unwrap_or(Color::White);
            
            event_lines.push(Line::from(Span::styled(
                event_text,
                Style::default().fg(event_color)
            )));
        }

        // Show "+N more" if there are more events
        if events.len() > max_events_to_show && event_lines.len() < events_area.height as usize {
            let more_count = events.len() - max_events_to_show;
            event_lines.push(Line::from(Span::styled(
                format!("+{} more", more_count),
                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
            )));
        }

        f.render_widget(Paragraph::new(event_lines), events_area);
    }
}

fn render_day_view(f: &mut Frame, app: &mut App, area: Rect) {
    let events = app.get_selected_date_events();

    let title = format!(
        " Day View - {} ",
        app.selected_date.format("%Y-%m-%d (%A)")
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    // Reserve space for help text at the bottom
    let help_height = 2;
    let available_height = inner.height.saturating_sub(help_height);

    // Render hour slots
    let mut lines = Vec::new();
    
    for hour in 0..24 {
        let hour_events: Vec<&CalendarEvent> = events
            .iter()
            .copied()
            .filter(|e| {
                if let Some(event_time) = e.start_time {
                    event_time.hour() == hour
                } else {
                    false
                }
            })
            .collect();

        // Format hour
        let hour_label = format!("{:02}:00", hour);
        
        if hour_events.is_empty() {
            // Empty hour slot
            lines.push(Line::from(vec![
                Span::styled(hour_label, Style::default().fg(Color::Gray)),
                Span::raw("  "),
                Span::styled("─".repeat(inner.width.saturating_sub(8) as usize), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            // Hour with events
            for (idx, event) in hour_events.iter().enumerate() {
                // Show time range if available
                let time_str = if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                    format!("{}-{}", start.format("%H:%M"), end.format("%H:%M"))
                } else if let Some(start) = event.start_time {
                    format!("{}", start.format("%H:%M"))
                } else {
                    format!("{:02}:00", hour)
                };
                
                let prefix = if idx == 0 {
                    time_str
                } else {
                    "      ".to_string()
                };
                
                let available_width = inner.width.saturating_sub(8) as usize;
                let title = if event.title.chars().count() > available_width {
                    format!("{}…", event.title.chars().take(available_width.saturating_sub(1)).collect::<String>())
                } else {
                    event.title.clone()
                };
                
                // Choose color based on category
                let event_color = event.category.as_ref().and_then(|cat| {
                    match cat.to_lowercase().as_str() {
                        "work" => Some(Color::Cyan),
                        "personal" => Some(Color::Green),
                        "meeting" => Some(Color::Yellow),
                        "important" => Some(Color::Red),
                        _ => None,
                    }
                }).unwrap_or(Color::White);
                
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::raw("  "),
                    Span::styled(title, Style::default().fg(event_color).add_modifier(Modifier::BOLD)),
                ]));
            }
        }
    }

    // Add all-day events and multi-day events at the top
    let all_day_events: Vec<&CalendarEvent> = events
        .iter()
        .copied()
        .filter(|e| e.start_time.is_none())
        .collect();
    
    if !all_day_events.is_empty() {
        let mut all_day_lines = vec![
            Line::from(Span::styled("All Day", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        ];
        
        for event in all_day_events {
            let available_width = inner.width.saturating_sub(4) as usize;
            let title = if event.title.chars().count() > available_width {
                format!("{}…", event.title.chars().take(available_width.saturating_sub(1)).collect::<String>())
            } else {
                event.title.clone()
            };
            
            // Show date range for multi-day events
            let date_info = if let Some(end_date) = event.end_date {
                if end_date != event.start_date {
                    format!(" ({} - {})", event.start_date.format("%m/%d"), end_date.format("%m/%d"))
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            // Choose color based on category
            let event_color = event.category.as_ref().and_then(|cat| {
                match cat.to_lowercase().as_str() {
                    "work" => Some(Color::Cyan),
                    "personal" => Some(Color::Green),
                    "meeting" => Some(Color::Yellow),
                    "important" => Some(Color::Red),
                    _ => None,
                }
            }).unwrap_or(Color::White);
            
            all_day_lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(title, Style::default().fg(event_color).add_modifier(Modifier::BOLD)),
                Span::styled(date_info, Style::default().fg(Color::Gray)),
            ]));
        }
        
        all_day_lines.push(Line::from(""));
        
        // Prepend all-day events
        lines.splice(0..0, all_day_lines);
    }

    // If no events at all, show a message
    if events.is_empty() {
        lines.clear();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "No events for this day",
            Style::default().fg(Color::Gray)
        )));
    }

    // Render the day view
    let day_view_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: available_height,
    };
    
    let paragraph = Paragraph::new(lines)
        .scroll((0, 0));
    f.render_widget(paragraph, day_view_area);

    // Render help text at bottom
    let help_area = Rect {
        x: inner.x,
        y: inner.y + available_height,
        width: inner.width,
        height: help_height,
    };

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Ctrl+N", Style::default().fg(Color::Yellow)),
            Span::raw(": New Event  "),
            Span::styled("Ctrl+T", Style::default().fg(Color::Yellow)),
            Span::raw(": Today"),
        ]),
    ];
    f.render_widget(Paragraph::new(help_text), help_area);
}

// Helper function to render cursor for input fields
fn render_input_cursor(f: &mut Frame, input: &Input, area: Rect, y_offset: u16) {
    if area.width == 0 {
        return;
    }
    let cursor_pos = input.visual_cursor().min(area.width.saturating_sub(1) as usize);
    f.set_cursor_position((area.x + cursor_pos as u16, area.y + y_offset));
}

fn render_create_event_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, f.area());

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
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Description
            Constraint::Length(3),  // Start Date
            Constraint::Length(3),  // End Date
            Constraint::Length(3),  // Start Time
            Constraint::Length(3),  // End Time
            Constraint::Length(3),  // Category
            Constraint::Min(1),     // Help
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
    
    if matches!(app.create_event_field, CreateEventField::Title) {
        render_input_cursor(f, &app.new_event_title, chunks[0], 1);
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
    
    if matches!(app.create_event_field, CreateEventField::Description) {
        render_input_cursor(f, &app.new_event_description, chunks[1], 1);
    }

    // Start Date field
    let start_date_style = if matches!(app.create_event_field, CreateEventField::StartDate) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let start_date_text = vec![
        Line::from(Span::styled("Start Date (YYYY-MM-DD):", start_date_style)),
        Line::from(app.new_event_start_date.value()),
    ];
    let start_date_para = Paragraph::new(start_date_text);
    f.render_widget(start_date_para, chunks[2]);
    
    if matches!(app.create_event_field, CreateEventField::StartDate) {
        render_input_cursor(f, &app.new_event_start_date, chunks[2], 1);
    }

    // End Date field
    let end_date_style = if matches!(app.create_event_field, CreateEventField::EndDate) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let end_date_text = vec![
        Line::from(Span::styled("End Date (YYYY-MM-DD, optional for multi-day):", end_date_style)),
        Line::from(app.new_event_end_date.value()),
    ];
    let end_date_para = Paragraph::new(end_date_text);
    f.render_widget(end_date_para, chunks[3]);
    
    if matches!(app.create_event_field, CreateEventField::EndDate) {
        render_input_cursor(f, &app.new_event_end_date, chunks[3], 1);
    }

    // Start Time field
    let start_time_style = if matches!(app.create_event_field, CreateEventField::StartTime) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let start_time_text = vec![
        Line::from(Span::styled("Start Time (HH:MM, optional):", start_time_style)),
        Line::from(app.new_event_start_time.value()),
    ];
    let start_time_para = Paragraph::new(start_time_text);
    f.render_widget(start_time_para, chunks[4]);
    
    if matches!(app.create_event_field, CreateEventField::StartTime) {
        render_input_cursor(f, &app.new_event_start_time, chunks[4], 1);
    }

    // End Time field
    let end_time_style = if matches!(app.create_event_field, CreateEventField::EndTime) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let end_time_text = vec![
        Line::from(Span::styled("End Time (HH:MM, optional):", end_time_style)),
        Line::from(app.new_event_end_time.value()),
    ];
    let end_time_para = Paragraph::new(end_time_text);
    f.render_widget(end_time_para, chunks[5]);
    
    if matches!(app.create_event_field, CreateEventField::EndTime) {
        render_input_cursor(f, &app.new_event_end_time, chunks[5], 1);
    }

    // Category field
    let category_style = if matches!(app.create_event_field, CreateEventField::Category) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let category_text = vec![
        Line::from(Span::styled("Category (work/personal/meeting/important):", category_style)),
        Line::from(app.new_event_category.value()),
    ];
    let category_para = Paragraph::new(category_text);
    f.render_widget(category_para, chunks[6]);
    
    if matches!(app.create_event_field, CreateEventField::Category) {
        render_input_cursor(f, &app.new_event_category, chunks[6], 1);
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
    f.render_widget(Paragraph::new(help_text), chunks[7]);
}

fn render_edit_event_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 70, f.area());

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
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Description
            Constraint::Length(3),  // Start Date
            Constraint::Length(3),  // End Date
            Constraint::Length(3),  // Start Time
            Constraint::Length(3),  // End Time
            Constraint::Length(3),  // Category
            Constraint::Min(1),     // Help
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
    
    if matches!(app.edit_event_field, CreateEventField::Title) {
        render_input_cursor(f, &app.edit_event_title, chunks[0], 1);
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
    
    if matches!(app.edit_event_field, CreateEventField::Description) {
        render_input_cursor(f, &app.edit_event_description, chunks[1], 1);
    }

    // Start Date field
    let start_date_style = if matches!(app.edit_event_field, CreateEventField::StartDate) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let start_date_text = vec![
        Line::from(Span::styled("Start Date (YYYY-MM-DD):", start_date_style)),
        Line::from(app.edit_event_start_date.value()),
    ];
    let start_date_para = Paragraph::new(start_date_text);
    f.render_widget(start_date_para, chunks[2]);
    
    if matches!(app.edit_event_field, CreateEventField::StartDate) {
        render_input_cursor(f, &app.edit_event_start_date, chunks[2], 1);
    }

    // End Date field
    let end_date_style = if matches!(app.edit_event_field, CreateEventField::EndDate) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let end_date_text = vec![
        Line::from(Span::styled("End Date (YYYY-MM-DD, optional for multi-day):", end_date_style)),
        Line::from(app.edit_event_end_date.value()),
    ];
    let end_date_para = Paragraph::new(end_date_text);
    f.render_widget(end_date_para, chunks[3]);
    
    if matches!(app.edit_event_field, CreateEventField::EndDate) {
        render_input_cursor(f, &app.edit_event_end_date, chunks[3], 1);
    }

    // Start Time field
    let start_time_style = if matches!(app.edit_event_field, CreateEventField::StartTime) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let start_time_text = vec![
        Line::from(Span::styled("Start Time (HH:MM, optional):", start_time_style)),
        Line::from(app.edit_event_start_time.value()),
    ];
    let start_time_para = Paragraph::new(start_time_text);
    f.render_widget(start_time_para, chunks[4]);
    
    if matches!(app.edit_event_field, CreateEventField::StartTime) {
        render_input_cursor(f, &app.edit_event_start_time, chunks[4], 1);
    }

    // End Time field
    let end_time_style = if matches!(app.edit_event_field, CreateEventField::EndTime) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let end_time_text = vec![
        Line::from(Span::styled("End Time (HH:MM, optional):", end_time_style)),
        Line::from(app.edit_event_end_time.value()),
    ];
    let end_time_para = Paragraph::new(end_time_text);
    f.render_widget(end_time_para, chunks[5]);
    
    if matches!(app.edit_event_field, CreateEventField::EndTime) {
        render_input_cursor(f, &app.edit_event_end_time, chunks[5], 1);
    }

    // Category field
    let category_style = if matches!(app.edit_event_field, CreateEventField::Category) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let category_text = vec![
        Line::from(Span::styled("Category (work/personal/meeting/important):", category_style)),
        Line::from(app.edit_event_category.value()),
    ];
    let category_para = Paragraph::new(category_text);
    f.render_widget(category_para, chunks[6]);
    
    if matches!(app.edit_event_field, CreateEventField::Category) {
        render_input_cursor(f, &app.edit_event_category, chunks[6], 1);
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
    f.render_widget(Paragraph::new(help_text), chunks[7]);
}

fn render_view_event_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.area());

    if let Some(event_id) = app.selected_event_index {
        if let Some(event) = app.events.iter().find(|e| e.id == event_id) {
            let block = Block::default()
                .title(" Event Details ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));

            f.render_widget(Clear, area);
            f.render_widget(block.clone(), area);

            let inner = block.inner(area);

            // Build time string
            let time_str = if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                format!("Time: {} - {}\n", start.format("%H:%M"), end.format("%H:%M"))
            } else if let Some(start) = event.start_time {
                format!("Time: {}\n", start.format("%H:%M"))
            } else {
                String::new()
            };

            // Build date string
            let date_str = if let Some(end_date) = event.end_date {
                if end_date != event.start_date {
                    format!("Dates: {} to {}\n", event.start_date.format("%Y-%m-%d"), end_date.format("%Y-%m-%d"))
                } else {
                    format!("Date: {}\n", event.start_date.format("%Y-%m-%d"))
                }
            } else {
                format!("Date: {}\n", event.start_date.format("%Y-%m-%d"))
            };

            // Build category string
            let category_str = event.category.as_ref()
                .map(|c| format!("Category: {}\n", c))
                .unwrap_or_default();

            let content = format!(
                "Title: {}\n\n{}{}{}\nDescription:\n{}",
                event.title,
                date_str,
                time_str,
                category_str,
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

fn render_week_view(f: &mut Frame, app: &App, area: Rect) {
    let week_dates = app.get_week_dates();
    
    // Get week range for title
    let week_start = week_dates.first().unwrap();
    let week_end = week_dates.last().unwrap();
    
    let title = format!(
        " Week View - {} to {} ",
        week_start.format("%Y-%m-%d"),
        week_end.format("%Y-%m-%d")
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 10 || inner.width < 40 {
        return;
    }

    // Create layout for 7 days
    let day_width = inner.width / 7;
    let header_height = 2;
    let help_height = 2;
    
    // Render day headers
    let mut header_x = inner.x;
    let weekday_labels = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    
    for (idx, date) in week_dates.iter().enumerate() {
        let is_today = *date == app.current_date;
        let is_selected = *date == app.selected_date;
        
        let header_area = Rect {
            x: header_x,
            y: inner.y,
            width: day_width.min(inner.x + inner.width - header_x),
            height: header_height,
        };
        
        let day_label = format!("{}", weekday_labels[idx]);
        let date_label = format!("{}", date.format("%m/%d"));
        
        let header_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_today {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        let header_text = vec![
            Line::from(Span::styled(&day_label[0..3], header_style)),
            Line::from(Span::styled(date_label, header_style)),
        ];
        
        f.render_widget(
            Paragraph::new(header_text).alignment(Alignment::Center),
            header_area
        );
        
        header_x += day_width;
    }
    
    // Render events for each day
    let events_area_height = inner.height.saturating_sub(header_height + help_height);
    let mut day_x = inner.x;
    
    for date in week_dates.iter() {
        let events = app.get_events_for_date(*date);
        let is_selected = *date == app.selected_date;
        
        let day_area = Rect {
            x: day_x,
            y: inner.y + header_height,
            width: day_width.min(inner.x + inner.width - day_x),
            height: events_area_height,
        };
        
        // Draw border for this day
        let border_style = if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let day_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(border_style);
        
        let day_inner = day_block.inner(day_area);
        f.render_widget(day_block, day_area);
        
        // Render events for this day
        if !events.is_empty() {
            let mut event_lines = Vec::new();
            let max_events = (day_inner.height as usize).min(events.len());
            
            for event in events.iter().take(max_events) {
                let time_str = if let (Some(start), Some(end)) = (event.start_time, event.end_time) {
                    format!("{}-{}", start.format("%H:%M"), end.format("%H:%M"))
                } else if let Some(start) = event.start_time {
                    format!("{}", start.format("%H:%M"))
                } else {
                    "All Day".to_string()
                };
                
                let available_width = day_inner.width.saturating_sub(2) as usize;
                let title = if event.title.chars().count() > available_width {
                    format!("{}…", event.title.chars().take(available_width.saturating_sub(1)).collect::<String>())
                } else {
                    event.title.clone()
                };
                
                // Choose color based on category
                let event_color = event.category.as_ref().and_then(|cat| {
                    match cat.to_lowercase().as_str() {
                        "work" => Some(Color::Cyan),
                        "personal" => Some(Color::Green),
                        "meeting" => Some(Color::Yellow),
                        "important" => Some(Color::Red),
                        _ => None,
                    }
                }).unwrap_or(Color::White);
                
                event_lines.push(Line::from(Span::styled(
                    time_str,
                    Style::default().fg(Color::Gray)
                )));
                event_lines.push(Line::from(Span::styled(
                    title,
                    Style::default().fg(event_color).add_modifier(Modifier::BOLD)
                )));
            }
            
            if events.len() > max_events {
                let more_count = events.len() - max_events;
                event_lines.push(Line::from(Span::styled(
                    format!("+{} more", more_count),
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
                )));
            }
            
            f.render_widget(Paragraph::new(event_lines), day_inner);
        }
        
        day_x += day_width;
    }
    
    // Render help text
    let help_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(help_height),
        width: inner.width,
        height: help_height,
    };

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("W", Style::default().fg(Color::Yellow)),
            Span::raw(": Toggle Week View  "),
            Span::styled("Ctrl+N", Style::default().fg(Color::Yellow)),
            Span::raw(": New Event  "),
            Span::styled("←/→", Style::default().fg(Color::Yellow)),
            Span::raw(": Prev/Next Week  "),
            Span::styled("Q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit"),
        ]),
    ];
    f.render_widget(Paragraph::new(help_text), help_area);
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
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.selected_date = app.current_date;
                        app.number_buffer.clear();
                    }
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        app.toggle_week_view();
                        app.number_buffer.clear();
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        // Build up number buffer for vim-style numeric prefixes
                        if app.number_buffer.len() < MAX_BUFFER_LEN {
                            app.number_buffer.push(c);
                        }
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        // Reserved for future use
                        app.number_buffer.clear();
                    }
                    KeyCode::Left => {
                        let count = app.get_count();
                        app.move_selection_left_by(count);
                        app.number_buffer.clear();
                    }
                    KeyCode::Right => {
                        let count = app.get_count();
                        app.move_selection_right_by(count);
                        app.number_buffer.clear();
                    }
                    KeyCode::Up => {
                        let count = app.get_count();
                        app.move_selection_up_by(count);
                        app.number_buffer.clear();
                    }
                    KeyCode::Down => {
                        let count = app.get_count();
                        app.move_selection_down_by(count);
                        app.number_buffer.clear();
                    }
                    KeyCode::Tab => {
                        // Tab can be used for future features or removed
                        app.number_buffer.clear();
                    }
                    KeyCode::Enter => {
                        // Enter can be used for future features or removed
                        app.number_buffer.clear();
                    }
                    KeyCode::Delete => {
                        // Reserved for future use
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
                AppMode::WeekView => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Ok(());
                    }
                    KeyCode::Char('w') | KeyCode::Char('W') => {
                        app.toggle_week_view();
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.start_create_event();
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.selected_date = app.current_date;
                    }
                    KeyCode::Left => {
                        // Move to previous week
                        if let Some(new_date) = app.selected_date.checked_sub_days(chrono::Days::new(7)) {
                            app.selected_date = new_date;
                        }
                    }
                    KeyCode::Right => {
                        // Move to next week
                        if let Some(new_date) = app.selected_date.checked_add_days(chrono::Days::new(7)) {
                            app.selected_date = new_date;
                        }
                    }
                    KeyCode::Esc => {
                        app.toggle_week_view();
                    }
                    _ => {}
                },
                AppMode::CreateEvent => {
                    let input_event = Event::Key(key);
                    match key.code {
                        KeyCode::Tab => {
                            app.create_event_field = match app.create_event_field {
                                CreateEventField::Title => CreateEventField::Description,
                                CreateEventField::Description => CreateEventField::StartDate,
                                CreateEventField::StartDate => CreateEventField::EndDate,
                                CreateEventField::EndDate => CreateEventField::StartTime,
                                CreateEventField::StartTime => CreateEventField::EndTime,
                                CreateEventField::EndTime => CreateEventField::Category,
                                CreateEventField::Category => CreateEventField::Title,
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
                                CreateEventField::Description => CreateEventField::StartDate,
                                CreateEventField::StartDate => CreateEventField::EndDate,
                                CreateEventField::EndDate => CreateEventField::StartTime,
                                CreateEventField::StartTime => CreateEventField::EndTime,
                                CreateEventField::EndTime => CreateEventField::Category,
                                CreateEventField::Category => CreateEventField::Title,
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
