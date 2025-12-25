mod epub_parser;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use epub_parser::BookContent;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io;
use ui::{render_book_content, render_toc};

enum Focus {
    Toc,
    Content,
}

struct App {
    book_content: BookContent,
    toc_visible: bool,
    focus: Focus,
    selected_toc_index: usize,
    toc_scroll_offset: usize,
    content_scroll_offset: usize,
    current_section_index: usize,
    should_quit: bool,
    max_scroll: usize,
}

impl App {
    fn new(book_content: BookContent) -> Self {
        Self {
            book_content,
            toc_visible: true,
            focus: Focus::Content,
            selected_toc_index: 0,
            toc_scroll_offset: 0,
            content_scroll_offset: 0,
            current_section_index: 0,
            should_quit: false,
            max_scroll: 0,
        }
    }

    fn toggle_toc(&mut self) {
        self.toc_visible = !self.toc_visible;
        if !self.toc_visible {
            self.focus = Focus::Content;
        }
    }

    fn next_toc_item(&mut self) {
        if !self.book_content.toc.is_empty() && self.selected_toc_index < self.book_content.toc.len() - 1 {
            self.selected_toc_index += 1;
        }
    }

    fn previous_toc_item(&mut self) {
        if self.selected_toc_index > 0 {
            self.selected_toc_index -= 1;
        }
    }

    fn select_toc_item(&mut self) {
        if self.selected_toc_index < self.book_content.toc.len() {
            let section_index = self.book_content.toc[self.selected_toc_index].section_index;
            self.current_section_index = section_index;
            self.content_scroll_offset = 0;
            self.focus = Focus::Content;
        }
    }

    fn scroll_content_down(&mut self) {
        if self.content_scroll_offset < self.max_scroll {
            self.content_scroll_offset += 1;
        } else if self.current_section_index < self.book_content.sections.len() - 1 {
            // At the end of current chapter, move to next chapter
            self.current_section_index += 1;
            self.content_scroll_offset = 0;
            self.sync_toc_with_current_section();
        }
    }

    fn scroll_content_up(&mut self) {
        if self.content_scroll_offset > 0 {
            self.content_scroll_offset -= 1;
        } else if self.current_section_index > 0 {
            // At the beginning of current chapter, move to previous chapter
            self.current_section_index -= 1;
            // Set scroll to maximum value - will be clamped to actual max in render
            // This ensures we start at the bottom of the previous chapter
            self.content_scroll_offset = usize::MAX;
            self.sync_toc_with_current_section();
        }
    }

    fn update_toc_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }

        // Calculate scroll offset to keep selected item visible
        if self.selected_toc_index < self.toc_scroll_offset {
            // Selected item is above visible area
            self.toc_scroll_offset = self.selected_toc_index;
        } else if self.selected_toc_index >= self.toc_scroll_offset + visible_height {
            // Selected item is below visible area
            self.toc_scroll_offset = self.selected_toc_index.saturating_sub(visible_height - 1);
        }
    }

    fn switch_focus(&mut self) {
        if self.toc_visible {
            self.focus = match self.focus {
                Focus::Toc => Focus::Content,
                Focus::Content => Focus::Toc,
            };
        }
    }

    fn sync_toc_with_current_section(&mut self) {
        // Find the TOC entry that corresponds to the current section
        // We look for the TOC entry with the highest section_index that is <= current_section_index
        if self.book_content.toc.is_empty() {
            return;
        }
        
        let mut best_match = 0;
        for (i, entry) in self.book_content.toc.iter().enumerate() {
            if entry.section_index <= self.current_section_index {
                best_match = i;
            } else {
                // TOC is sorted by section_index, so we can stop here
                break;
            }
        }
        
        self.selected_toc_index = best_match;
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: tui-book <book.epub>");
        std::process::exit(1);
    }

    let epub_path = &args[1];
    let book_content = epub_parser::parse_epub(epub_path)
        .context("Failed to parse EPUB file")?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(book_content);
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = if app.toc_visible {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                    .split(size)
            } else {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .split(size)
            };

            if app.toc_visible {
                let toc_focused = matches!(app.focus, Focus::Toc);
                let toc_height = chunks[0].height.saturating_sub(2) as usize; // Subtract borders
                app.update_toc_scroll(toc_height);
                
                render_toc(
                    f,
                    chunks[0],
                    &app.book_content.toc,
                    app.selected_toc_index,
                    app.toc_scroll_offset,
                    toc_focused,
                );

                let content_focused = matches!(app.focus, Focus::Content);
                let current_content = if app.current_section_index < app.book_content.sections.len() {
                    &app.book_content.sections[app.current_section_index]
                } else {
                    ""
                };

                let max_scroll = render_book_content(
                    f,
                    chunks[1],
                    current_content,
                    app.content_scroll_offset,
                    content_focused,
                );

                // Update max_scroll and adjust scroll offset if needed
                app.max_scroll = max_scroll;
                if app.content_scroll_offset > max_scroll {
                    app.content_scroll_offset = max_scroll;
                }
            } else {
                let content_focused = matches!(app.focus, Focus::Content);
                let current_content = if app.current_section_index < app.book_content.sections.len() {
                    &app.book_content.sections[app.current_section_index]
                } else {
                    ""
                };

                let max_scroll = render_book_content(
                    f,
                    chunks[0],
                    current_content,
                    app.content_scroll_offset,
                    content_focused,
                );

                app.max_scroll = max_scroll;
                if app.content_scroll_offset > max_scroll {
                    app.content_scroll_offset = max_scroll;
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.focus {
                    Focus::Toc => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('q'), KeyModifiers::NONE) => app.should_quit = true,
                            (KeyCode::Char('b'), KeyModifiers::CONTROL) => app.toggle_toc(),
                            (KeyCode::Up, KeyModifiers::NONE) => app.previous_toc_item(),
                            (KeyCode::Down, KeyModifiers::NONE) => app.next_toc_item(),
                            (KeyCode::Enter, KeyModifiers::NONE) => app.select_toc_item(),
                            (KeyCode::Tab, KeyModifiers::NONE) => app.switch_focus(),
                            _ => {}
                        }
                    }
                    Focus::Content => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('q'), KeyModifiers::NONE) => app.should_quit = true,
                            (KeyCode::Char('b'), KeyModifiers::CONTROL) => app.toggle_toc(),
                            (KeyCode::Up, KeyModifiers::NONE) => app.scroll_content_up(),
                            (KeyCode::Down, KeyModifiers::NONE) => app.scroll_content_down(),
                            (KeyCode::Tab, KeyModifiers::NONE) => app.switch_focus(),
                            _ => {}
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
