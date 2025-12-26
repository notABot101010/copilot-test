use anyhow::Result;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
}

pub struct Player {
    sink: Arc<Mutex<Option<Sink>>>,
    _stream: Option<OutputStream>,
    _stream_handle: Option<OutputStreamHandle>,
    current_track: Arc<Mutex<Option<TrackInfo>>>,
}

impl Player {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().ok().unzip();
        
        Self {
            sink: Arc::new(Mutex::new(None)),
            _stream: stream,
            _stream_handle: stream_handle,
            current_track: Arc::new(Mutex::new(None)),
        }
    }

    pub fn play(&self, path: &Path, title: String, artist: String) -> Result<()> {
        // Use the existing stream handle from the struct
        let stream_handle = self._stream_handle.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No audio output stream available"))?;
        
        let sink = Sink::try_new(stream_handle)?;

        // Load the audio file
        let file = File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;
        
        // Play the audio
        sink.append(source);
        
        // Store the sink
        if let Ok(mut guard) = self.sink.lock() {
            *guard = Some(sink);
        }

        // Store current track info
        if let Ok(mut track) = self.current_track.lock() {
            *track = Some(TrackInfo { title, artist });
        }

        Ok(())
    }

    pub fn toggle_pause(&self) {
        if let Ok(guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
        }
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.sink.lock() {
            if let Some(sink) = guard.take() {
                sink.stop();
            }
        }
        
        if let Ok(mut track) = self.current_track.lock() {
            *track = None;
        }
    }

    pub fn is_playing(&self) -> bool {
        if let Ok(guard) = self.sink.lock() {
            if let Some(sink) = guard.as_ref() {
                return !sink.is_paused() && !sink.empty();
            }
        }
        false
    }

    pub fn current_track(&self) -> Option<TrackInfo> {
        if let Ok(track) = self.current_track.lock() {
            track.clone()
        } else {
            None
        }
    }
}
