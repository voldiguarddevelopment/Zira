//! T-01.15 — wake-word detection over a 16 kHz PCM buffer via rustpotter (a pure-Rust
//! MFCC detector). Live microphone capture is the device-bound source; detection itself
//! runs on a supplied buffer + a wake-word model, so it is verifiable with a fixture.

use rustpotter::{Rustpotter, RustpotterConfig};
use std::path::Path;
use thiserror::Error;
use zira_core::WakeSource;
use zira_proto::Event;

/// Failure loading or configuring the wake-word detector.
#[derive(Debug, Error)]
pub enum WakeError {
    /// The Rustpotter engine could not be constructed.
    #[error("wake engine init failed: {0}")]
    Init(String),
    /// The wake-word model file could not be loaded.
    #[error("wake model load failed: {0}")]
    ModelLoad(String),
}

/// A [`WakeSource`] backed by a rustpotter wake-word model. Hold captured 16 kHz mono PCM
/// (`with_audio`) and await detections via [`WakeSource::next_wake`], or scan a whole
/// buffer with [`RustpotterWake::detect`].
pub struct RustpotterWake {
    rp: Rustpotter,
    audio: Vec<i16>,
    pos: usize,
}

impl RustpotterWake {
    /// Load a wake-word model (`.rpw`) into a fresh detector.
    pub fn load(model_path: &Path) -> Result<Self, WakeError> {
        let mut rp =
            Rustpotter::new(&RustpotterConfig::default()).map_err(WakeError::Init)?;
        let path = model_path.to_string_lossy();
        rp.add_wakeword_from_file("wake", &path)
            .map_err(WakeError::ModelLoad)?;
        Ok(Self {
            rp,
            audio: Vec::new(),
            pos: 0,
        })
    }

    /// Hold a 16 kHz mono PCM buffer for the streaming [`WakeSource`] interface.
    #[must_use]
    pub fn with_audio(mut self, audio: Vec<i16>) -> Self {
        self.audio = audio;
        self
    }

    /// Scan a 16 kHz mono PCM buffer frame by frame; return the detection score of the
    /// first wake-word match, or `None` if the phrase is never detected.
    pub fn detect(&mut self, pcm: &[i16]) -> Option<f32> {
        let frame = self.rp.get_samples_per_frame();
        for chunk in pcm.chunks(frame) {
            if chunk.len() < frame {
                break;
            }
            if let Some(d) = self.rp.process_samples(chunk.to_vec()) {
                return Some(d.score);
            }
        }
        None
    }
}

impl WakeSource for RustpotterWake {
    /// Advance through the held audio until the wake phrase is detected, emitting
    /// [`Event::WakeDetected`]. When the audio is exhausted with no detection, also returns
    /// `WakeDetected` is NOT forced — instead it returns the terminal boundary so callers
    /// can decide; here we simply return `WakeDetected` only on a real match and otherwise
    /// emit `Event::Error` when the buffer is exhausted.
    async fn next_wake(&mut self) -> Event {
        let frame = self.rp.get_samples_per_frame();
        while self.pos + frame <= self.audio.len() {
            let chunk = self.audio[self.pos..self.pos + frame].to_vec();
            self.pos += frame;
            if self.rp.process_samples(chunk).is_some() {
                return Event::WakeDetected;
            }
        }
        Event::Error("wake audio exhausted with no detection".to_string())
    }
}
