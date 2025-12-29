mod http_client;
mod models;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use http_client::HttpClient;
use models::{Bookmark, HistoryEntry, NavigationHistory, Tab};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io;
use ui::{ContentArea, FavoritesBar, HelpDialog, StatusBar, TabBar, UrlBar};

const SCROLL_STEP: usize = 1;
const PAGE_SCROLL_STEP: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPanel {
    TabBar,
    UrlBar,
    FavoritesBar,
    Content,
}

struct App {
    tabs: Vec<Tab>,
    current_tab_index: usize,
    bookmarks: Vec<Bookmark>,
    selected_bookmark_index: Option<usize>,
    focused_panel: FocusPanel,
    url_input: String,
    url_cursor_position: usize,
    should_quit: bool,
    http_client: HttpClient,
    history: NavigationHistory,
    show_help: bool,
    status_message: String,
}

impl App {
    fn new() -> Result<Self> {
        let mut tabs = Vec::new();
        tabs.push(Tab::new());
        
        Ok(Self {
            tabs,
            current_tab_index: 0,
            bookmarks: Vec::new(),
            selected_bookmark_index: None,
            focused_panel: FocusPanel::UrlBar,
            url_input: String::new(),
            url_cursor_position: 0,
            should_quit: false,
            http_client: HttpClient::new()?,
            history: NavigationHistory::new(),
            show_help: false,
            status_message: "Welcome to TUI Browser! Press Ctrl+H for help.".to_string(),
        })
    }

    fn current_tab(&self) -> &Tab {
        &self.tabs[self.current_tab_index]
    }

    fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.current_tab_index]
    }

    fn open_new_tab(&mut self) {
        self.tabs.push(Tab::new());
        self.current_tab_index = self.tabs.len() - 1;
        self.url_input.clear();
        self.url_cursor_position = 0;
        self.focused_panel = FocusPanel::UrlBar;
        self.status_message = format!("Opened new tab ({})", self.tabs.len());
    }

    fn close_current_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.current_tab_index);
            if self.current_tab_index >= self.tabs.len() {
                self.current_tab_index = self.tabs.len() - 1;
            }
            self.status_message = format!("Closed tab. {} tab(s) remaining", self.tabs.len());
        } else {
            self.status_message = "Cannot close the last tab".to_string();
        }
    }

    fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.current_tab_index = (self.current_tab_index + 1) % self.tabs.len();
            self.update_url_bar_from_current_tab();
        }
    }

    fn previous_tab(&mut self) {
        if self.tabs.len() > 1 {
            if self.current_tab_index == 0 {
                self.current_tab_index = self.tabs.len() - 1;
            } else {
                self.current_tab_index -= 1;
            }
            self.update_url_bar_from_current_tab();
        }
    }

    fn update_url_bar_from_current_tab(&mut self) {
        self.url_input = self.current_tab().url.clone();
        self.url_cursor_position = self.url_input.len();
    }

    fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusPanel::TabBar => FocusPanel::UrlBar,
            FocusPanel::UrlBar => FocusPanel::FavoritesBar,
            FocusPanel::FavoritesBar => FocusPanel::Content,
            FocusPanel::Content => FocusPanel::TabBar,
        };
    }

    fn add_bookmark(&mut self) {
        let tab = self.current_tab();
        if !tab.url.is_empty() {
            let title = tab.title.clone();
            let url = tab.url.clone();
            let bookmark = Bookmark::new(title.clone(), url);
            self.bookmarks.push(bookmark);
            self.status_message = format!("Added '{}' to favorites", title);
        } else {
            self.status_message = "No page loaded to bookmark".to_string();
        }
    }

    fn next_bookmark(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }

        self.selected_bookmark_index = Some(
            self.selected_bookmark_index
                .map(|i| (i + 1) % self.bookmarks.len())
                .unwrap_or(0),
        );
    }

    fn previous_bookmark(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }

        self.selected_bookmark_index = Some(
            self.selected_bookmark_index
                .map(|i| {
                    if i == 0 {
                        self.bookmarks.len() - 1
                    } else {
                        i - 1
                    }
                })
                .unwrap_or(0),
        );
    }

    fn open_selected_bookmark(&mut self) {
        if let Some(idx) = self.selected_bookmark_index {
            if let Some(bookmark) = self.bookmarks.get(idx) {
                self.url_input = bookmark.url.clone();
                self.navigate_to_url();
            }
        }
    }

    fn navigate_to_url(&mut self) {
        if self.url_input.trim().is_empty() {
            self.status_message = "Please enter a URL".to_string();
            return;
        }

        let url = self.url_input.trim().to_string();
        
        // Add http:// if no protocol specified
        let url = if !url.starts_with("http://") && !url.starts_with("https://") {
            format!("https://{}", url)
        } else {
            url
        };

        self.status_message = format!("Loading {}...", url);
        
        let tab = self.current_tab_mut();
        tab.url = url.clone();
        tab.loading = true;
        tab.title = "Loading...".to_string();

        // Fetch the page
        match self.http_client.fetch_page(&url) {
            Ok(html) => {
                let text_content = self.http_client.render_html_to_text(&html);
                
                // Extract title from HTML (simple approach)
                let title = Self::extract_title(&html).unwrap_or_else(|| url.clone());
                
                let tab = self.current_tab_mut();
                tab.content = text_content;
                tab.loading = false;
                tab.scroll_offset = 0;
                tab.title = title.clone();
                
                // Add to history
                let entry = HistoryEntry::new(url.clone(), title.clone());
                self.history.add_entry(entry);
                
                self.status_message = format!("Loaded: {}", title);
            }
            Err(err) => {
                let tab = self.current_tab_mut();
                tab.loading = false;
                tab.content = format!("Error loading page:\n\n{}", err);
                tab.title = "Error".to_string();
                self.status_message = format!("Error: {}", err);
            }
        }
    }

    fn extract_title(html: &str) -> Option<String> {
        // Simple title extraction
        let lower = html.to_lowercase();
        if let Some(start) = lower.find("<title>") {
            if let Some(end) = lower[start..].find("</title>") {
                let title_start = start + 7;
                let title_end = start + end;
                return Some(html[title_start..title_end].trim().to_string());
            }
        }
        None
    }

    fn go_back(&mut self) {
        if !self.history.can_go_back() {
            self.status_message = "No previous page in history".to_string();
            return;
        }
        
        if let Some(entry) = self.history.go_back() {
            let url = entry.url.clone();
            let title = entry.title.clone();
            self.url_input = url;
            self.navigate_to_url();
            self.status_message = format!("Back: {}", title);
        }
    }

    fn go_forward(&mut self) {
        if !self.history.can_go_forward() {
            self.status_message = "No next page in history".to_string();
            return;
        }
        
        if let Some(entry) = self.history.go_forward() {
            let url = entry.url.clone();
            let title = entry.title.clone();
            self.url_input = url;
            self.navigate_to_url();
            self.status_message = format!("Forward: {}", title);
        }
    }

    fn scroll_content_up(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_sub(SCROLL_STEP);
    }

    fn scroll_content_down(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_add(SCROLL_STEP);
    }

    fn scroll_content_page_up(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_sub(PAGE_SCROLL_STEP);
    }

    fn scroll_content_page_down(&mut self) {
        let tab = self.current_tab_mut();
        tab.scroll_offset = tab.scroll_offset.saturating_add(PAGE_SCROLL_STEP);
    }

    fn handle_key_event(&mut self, key: event::KeyEvent) {
        // Help dialog handling
        if self.show_help {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                self.show_help = false;
            }
            return;
        }

        // Global shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('t') => {
                    self.open_new_tab();
                    return;
                }
                KeyCode::Char('w') => {
                    self.close_current_tab();
                    return;
                }
                KeyCode::Char('f') => {
                    self.add_bookmark();
                    return;
                }
                KeyCode::Char('l') => {
                    self.focused_panel = FocusPanel::UrlBar;
                    self.update_url_bar_from_current_tab();
                    return;
                }
                KeyCode::Char('h') => {
                    self.show_help = !self.show_help;
                    return;
                }
                KeyCode::Left => {
                    self.go_back();
                    return;
                }
                KeyCode::Right => {
                    self.go_forward();
                    return;
                }
                _ => {}
            }
        }

        // Panel-specific handling
        match self.focused_panel {
            FocusPanel::TabBar => {
                match key.code {
                    KeyCode::Left => self.previous_tab(),
                    KeyCode::Right => self.next_tab(),
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }
            FocusPanel::UrlBar => {
                match key.code {
                    KeyCode::Enter => {
                        self.navigate_to_url();
                        self.focused_panel = FocusPanel::Content;
                    }
                    KeyCode::Char(c) => {
                        self.url_input.insert(self.url_cursor_position, c);
                        self.url_cursor_position += 1;
                    }
                    KeyCode::Backspace => {
                        if self.url_cursor_position > 0 {
                            self.url_cursor_position -= 1;
                            self.url_input.remove(self.url_cursor_position);
                        }
                    }
                    KeyCode::Delete => {
                        if self.url_cursor_position < self.url_input.len() {
                            self.url_input.remove(self.url_cursor_position);
                        }
                    }
                    KeyCode::Left => {
                        self.url_cursor_position = self.url_cursor_position.saturating_sub(1);
                    }
                    KeyCode::Right => {
                        self.url_cursor_position = (self.url_cursor_position + 1).min(self.url_input.len());
                    }
                    KeyCode::Home => {
                        self.url_cursor_position = 0;
                    }
                    KeyCode::End => {
                        self.url_cursor_position = self.url_input.len();
                    }
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Esc => {
                        self.focused_panel = FocusPanel::Content;
                    }
                    _ => {}
                }
            }
            FocusPanel::FavoritesBar => {
                match key.code {
                    KeyCode::Left => self.previous_bookmark(),
                    KeyCode::Right => self.next_bookmark(),
                    KeyCode::Enter => {
                        self.open_selected_bookmark();
                        self.focused_panel = FocusPanel::Content;
                    }
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Esc => {
                        self.focused_panel = FocusPanel::Content;
                    }
                    _ => {}
                }
            }
            FocusPanel::Content => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => self.scroll_content_up(),
                    KeyCode::Down | KeyCode::Char('j') => self.scroll_content_down(),
                    KeyCode::PageUp => self.scroll_content_page_up(),
                    KeyCode::PageDown => self.scroll_content_page_down(),
                    KeyCode::Tab => self.cycle_focus(),
                    KeyCode::Char('q') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.should_quit = true;
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
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Tab bar
                    Constraint::Length(3), // URL bar
                    Constraint::Length(3), // Favorites bar
                    Constraint::Min(5),    // Content area
                    Constraint::Length(3), // Status bar
                ])
                .split(f.area());

            // Render tab bar
            TabBar::render(
                chunks[0],
                f.buffer_mut(),
                &app.tabs,
                app.current_tab_index,
                app.focused_panel == FocusPanel::TabBar,
            );

            // Render URL bar
            UrlBar::render(
                chunks[1],
                f.buffer_mut(),
                &app.url_input,
                app.url_cursor_position,
                app.focused_panel == FocusPanel::UrlBar,
            );

            // Render favorites bar
            FavoritesBar::render(
                chunks[2],
                f.buffer_mut(),
                &app.bookmarks,
                app.selected_bookmark_index,
                app.focused_panel == FocusPanel::FavoritesBar,
            );

            // Render content area
            let tab = app.current_tab();
            ContentArea::render(
                chunks[3],
                f.buffer_mut(),
                &tab.content,
                tab.scroll_offset,
                app.focused_panel == FocusPanel::Content,
            );

            // Render status bar
            let help_text = "Ctrl+H: Help | Ctrl+Q: Quit";
            StatusBar::render(
                chunks[4],
                f.buffer_mut(),
                &app.status_message,
                help_text,
            );

            // Render help dialog if shown
            if app.show_help {
                HelpDialog::render(f.area(), f.buffer_mut());
            }
        })?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            app.handle_key_event(key);
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
    let app = App::new()?;
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
