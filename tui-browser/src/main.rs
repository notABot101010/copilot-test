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

#[derive(Debug, Clone)]
struct Link {
    text: String,
    url: String,
    line_index: usize,
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
    help_scroll_offset: usize,
    status_message: String,
    link_navigation_mode: bool,
    current_links: Vec<Link>,
    selected_link_index: Option<usize>,
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
            help_scroll_offset: 0,
            status_message: "Welcome to TUI Browser! Press Ctrl+H for help.".to_string(),
            link_navigation_mode: false,
            current_links: Vec::new(),
            selected_link_index: None,
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
                
                // Extract links from HTML
                let links = Self::extract_links(&html, &text_content);
                
                let tab = self.current_tab_mut();
                tab.content = text_content;
                tab.loading = false;
                tab.scroll_offset = 0;
                tab.title = title.clone();
                
                // Update current links
                self.current_links = links;
                self.link_navigation_mode = false;
                self.selected_link_index = None;
                
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
                self.current_links.clear();
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

    fn extract_href_from_link(link_html: &str) -> Option<String> {
        let href_pattern_double = "href=\"";
        let href_pattern_single = "href='";
        
        let (href_start, quote_char) = if let Some(pos) = link_html.to_lowercase().find(href_pattern_double) {
            (Some(pos + href_pattern_double.len()), '"')
        } else if let Some(pos) = link_html.to_lowercase().find(href_pattern_single) {
            (Some(pos + href_pattern_single.len()), '\'')
        } else {
            (None, '"')
        };
        
        if let Some(href_value_start) = href_start {
            if let Some(href_end) = link_html[href_value_start..].find(quote_char) {
                return Some(link_html[href_value_start..href_value_start + href_end].to_string());
            }
        }
        None
    }

