use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const UNKNOWN_ARTIST: &str = "Unknown Artist";
pub const UNKNOWN_TITLE: &str = "Unknown Title";
pub const UNKNOWN_ALBUM: &str = "Unknown Album";

#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Tracks,
    Albums,
    Artists,
}

pub struct Library {
    tracks: Vec<Track>,
    view_mode: ViewMode,
    folders: Vec<PathBuf>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            view_mode: ViewMode::Tracks,
            folders: Vec::new(),
        }
    }

    pub fn add_folder(&mut self, path: &str) {
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return;
        }

        self.folders.push(path_buf.clone());
        self.scan_folder(&path_buf);
    }

    fn scan_folder(&mut self, path: &Path) {
        let audio_extensions = vec!["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus"];
        
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if audio_extensions.contains(&ext_str.to_lowercase().as_str()) {
                        let track = self.load_track_metadata(path);
                        self.tracks.push(track);
                    }
                }
            }
        }

        // Sort tracks by artist, then album, then title
        self.tracks.sort_by(|a, b| {
            let artist_cmp = a.artist.cmp(&b.artist);
            if artist_cmp != std::cmp::Ordering::Equal {
                return artist_cmp;
            }
            let album_cmp = a.album.cmp(&b.album);
            if album_cmp != std::cmp::Ordering::Equal {
                return album_cmp;
            }
            a.title.cmp(&b.title)
        });
    }

    fn load_track_metadata(&self, path: &Path) -> Track {
        // Try to extract metadata using symphonia
        let metadata = self.extract_metadata_symphonia(path);
        
        let title = metadata.0.or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        });

        Track {
            path: path.to_path_buf(),
            title,
            artist: metadata.1,
            album: metadata.2,
        }
    }

    fn extract_metadata_symphonia(&self, path: &Path) -> (Option<String>, Option<String>, Option<String>) {
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return (None, None, None),
        };

        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                hint.with_extension(ext_str);
            }
        }

        let meta_opts: MetadataOptions = Default::default();
        let mut probed = match symphonia::default::get_probe().format(&hint, mss, &Default::default(), &meta_opts) {
            Ok(p) => p,
            Err(_) => return (None, None, None),
        };

        let mut title = None;
        let mut artist = None;
        let mut album = None;

        if let Some(metadata) = probed.metadata.get() {
            if let Some(current) = metadata.current() {
                for tag in current.tags() {
                    match tag.std_key {
                        Some(symphonia::core::meta::StandardTagKey::TrackTitle) => {
                            title = Some(tag.value.to_string());
                        }
                        Some(symphonia::core::meta::StandardTagKey::Artist) => {
                            artist = Some(tag.value.to_string());
                        }
                        Some(symphonia::core::meta::StandardTagKey::Album) => {
                            album = Some(tag.value.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        (title, artist, album)
    }

    pub fn get_current_tracks(&self) -> Vec<Track> {
        match self.view_mode {
            ViewMode::Tracks => self.tracks.clone(),
            ViewMode::Albums => {
                // Group by album and return unique albums
                let mut seen = HashSet::new();
                self.tracks
                    .iter()
                    .filter(|t| {
                        let album = t.album.as_ref().map(|s| s.as_str()).unwrap_or(UNKNOWN_ALBUM);
                        seen.insert(album.to_string())
                    })
                    .cloned()
                    .collect()
            }
            ViewMode::Artists => {
                // Group by artist and return unique artists
                let mut seen = HashSet::new();
                self.tracks
                    .iter()
                    .filter(|t| {
                        let artist = t.artist.as_ref().map(|s| s.as_str()).unwrap_or(UNKNOWN_ARTIST);
                        seen.insert(artist.to_string())
                    })
                    .cloned()
                    .collect()
            }
        }
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }
}
