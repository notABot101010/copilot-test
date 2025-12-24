mod crypto;
mod ui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui_input::backend::crossterm::EventHandler;
use crypto::{Credential, Vault};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use ui::{ConfirmDialog, CredentialDetail, CredentialList, HelpBar, InputDialog};
use tui_input::Input;
use zeroize::Zeroize;

#[derive(Parser)]
#[command(name = "tui-pass")]
#[command(about = "A terminal-based password manager with encrypted vaults", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the vault file (when opening)
    vault_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault
    Create {
        /// Path to the new vault file
        vault_file: PathBuf,
    },
}

enum AppMode {
    Normal,
    AddingCredential,
    EditingCredential(usize),
    ConfirmDelete(usize),
}

struct InputState {
    title: Input,
    username: Input,
    password: Input,
    url: Input,
    notes: Input,
    active_field: usize,
}

impl InputState {
    const MAX_FIELD_INDEX: usize = 4;

    fn new() -> Self {
        Self {
            title: Input::default(),
            username: Input::default(),
            password: Input::default(),
            url: Input::default(),
            notes: Input::default(),
            active_field: 0,
        }
    }

    fn from_credential(cred: &Credential) -> Self {
        Self {
            title: Input::new(cred.title.clone()),
            username: Input::new(cred.username.clone()),
            password: Input::new(cred.password.clone()),
            url: Input::new(cred.url.clone()),
            notes: Input::new(cred.notes.clone()),
            active_field: 0,
        }
    }

    fn to_credential(&self) -> Credential {
        Credential {
            title: self.title.value().to_string(),
            username: self.username.value().to_string(),
            password: self.password.value().to_string(),
            url: self.url.value().to_string(),
            notes: self.notes.value().to_string(),
        }
    }

    fn clear(&mut self) {
        self.title = Input::default();
        self.username = Input::default();
        self.password = Input::default();
        self.url = Input::default();
        self.notes = Input::default();
        self.active_field = 0;
    }

    fn get_active_field_mut(&mut self) -> &mut Input {
        match self.active_field {
            0 => &mut self.title,
            1 => &mut self.username,
            2 => &mut self.password,
            3 => &mut self.url,
            4 => &mut self.notes,
            _ => &mut self.title,
        }
    }

    fn move_to_previous_field(&mut self) {
        self.active_field = self.active_field.saturating_sub(1);
    }

    fn move_to_next_field(&mut self) {
        self.active_field = (self.active_field + 1).min(Self::MAX_FIELD_INDEX);
    }
}

struct App {
    vault: Vault,
    vault_path: PathBuf,
    selected_idx: Option<usize>,
    scroll_offset: usize,
    show_password: bool,
    mode: AppMode,
    input_state: InputState,
    modified: bool,
    credential_list_area: ratatui::layout::Rect,
}

impl App {
    fn new(vault: Vault, vault_path: PathBuf) -> Self {
        Self {
            vault,
            vault_path,
            selected_idx: None,
            scroll_offset: 0,
            show_password: false,
            mode: AppMode::Normal,
            input_state: InputState::new(),
            modified: false,
            credential_list_area: ratatui::layout::Rect::default(),
        }
    }

    fn select_next(&mut self) {
        if self.vault.is_empty() {
            return;
        }

        let new_idx = match self.selected_idx {
            Some(idx) => {
                if idx + 1 < self.vault.len() {
                    idx + 1
                } else {
                    idx
                }
            }
            None => 0,
        };

        self.selected_idx = Some(new_idx);
        self.ensure_visible(new_idx);
    }

    fn select_prev(&mut self) {
        if self.vault.is_empty() {
            return;
        }

        let new_idx = match self.selected_idx {
            Some(idx) => idx.saturating_sub(1),
            None => 0,
        };

        self.selected_idx = Some(new_idx);
        self.ensure_visible(new_idx);
    }

    fn ensure_visible(&mut self, idx: usize) {
        let visible_height = self.credential_list_area.height.saturating_sub(2) as usize;
        if visible_height == 0 {
            return;
        }

        if idx < self.scroll_offset {
            self.scroll_offset = idx;
        } else if idx >= self.scroll_offset + visible_height {
            self.scroll_offset = idx.saturating_sub(visible_height - 1);
        }
    }

    fn toggle_password(&mut self) {
        self.show_password = !self.show_password;
    }

    fn start_add_credential(&mut self) {
        self.input_state.clear();
        self.mode = AppMode::AddingCredential;
    }

    fn start_edit_credential(&mut self) {
        if let Some(idx) = self.selected_idx {
            if idx < self.vault.len() {
                if let Ok(cred) = self.vault.get_credential(idx) {
                    self.input_state = InputState::from_credential(cred);
                    self.mode = AppMode::EditingCredential(idx);
                }
            }
        }
    }

    fn start_delete_credential(&mut self) {
        if let Some(idx) = self.selected_idx {
            if idx < self.vault.len() {
                self.mode = AppMode::ConfirmDelete(idx);
            }
        }
    }