    fn extract_text_from_link(link_html: &str) -> Option<String> {
        if let Some(text_start) = link_html.find('>') {
            let text_end = link_html.len() - 4; // remove </a>
            let raw_text = &link_html[text_start + 1..text_end];
            // Simple HTML tag stripping
            let mut text = String::new();
            let mut in_tag = false;
            for ch in raw_text.chars() {
                if ch == '<' {
                    in_tag = true;
                } else if ch == '>' {
                    in_tag = false;
                } else if !in_tag {
                    text.push(ch);
                }
            }
            let text = text.trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }

    fn find_line_index_for_text(text: &str, content_lines: &[&str]) -> usize {
        let text_words: Vec<String> = text.split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        
        let mut line_index = 0;
        let mut best_match = 0;
        
        for (idx, line) in content_lines.iter().enumerate() {
            // Strategy 1: Exact match
            if line.contains(text) {
                return idx;
            }
            
            // Strategy 2: Word-based matching with pre-computed lowercase words
            let line_lower = line.to_lowercase();
            let matches = text_words.iter()
                .filter(|word| line_lower.contains(word.as_str()))
                .count();
            
            if matches > best_match {
                best_match = matches;
                line_index = idx;
            }
        }
        
        line_index
    }

    fn extract_links(html: &str, content: &str) -> Vec<Link> {
        let mut links = Vec::new();
        let lower = html.to_lowercase();
        let content_lines: Vec<&str> = content.lines().collect();
        
        let mut pos = 0;
        while let Some(start_pos) = lower[pos..].find("<a") {
            let abs_start = pos + start_pos;
            // Check if it's actually an anchor tag (followed by space, >, newline, or tab)
            let next_char_pos = abs_start + 2;
            if next_char_pos < html.len() {
                let html_bytes = html.as_bytes();
                if next_char_pos < html_bytes.len() {
                    let next_byte = html_bytes[next_char_pos];
                    if !matches!(next_byte, b' ' | b'>' | b'\n' | b'\t') {
                        pos = abs_start + 2;
                        continue;
                    }
                }
            }
            
            if let Some(end) = lower[abs_start..].find("</a>") {
                let abs_end = abs_start + end;
                let link_html = &html[abs_start..abs_end + 4];
                
                if let Some(url) = Self::extract_href_from_link(link_html) {
                    if let Some(text) = Self::extract_text_from_link(link_html) {
                        let line_index = Self::find_line_index_for_text(&text, &content_lines);
                        
                        if !url.is_empty() {
                            links.push(Link {
                                text,
                                url,
                                line_index,
                            });
                        }
                    }
                }
                
                pos = abs_end + 4;
            } else {
                break;
            }
        }
        
        links
    }

    fn enter_link_navigation_mode(&mut self) {
        if !self.current_links.is_empty() {
            self.link_navigation_mode = true;
            self.selected_link_index = Some(0);
            self.status_message = format!(
                "Link navigation mode: {}/{} links. Use ↑/↓ to navigate, Enter to open, Esc to exit.",
                1,
                self.current_links.len()
            );
        } else {
            self.status_message = "No links found on this page".to_string();
        }
    }

    fn exit_link_navigation_mode(&mut self) {
        self.link_navigation_mode = false;
        self.selected_link_index = None;
        self.status_message = "Exited link navigation mode".to_string();
    }

    fn next_link(&mut self) {
        if let Some(idx) = self.selected_link_index {
            let new_idx = (idx + 1) % self.current_links.len();
            self.selected_link_index = Some(new_idx);
            self.status_message = format!(
                "Link {}/{}: {}",
                new_idx + 1,
                self.current_links.len(),
                self.current_links[new_idx].text
            );
        }
    }

    fn previous_link(&mut self) {
        if let Some(idx) = self.selected_link_index {
            let new_idx = if idx == 0 {
                self.current_links.len() - 1
            } else {
                idx - 1
            };
            self.selected_link_index = Some(new_idx);
            self.status_message = format!(
                "Link {}/{}: {}",
                new_idx + 1,
                self.current_links.len(),
                self.current_links[new_idx].text
            );
        }
    }

    fn open_selected_link(&mut self) {
        if let Some(idx) = self.selected_link_index {
            if let Some(link) = self.current_links.get(idx) {
                let mut url = link.url.clone();
                
                // Handle relative URLs
                let current_url = &self.current_tab().url;
                if url.starts_with('/') {
                    // Absolute path - need to construct full URL
                    if let Ok(parsed) = url::Url::parse(current_url) {
                        if let Some(host) = parsed.host_str() {
                            let scheme = parsed.scheme();
                            url = format!("{}://{}{}", scheme, host, url);
                        }
                    }
                } else if !url.starts_with("http://") && !url.starts_with("https://") {
                    // Relative path
                    if let Ok(parsed) = url::Url::parse(current_url) {
                        if let Ok(joined) = parsed.join(&url) {
                            url = joined.to_string();
                        }
                    }
                }
                
                self.url_input = url;
                self.exit_link_navigation_mode();
                self.navigate_to_url();
            }
        }
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
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.show_help = false;
                    self.help_scroll_offset = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(1);
                }
                KeyCode::PageUp => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(PAGE_SCROLL_STEP);
                }
                KeyCode::PageDown => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(PAGE_SCROLL_STEP);
                }
                _ => {}
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
                // Handle link navigation mode
                if self.link_navigation_mode {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => self.previous_link(),
                        KeyCode::Down | KeyCode::Char('j') => self.next_link(),
                        KeyCode::PageUp => self.scroll_content_page_up(),
                        KeyCode::PageDown => self.scroll_content_page_down(),
                        KeyCode::Enter => self.open_selected_link(),
                        KeyCode::Esc => self.exit_link_navigation_mode(),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => self.scroll_content_up(),
                        KeyCode::Down | KeyCode::Char('j') => self.scroll_content_down(),
                        KeyCode::PageUp => self.scroll_content_page_up(),
                        KeyCode::PageDown => self.scroll_content_page_down(),
                        KeyCode::Enter => self.enter_link_navigation_mode(),
                        KeyCode::Backspace => self.go_back(),
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
            let selected_link_line = if app.link_navigation_mode {
                app.selected_link_index.and_then(|idx| app.current_links.get(idx).map(|link| link.line_index))
            } else {
                None
            };
            ContentArea::render(
                chunks[3],
                f.buffer_mut(),
                &tab.content,
                tab.scroll_offset,
                app.focused_panel == FocusPanel::Content,
                selected_link_line,
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
                HelpDialog::render(f.area(), f.buffer_mut(), app.help_scroll_offset);
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
