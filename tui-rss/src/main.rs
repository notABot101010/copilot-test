mod feed_manager;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use feed_manager::{Article, FeedManager};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io;
use tui_input::Input;
use ui::{ArticleList, ArticleReader, FeedList, HelpOverlay, SearchModal, StatusBar};

const SCROLL_STEP: usize = 3;
const PAGE_SCROLL_STEP: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    ArticleList,
    ArticleReader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    AddFeed,
    Search,
    FeedManagement,
    Help,
}

struct App {
    feed_manager: FeedManager,
    articles: Vec<Article>,
    selected_article: Option<usize>,
    article_scroll_offset: usize,
    list_scroll_offset: usize,
    active_panel: Panel,
    mode: Mode,
    should_quit: bool,
    focus_mode: bool,
    input: Input,
    search_query: String,
    status_message: String,
    show_only_unread: bool,
    selected_feed: Option<usize>,
}

impl App {
    fn new() -> Result<Self> {
        let feed_manager = FeedManager::load()?;
        let articles = feed_manager.get_all_articles();
        
        Ok(Self {
            feed_manager,
            articles,
            selected_article: None,
            article_scroll_offset: 0,
            list_scroll_offset: 0,
            active_panel: Panel::ArticleList,
            mode: Mode::Normal,
            should_quit: false,
            focus_mode: false,
            input: Input::default(),
            search_query: String::new(),
            status_message: String::new(),
            show_only_unread: false,
            selected_feed: None,
        })
    }

    fn refresh_articles(&mut self) {
        self.articles = if let Some(feed_idx) = self.selected_feed {
            let feeds = self.feed_manager.get_feeds();
            if let Some(feed) = feeds.get(feed_idx) {
                self.feed_manager.get_articles_for_feed(&feed.url)
            } else {
                self.feed_manager.get_all_articles()
            }
        } else {
            self.feed_manager.get_all_articles()
        };

        if self.show_only_unread {
            self.articles.retain(|a| !a.read);
        }

        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.articles.retain(|a| {
                a.title.to_lowercase().contains(&query)
                    || a.description.to_lowercase().contains(&query)
            });
        }

