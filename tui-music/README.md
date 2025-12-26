# TUI Music Player

A terminal-based music player built with Rust and ratatui, featuring a Spotify-inspired interface.

## Features

- 🎵 Support for multiple audio formats (MP3, FLAC, WAV, OGG, M4A, AAC, OPUS)
- 📁 Recursive folder scanning to load music libraries
- 🎨 Three-pane interface inspired by Spotify:
  - Left pane: Navigation (Tracks, Albums, Artists)
  - Center pane: Content listing
  - Bottom bar: Now playing information
- 🎹 Keyboard controls for navigation and playback
- 🏷️ Automatic metadata extraction from audio files

## Installation

Build the project from the repository root:

```bash
cargo build --release -p tui-music
```

## Usage

Run the music player:

```bash
cargo run -p tui-music
```

## Controls

### Navigation
- `1` - Switch to Tracks view
- `2` - Switch to Albums view
- `3` - Switch to Artists view
- `↑`/`k` - Move selection up
- `↓`/`j` - Move selection down

### Playback
- `Enter` - Play selected track
- `Space` - Pause/Resume playback
- `s` - Stop playback

### Library
- `a` - Add a music folder to library

### General
- `q` - Quit application

## Adding Music

1. Press `a` to open the "Add Music Folder" dialog
2. Type the full path to your music folder (e.g., `/home/user/Music`)
3. Press `Enter` to confirm
4. The application will recursively scan the folder and load all supported audio files

## Supported Audio Formats

- MP3 (`.mp3`)
- FLAC (`.flac`)
- WAV (`.wav`)
- OGG Vorbis (`.ogg`)
- M4A (`.m4a`)
- AAC (`.aac`)
- Opus (`.opus`)

## Architecture

The application is structured into four main modules:

- `main.rs` - Application state and event handling
- `ui.rs` - User interface rendering with ratatui
- `library.rs` - Music library management and metadata extraction
- `player.rs` - Audio playback using rodio

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation
- `rodio` - Audio playback
- `symphonia` - Audio metadata extraction
- `walkdir` - Recursive directory traversal

## License

MIT
