mod ui;
mod library;
mod player;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use library::{Library, ViewMode};
use player::Player;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

const SCROLL_STEP: usize = 1;

enum InputMode {
    Normal,
    AddFolder,
}

struct App {
    library: Library,
    player: Player,
    input_mode: InputMode,
    folder_input: String,
    should_quit: bool,
    scroll_offset: usize,
    selected_track_index: Option<usize>,
}

impl App {
    fn new() -> Self {
        Self {
            library: Library::new(),
            player: Player::new(),
            input_mode: InputMode::Normal,
            folder_input: String::new(),
            should_quit: false,
            scroll_offset: 0,
            selected_track_index: None,
        }
    }

    fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(SCROLL_STEP);
    }

    fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(SCROLL_STEP);
    }

    fn select_next(&mut self) {
        let tracks = self.library.get_current_tracks();
        if tracks.is_empty() {
            return;
        }
        
        self.selected_track_index = if let Some(i) = self.selected_track_index {
            if i >= tracks.len() - 1 {
                Some(0)
            } else {
                Some(i + 1)
            }
        } else {
            Some(0)
        };
    }

    fn select_previous(&mut self) {
        let tracks = self.library.get_current_tracks();
        if tracks.is_empty() {
            return;
        }
        
        self.selected_track_index = if let Some(i) = self.selected_track_index {
            if i == 0 {
                Some(tracks.len() - 1)
            } else {
                Some(i - 1)
            }
        } else {
            Some(0)
        };
    }

    fn play_selected(&mut self) {
        use library::{UNKNOWN_ARTIST, UNKNOWN_TITLE};
        
        if let Some(idx) = self.selected_track_index {
            let tracks = self.library.get_current_tracks();
            if let Some(track) = tracks.get(idx) {
                let title = track.title.clone().unwrap_or_else(|| UNKNOWN_TITLE.to_string());
                let artist = track.artist.clone().unwrap_or_else(|| UNKNOWN_ARTIST.to_string());
                
                if let Err(err) = self.player.play(&track.path, title, artist) {
                    eprintln!("Failed to play track: {:?}", err);
                }
            }
        }
    }

    fn toggle_playback(&mut self) {
        self.player.toggle_pause();
    }

    fn stop_playback(&mut self) {
        self.player.stop();
    }

    fn add_folder(&mut self) {
        if !self.folder_input.trim().is_empty() {
            self.library.add_folder(&self.folder_input);
            self.folder_input.clear();
            self.input_mode = InputMode::Normal;
        }
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    self.input_mode = InputMode::AddFolder;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_previous();
                }
                KeyCode::Char('1') => {
                    self.library.set_view_mode(ViewMode::Tracks);
                    self.selected_track_index = None;
                    self.scroll_offset = 0;
                }
                KeyCode::Char('2') => {
                    self.library.set_view_mode(ViewMode::Albums);
                    self.selected_track_index = None;
                    self.scroll_offset = 0;
                }
                KeyCode::Char('3') => {
                    self.library.set_view_mode(ViewMode::Artists);
                    self.selected_track_index = None;
                    self.scroll_offset = 0;
                }
                KeyCode::Enter => {
                    self.play_selected();
                }
                KeyCode::Char(' ') => {
                    self.toggle_playback();
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.stop_playback();
                }
                _ => {}
            },
            InputMode::AddFolder => match key_event.code {
                KeyCode::Enter => {
                    self.add_folder();
                }
                KeyCode::Char(c) => {
                    self.folder_input.push(c);
                }
                KeyCode::Backspace => {
                    self.folder_input.pop();
                }
                KeyCode::Esc => {
                    self.folder_input.clear();
                    self.input_mode = InputMode::Normal;
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
            ui::render(
                f,
                &app.library,
                &app.player,
                app.selected_track_index,
                app.scroll_offset,
                &app.folder_input,
                matches!(app.input_mode, InputMode::AddFolder),
            );
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
