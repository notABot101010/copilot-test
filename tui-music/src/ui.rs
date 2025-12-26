use crate::library::{Library, ViewMode};
use crate::player::Player;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

const UNKNOWN_ARTIST: &str = "Unknown Artist";

pub fn render(
    f: &mut Frame,
    library: &Library,
    player: &Player,
    selected_index: Option<usize>,
    scroll_offset: usize,
    folder_input: &str,
    is_adding_folder: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(chunks[0]);

    render_sidebar(f, main_chunks[0], library.get_view_mode());
    render_content(f, main_chunks[1], library, selected_index, scroll_offset);
    render_player_bar(f, chunks[1], player, folder_input, is_adding_folder);
}

fn render_sidebar(f: &mut Frame, area: Rect, view_mode: ViewMode) {
    let items = vec![
        ("1", "Tracks", ViewMode::Tracks),
        ("2", "Albums", ViewMode::Albums),
        ("3", "Artists", ViewMode::Artists),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(key, label, mode)| {
            let style = if *mode == view_mode {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if *mode == view_mode { "▶ " } else { "  " };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{} ", prefix, key), style),
                Span::styled(*label, style),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Library ")
        .style(Style::default().fg(Color::Cyan));

    let list = List::new(list_items).block(block);
    f.render_widget(list, area);
}

fn render_content(
    f: &mut Frame,
    area: Rect,
    library: &Library,
    selected_index: Option<usize>,
    scroll_offset: usize,
) {
    let tracks = library.get_current_tracks();
    let view_mode = library.get_view_mode();
    
    let title = match view_mode {
        ViewMode::Tracks => " All Tracks ",
        ViewMode::Albums => " Albums ",
        ViewMode::Artists => " Artists ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if tracks.is_empty() {
        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No music in library",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'a' to add a music folder",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Controls:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  1/2/3 - Switch views (Tracks/Albums/Artists)",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "  ↑/↓ or j/k - Navigate",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "  Enter - Play selected track",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "  Space - Pause/Resume",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "  s - Stop playback",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "  q - Quit",
                Style::default().fg(Color::Gray),
            )),
        ];

        let paragraph = Paragraph::new(help_text);
        f.render_widget(paragraph, inner_area);
        return;
    }

    let items: Vec<ListItem> = tracks
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .map(|(idx, track)| {
            let style = if Some(idx) == selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = match view_mode {
                ViewMode::Tracks => {
                    let artist = track.artist.as_deref().unwrap_or(UNKNOWN_ARTIST);
                    let title = track.title.as_deref().unwrap_or("Unknown Title");
                    Line::from(vec![
                        Span::styled(format!("{} ", title), style),
                        Span::styled(format!("- {}", artist), Style::default().fg(Color::Gray)),
                    ])
                }
                ViewMode::Albums => {
                    let album = track.album.as_deref().unwrap_or("Unknown Album");
                    let artist = track.artist.as_deref().unwrap_or(UNKNOWN_ARTIST);
                    Line::from(vec![
                        Span::styled(format!("{} ", album), style),
                        Span::styled(format!("- {}", artist), Style::default().fg(Color::Gray)),
                    ])
                }
                ViewMode::Artists => {
                    let artist = track.artist.as_deref().unwrap_or(UNKNOWN_ARTIST);
                    let track_count = tracks.iter().filter(|t| t.artist.as_deref() == Some(artist)).count();
                    Line::from(vec![
                        Span::styled(format!("{} ", artist), style),
                        Span::styled(format!("({} tracks)", track_count), Style::default().fg(Color::Gray)),
                    ])
                }
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner_area);
}

fn render_player_bar(
    f: &mut Frame,
    area: Rect,
    player: &Player,
    folder_input: &str,
    is_adding_folder: bool,
) {
    if is_adding_folder {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Add Music Folder (Enter to confirm, Esc to cancel) ")
            .style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let text = if folder_input.is_empty() {
            vec![Line::from(Span::styled(
                "Enter folder path...",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            vec![Line::from(folder_input)]
        };

        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, inner_area);
    } else {
        let current_track = player.current_track();
        let is_playing = player.is_playing();
        
        let status = if is_playing {
            "▶ Playing"
        } else if current_track.is_some() {
            "⏸ Paused"
        } else {
            "⏹ Stopped"
        };

        let track_info = if let Some(track) = current_track {
            format!(" {} - {}", track, UNKNOWN_ARTIST)
        } else {
            " No track playing".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", status))
            .style(Style::default().fg(Color::Green));

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let text = vec![Line::from(track_info)];
        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, inner_area);
    }
}