        // Reset selection if out of bounds
        if self.selected_article.map_or(false, |idx| idx >= self.articles.len()) {
            self.selected_article = if self.articles.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    fn next_article(&mut self) {
        if self.articles.is_empty() {
            return;
        }

        self.selected_article = if let Some(i) = self.selected_article {
            if i >= self.articles.len() - 1 {
                Some(0)
            } else {
                Some(i + 1)
            }
        } else {
            Some(0)
        };
        self.article_scroll_offset = 0;
    }

    fn previous_article(&mut self) {
        if self.articles.is_empty() {
            return;
        }

        self.selected_article = if let Some(i) = self.selected_article {
            if i == 0 {
                Some(self.articles.len() - 1)
            } else {
                Some(i - 1)
            }
        } else {
            Some(0)
        };
        self.article_scroll_offset = 0;
    }

    fn mark_current_as_read(&mut self) {
        if let Some(idx) = self.selected_article {
            if let Some(article) = self.articles.get(idx) {
                self.feed_manager.mark_as_read(&article.id);
                self.articles[idx].read = true;
            }
        }
    }

    fn toggle_read_status(&mut self) {
        if let Some(idx) = self.selected_article {
            if let Some(article) = self.articles.get(idx) {
                let new_status = !article.read;
                if new_status {
                    self.feed_manager.mark_as_read(&article.id);
                } else {
                    self.feed_manager.mark_as_unread(&article.id);
                }
                self.articles[idx].read = new_status;
                self.status_message = format!(
                    "Article marked as {}",
                    if new_status { "read" } else { "unread" }
                );
            }
        }
    }

    fn refresh_feeds(&mut self) {
        self.status_message = "Refreshing feeds...".to_string();
        match self.feed_manager.refresh_all_feeds() {
            Ok(count) => {
                self.status_message = format!("Refreshed {} articles", count);
                self.refresh_articles();
            }
            Err(e) => {
                self.status_message = format!("Error refreshing feeds: {}", e);
            }
        }
    }

    fn add_feed(&mut self, url: String) {
        match self.feed_manager.add_feed(url.clone()) {
            Ok(_) => {
                self.status_message = format!("Added feed: {}", url);
                self.refresh_articles();
            }
            Err(e) => {
                self.status_message = format!("Error adding feed: {}", e);
            }
        }
    }

    fn delete_current_feed(&mut self) {
        if let Some(idx) = self.selected_feed {
            let feeds = self.feed_manager.get_feeds();
            if let Some(feed) = feeds.get(idx) {
                let url = feed.url.clone();
                match self.feed_manager.delete_feed(&url) {
                    Ok(_) => {
                        self.status_message = format!("Deleted feed: {}", url);
                        self.selected_feed = None;
                        self.refresh_articles();
                    }
                    Err(e) => {
                        self.status_message = format!("Error deleting feed: {}", e);
                    }
                }
            }
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match self.active_panel {
            Panel::ArticleList => match key {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_article();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous_article();
                }
                KeyCode::Enter => {
                    self.active_panel = Panel::ArticleReader;
                    self.mark_current_as_read();
                }
                KeyCode::Char('n') => {
                    self.mode = Mode::AddFeed;
                    self.input = Input::default();
                }
                KeyCode::Char('r') => {
                    self.refresh_feeds();
                }
                KeyCode::Char('f') => {
                    self.focus_mode = !self.focus_mode;
                    self.status_message = format!(
                        "Focus mode {}",
                        if self.focus_mode { "enabled" } else { "disabled" }
                    );
                }
                KeyCode::Char('/') => {
                    self.mode = Mode::Search;
                    self.input = Input::default();
                    self.search_query.clear();
                }
                KeyCode::Char('m') => {
                    self.mode = Mode::FeedManagement;
                    self.selected_feed = None;
                }
                KeyCode::Char('u') => {
                    self.show_only_unread = !self.show_only_unread;
                    self.refresh_articles();
                    self.status_message = format!(
                        "Showing {} articles",
                        if self.show_only_unread { "unread" } else { "all" }
                    );
                }
                KeyCode::Char('t') => {
                    self.toggle_read_status();
                }
                KeyCode::Char('?') => {
                    self.mode = Mode::Help;
                }
                KeyCode::Tab => {
                    self.active_panel = Panel::ArticleReader;
                }
                _ => {}
            },
            Panel::ArticleReader => match key {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.article_scroll_offset = self.article_scroll_offset.saturating_add(SCROLL_STEP);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.article_scroll_offset = self.article_scroll_offset.saturating_sub(SCROLL_STEP);
                }
                KeyCode::PageDown => {
                    self.article_scroll_offset = self.article_scroll_offset.saturating_add(PAGE_SCROLL_STEP);
                }
                KeyCode::PageUp => {
                    self.article_scroll_offset = self.article_scroll_offset.saturating_sub(PAGE_SCROLL_STEP);
                }
                KeyCode::Esc => {
                    self.active_panel = Panel::ArticleList;
                }
                KeyCode::Char('f') => {
                    self.focus_mode = !self.focus_mode;
                    self.status_message = format!(
                        "Focus mode {}",
                        if self.focus_mode { "enabled" } else { "disabled" }
                    );
                }
                KeyCode::Char('n') => {
                    self.next_article();
                    self.article_scroll_offset = 0;
                    self.mark_current_as_read();
                }
                KeyCode::Char('p') => {
                    self.previous_article();
                    self.article_scroll_offset = 0;
                    self.mark_current_as_read();
                }
                KeyCode::Char('t') => {
                    self.toggle_read_status();
                }
                KeyCode::Char('?') => {
                    self.mode = Mode::Help;
                }
                KeyCode::Tab => {
                    self.active_panel = Panel::ArticleList;
                }
                _ => {}
            },
        }
    }