    fn save_vault(&mut self) -> Result<()> {
        let encrypted = crypto::encrypt_vault(&self.vault)
            .context("Failed to encrypt vault")?;

        fs::write(&self.vault_path, encrypted).context("Failed to write vault file")?;

        self.modified = false;
        Ok(())
    }

    fn handle_input_mode_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.input_state.move_to_previous_field();
                } else {
                    self.input_state.move_to_next_field();
                }
            }
            KeyCode::Up => {
                self.input_state.move_to_previous_field();
            }
            KeyCode::Down => {
                self.input_state.move_to_next_field();
            }
            KeyCode::Enter => {
                let credential = self.input_state.to_credential();
                
                match self.mode {
                    AppMode::AddingCredential => {
                        if let Err(e) = self.vault.add_credential(credential) {
                            eprintln!("Failed to add credential: {}", e);
                        } else {
                            self.selected_idx = Some(self.vault.len() - 1);
                            self.modified = true;
                        }
                    }
                    AppMode::EditingCredential(idx) => {
                        if let Err(e) = self.vault.update_credential(idx, credential) {
                            eprintln!("Failed to update credential: {}", e);
                        } else {
                            self.modified = true;
                        }
                    }
                    _ => {}
                }
                
                self.mode = AppMode::Normal;
            }
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            _ => {
                // Let tui-input handle all other keys
                self.input_state.get_active_field_mut().handle_event(&Event::Key(key));
            }
        }
    }

    fn handle_confirm_delete_key(&mut self, key: KeyCode) {
        if let AppMode::ConfirmDelete(idx) = self.mode {
            match key {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Err(e) = self.vault.remove_credential(idx) {
                        eprintln!("Failed to remove credential: {}", e);
                    } else {
                        self.modified = true;
                        
                        // Adjust selection
                        if self.vault.is_empty() {
                            self.selected_idx = None;
                        } else if let Some(selected) = self.selected_idx {
                            if selected >= self.vault.len() {
                                self.selected_idx = Some(self.vault.len() - 1);
                            }
                        }
                    }
                    self.mode = AppMode::Normal;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.mode = AppMode::Normal;
                }
                _ => {}
            }
        }
    }

    fn handle_mouse(&mut self, mouse: event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check if click is in credential list area
                if mouse.column >= self.credential_list_area.x
                    && mouse.column < self.credential_list_area.x + self.credential_list_area.width
                    && mouse.row >= self.credential_list_area.y
                    && mouse.row < self.credential_list_area.y + self.credential_list_area.height
                {
                    // Calculate which credential was clicked (accounting for border)
                    let relative_row = mouse.row.saturating_sub(self.credential_list_area.y + 1) as usize;
                    let clicked_idx = self.scroll_offset + relative_row;
                    
                    if clicked_idx < self.vault.len() {
                        self.selected_idx = Some(clicked_idx);
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            MouseEventKind::ScrollDown => {
                let max_scroll = self.vault.len().saturating_sub(1);
                self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
            }
            _ => {}
        }
    }

    fn enter_copy_mode(&mut self) -> bool {
        // Only allow copy mode if a credential is selected
        self.selected_idx.is_some()
    }
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        // Prepare data for rendering (decrypt if needed)
        let titles = app.vault.get_titles().unwrap_or_else(|_| Vec::new());
        let selected_credential = app
            .selected_idx
            .and_then(|idx| app.vault.get_credential(idx).ok())
            .cloned();

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(f.area());

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(chunks[0]);

            // Store area for mouse interaction
            app.credential_list_area = main_chunks[0];

            // Left pane: credential list (now uses titles only)
            let credential_list = CredentialList::new(
                &titles,
                app.selected_idx,
                app.scroll_offset,
            );
            f.render_widget(credential_list, main_chunks[0]);

            // Right pane: credential details
            let credential_detail = CredentialDetail::new(selected_credential.as_ref(), app.show_password);
            f.render_widget(credential_detail, main_chunks[1]);

            // Bottom: help bar
            f.render_widget(HelpBar, chunks[1]);

            // Overlay dialogs
            match &app.mode {
                AppMode::AddingCredential => {
                    let dialog = InputDialog {
                        title: "Add Credential",
                        title_input: &app.input_state.title,
                        username_input: &app.input_state.username,
                        password_input: &app.input_state.password,
                        url_input: &app.input_state.url,
                        notes_input: &app.input_state.notes,
                        active_field: app.input_state.active_field,
                    };
                    f.render_widget(dialog, f.area());
                }
                AppMode::EditingCredential(_) => {
                    let dialog = InputDialog {
                        title: "Edit Credential",
                        title_input: &app.input_state.title,
                        username_input: &app.input_state.username,
                        password_input: &app.input_state.password,
                        url_input: &app.input_state.url,
                        notes_input: &app.input_state.notes,
                        active_field: app.input_state.active_field,
                    };
                    f.render_widget(dialog, f.area());
                }
                AppMode::ConfirmDelete(_) => {
                    let dialog = ConfirmDialog {
                        message: "Are you sure you want to delete this credential?",
                    };
                    f.render_widget(dialog, f.area());
                }
                _ => {}
            }
        })?;

        match event::read()? {
            Event::Key(key) => match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        if app.modified {
                            // In a real app, we'd prompt to save here
                            // For now, just save automatically
                            app.save_vault()?;
                        }
                        break;
                    }
                    KeyCode::Down => app.select_next(),
                    KeyCode::Up => app.select_prev(),
                    KeyCode::Enter => {
                        if app.selected_idx.is_none() && !app.vault.is_empty() {
                            app.selected_idx = Some(0);
                        }
                    }
                    KeyCode::Char(' ') => app.toggle_password(),
                    KeyCode::Char('a') => app.start_add_credential(),
                    KeyCode::Char('e') => app.start_edit_credential(),
                    KeyCode::Char('d') => app.start_delete_credential(),
                    KeyCode::Char('s') => {
                        app.save_vault()?;
                    }
                    KeyCode::Char('c') => {
                        if app.enter_copy_mode() {
                            // Exit terminal temporarily for text selection
                            disable_raw_mode()?;
                            let mut stdout = io::stdout();
                            execute!(
                                stdout,
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;

                            // Display credential details for copying
                            display_credential_for_copying(&mut app)?;

                            // Wait for user to press a key
                            println!("\nPress any key to return to the application...");
                            // Temporarily enable raw mode to read a single key press
                            enable_raw_mode()?;
                            event::read()?;
                            disable_raw_mode()?;

                            // Re-enter terminal
                            enable_raw_mode()?;
                            execute!(
                                stdout,
                                EnterAlternateScreen,
                                EnableMouseCapture
                            )?;
                            terminal.clear()?;
                        }
                    }
                    _ => {}
                },
                AppMode::AddingCredential | AppMode::EditingCredential(_) => {
                    app.handle_input_mode_key(key);
                }
                AppMode::ConfirmDelete(_) => {
                    app.handle_confirm_delete_key(key.code);
                }
            },
            Event::Mouse(mouse) => {
                if matches!(app.mode, AppMode::Normal) {
                    app.handle_mouse(mouse);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn display_credential_for_copying(app: &mut App) -> Result<()> {
    if let Some(idx) = app.selected_idx {
        if let Ok(cred) = app.vault.get_credential(idx) {
            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║           COPY MODE - Select text with your mouse           ║");
            println!("║                                                              ║");
            println!("║  ⚠️  WARNING: Password is visible in plain text!           ║");
            println!("╚══════════════════════════════════════════════════════════════╝\n");
            
            println!("Title:    {}", cred.title);
            println!("Username: {}", cred.username);
            println!("Password: {}", cred.password);
            println!("URL:      {}", cred.url);
            if !cred.notes.is_empty() {
                println!("Notes:    {}", cred.notes);
            }
        }
    }
    Ok(())
}

fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::Write::flush(&mut io::stdout()).context("Failed to flush stdout")?;
    let password = rpassword::read_password().context("Failed to read password")?;
    Ok(password)
}

fn create_vault(vault_path: PathBuf) -> Result<()> {
    // Check if file already exists
    if vault_path.exists() {
        anyhow::bail!("Vault file already exists: {}", vault_path.display());
    }

    // Prompt for password twice
    let mut password = prompt_password("Enter master password: ")?;
    let mut password_confirm = prompt_password("Confirm master password: ")?;

    if password != password_confirm {
        // Zeroize passwords before returning error
        password.zeroize();
        password_confirm.zeroize();
        anyhow::bail!("Passwords do not match");
    }

    // Zeroize confirmation password as it's no longer needed
    password_confirm.zeroize();

    if password.is_empty() {
        password.zeroize();
        anyhow::bail!("Password cannot be empty");
    }

    // Create empty vault (password will be zeroized inside with_password)
    let vault = Vault::with_password(password)?;
    let encrypted = crypto::encrypt_vault(&vault).context("Failed to encrypt vault")?;

    fs::write(&vault_path, encrypted).context("Failed to write vault file")?;

    println!("Vault created successfully: {}", vault_path.display());
    Ok(())
}

fn open_vault(vault_path: PathBuf) -> Result<()> {
    // Check if file exists
    if !vault_path.exists() {
        anyhow::bail!("Vault file not found: {}", vault_path.display());
    }

    // Read vault file
    let encrypted = fs::read(&vault_path).context("Failed to read vault file")?;

    // Prompt for password
    let mut password = prompt_password("Enter master password: ")?;

    // Decrypt vault (password will be zeroized inside decrypt_vault -> with_password_and_salt)
    let vault = crypto::decrypt_vault(&encrypted, &password).context("Failed to decrypt vault")?;

    // Zeroize password from memory as it's no longer needed
    password.zeroize();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let app = App::new(vault, vault_path);
    let result = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Create { vault_file }) => create_vault(vault_file),
        None => {
            if let Some(vault_file) = cli.vault_file {
                open_vault(vault_file)
            } else {
                eprintln!("Usage:");
                eprintln!("  tui-pass <vault-file>           Open an existing vault");
                eprintln!("  tui-pass create <vault-file>    Create a new vault");
                std::process::exit(1);
            }
        }
    }
}