    fn handle_add_feed_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                let url = self.input.value().to_string();
                if !url.is_empty() {
                    self.add_feed(url);
                }
                self.mode = Mode::Normal;
                self.input = Input::default();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input = Input::default();
            }
            KeyCode::Char(c) => {
                self.input.handle(tui_input::InputRequest::InsertChar(c));
            }
            KeyCode::Backspace => {
                self.input.handle(tui_input::InputRequest::DeletePrevChar);
            }
            KeyCode::Left => {
                self.input.handle(tui_input::InputRequest::GoToPrevChar);
            }
            KeyCode::Right => {
                self.input.handle(tui_input::InputRequest::GoToNextChar);
            }
            _ => {}
        }
    }

    fn handle_search_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                self.search_query = self.input.value().to_string();
                self.refresh_articles();
                self.mode = Mode::Normal;
                self.input = Input::default();
                self.status_message = if self.search_query.is_empty() {
                    "Search cleared".to_string()
                } else {
                    format!("Searching for: {}", self.search_query)
                };
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input = Input::default();
            }
            KeyCode::Char(c) => {
                self.input.handle(tui_input::InputRequest::InsertChar(c));
            }
            KeyCode::Backspace => {
                self.input.handle(tui_input::InputRequest::DeletePrevChar);
            }
            KeyCode::Left => {
                self.input.handle(tui_input::InputRequest::GoToPrevChar);
            }
            KeyCode::Right => {
                self.input.handle(tui_input::InputRequest::GoToNextChar);
            }
            _ => {}
        }
    }

    fn handle_feed_management_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
                self.selected_feed = None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let feeds = self.feed_manager.get_feeds();
                if feeds.is_empty() {
                    return;
                }
                self.selected_feed = if let Some(i) = self.selected_feed {
                    if i >= feeds.len() - 1 {
                        Some(0)
                    } else {
                        Some(i + 1)
                    }
                } else {
                    Some(0)
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let feeds = self.feed_manager.get_feeds();
                if feeds.is_empty() {
                    return;
                }
                self.selected_feed = if let Some(i) = self.selected_feed {
                    if i == 0 {
                        Some(feeds.len() - 1)
                    } else {
                        Some(i - 1)
                    }
                } else {
                    Some(0)
                };
            }
            KeyCode::Char('d') => {
                self.delete_current_feed();
            }
            KeyCode::Char('r') => {
                if let Some(idx) = self.selected_feed {
                    let feeds = self.feed_manager.get_feeds();
                    if let Some(feed) = feeds.get(idx) {
                        self.status_message = format!("Refreshing {}...", feed.title);
                        match self.feed_manager.refresh_feed(&feed.url) {
                            Ok(count) => {
                                self.status_message = format!("Refreshed {} articles from {}", count, feed.title);
                                self.refresh_articles();
                            }
                            Err(e) => {
                                self.status_message = format!("Error refreshing feed: {}", e);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key, modifiers),
            Mode::AddFeed => self.handle_add_feed_mode(key),
            Mode::Search => self.handle_search_mode(key),
            Mode::FeedManagement => self.handle_feed_management_mode(key),
            Mode::Help => {
                if matches!(key, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                    self.mode = Mode::Normal;
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

            // Render based on mode
            match app.mode {
                Mode::Help => {
                    HelpOverlay::render(area, f.buffer_mut());
                    return;
                }
                Mode::AddFeed => {
                    // Render main UI
                    let chunks = if app.focus_mode {
                        Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(100)])
                            .split(area)
                    } else {
                        Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                            .split(area)
                    };

                    if !app.focus_mode {
                        ArticleList::render(
                            chunks[0],
                            f.buffer_mut(),
                            &app.articles,
                            app.selected_article,
                            app.active_panel == Panel::ArticleList,
                            app.list_scroll_offset,
                        );
                    }

                    let reader_area = if app.focus_mode { chunks[0] } else { chunks[1] };
                    let reader_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(3), Constraint::Length(3)])
                        .split(reader_area);

                    let selected_article = app
                        .selected_article
                        .and_then(|idx| app.articles.get(idx));
                    ArticleReader::render(
                        reader_chunks[0],
                        f.buffer_mut(),
                        selected_article,
                        app.article_scroll_offset,
                        app.active_panel == Panel::ArticleReader,
                    );

                    StatusBar::render(reader_chunks[1], f.buffer_mut(), &app.status_message);

                    // Render add feed modal on top
                    SearchModal::render(
                        area,
                        f.buffer_mut(),
                        "Add RSS Feed",
                        "Enter feed URL:",
                        &app.input,
                    );
                    return;
                }
                Mode::Search => {
                    // Render main UI
                    let chunks = if app.focus_mode {
                        Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(100)])
                            .split(area)
                    } else {
                        Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                            .split(area)
                    };

                    if !app.focus_mode {
                        ArticleList::render(
                            chunks[0],
                            f.buffer_mut(),
                            &app.articles,
                            app.selected_article,
                            app.active_panel == Panel::ArticleList,
                            app.list_scroll_offset,
                        );
                    }

                    let reader_area = if app.focus_mode { chunks[0] } else { chunks[1] };
                    let reader_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(3), Constraint::Length(3)])
                        .split(reader_area);

                    let selected_article = app
                        .selected_article
                        .and_then(|idx| app.articles.get(idx));
                    ArticleReader::render(
                        reader_chunks[0],
                        f.buffer_mut(),
                        selected_article,
                        app.article_scroll_offset,
                        app.active_panel == Panel::ArticleReader,
                    );

                    StatusBar::render(reader_chunks[1], f.buffer_mut(), &app.status_message);

                    // Render search modal on top
                    SearchModal::render(
                        area,
                        f.buffer_mut(),
                        "Search",
                        "Search articles:",
                        &app.input,
                    );
                    return;
                }
                Mode::FeedManagement => {
                    FeedList::render(
                        area,
                        f.buffer_mut(),
                        &app.feed_manager.get_feeds(),
                        app.selected_feed,
                    );
                    return;
                }
                Mode::Normal => {}
            }

            // Normal mode rendering
            let chunks = if app.focus_mode {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .split(area)
            } else {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                    .split(area)
            };

            if !app.focus_mode {
                ArticleList::render(
                    chunks[0],
                    f.buffer_mut(),
                    &app.articles,
                    app.selected_article,
                    app.active_panel == Panel::ArticleList,
                    app.list_scroll_offset,
                );
            }

            let reader_area = if app.focus_mode { chunks[0] } else { chunks[1] };
            let reader_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(reader_area);

            let selected_article = app
                .selected_article
                .and_then(|idx| app.articles.get(idx));
            ArticleReader::render(
                reader_chunks[0],
                f.buffer_mut(),
                selected_article,
                app.article_scroll_offset,
                app.active_panel == Panel::ArticleReader,
            );

            StatusBar::render(reader_chunks[1], f.buffer_mut(), &app.status_message);
        })?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            app.handle_key_event(key.code, key.modifiers);
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
