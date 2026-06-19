//! T-01.16 — voice-activity detection over a 16 kHz PCM buffer via earshot (a pure-Rust
//! WebRTC VAD). Live microphone capture is the device-bound source; the gate itself
//! operates on a supplied buffer, so it is verifiable with a fixture.

use earshot::{VoiceActivityDetector, VoiceActivityProfile};
use zira_core::VadGate;
use zira_proto::Event;

/// 10 ms frame at 16 kHz.
const FRAME: usize = 160;

/// A [`VadGate`] backed by earshot. Hold the captured 16 kHz mono PCM (`with_audio`) and
/// pull speech boundaries with [`VadGate::next_activity`], or scan a whole buffer at once
/// with [`EarshotVad::scan_16khz`].
pub struct EarshotVad {
    vad: VoiceActivityDetector,
    audio: Vec<i16>,
    pos: usize,
    voiced: bool,
}

impl EarshotVad {
    /// A fresh detector tuned to the aggressive profile (favours precision on clean audio).
    pub fn new() -> Self {
        Self {
            vad: VoiceActivityDetector::new(VoiceActivityProfile::AGGRESSIVE),
            audio: Vec::new(),
            pos: 0,
            voiced: false,
        }
    }

    /// Hold a 16 kHz mono PCM buffer for the streaming [`VadGate`] interface.
    #[must_use]
    pub fn with_audio(mut self, audio: Vec<i16>) -> Self {
        self.audio = audio;
        self
    }

    /// Predict whether one exactly-`FRAME`-sample frame is voiced (an error is treated as
    /// unvoiced — a conservative gate).
    fn frame_voiced(&mut self, frame: &[i16]) -> bool {
        self.vad.predict_16khz(frame).unwrap_or(false)
    }

    /// Scan a 16 kHz mono PCM buffer in 10 ms frames and return the speech-boundary events:
    /// an [`Event::SpeechStarted`] at the onset of voiced frames and an
    /// [`Event::SpeechEnded`] when voicing ends. A trailing partial frame is ignored.
    pub fn scan_16khz(&mut self, pcm: &[i16]) -> Vec<Event> {
        let mut events = Vec::new();
        let mut voiced = false;
        for frame in pcm.chunks(FRAME) {
            if frame.len() < FRAME {
                break;
            }
            let v = self.frame_voiced(frame);
            if v && !voiced {
                events.push(Event::SpeechStarted);
                voiced = true;
            } else if !v && voiced {
                events.push(Event::SpeechEnded);
                voiced = false;
            }
        }
        events
    }

    /// Fraction of full frames in `pcm` detected voiced, in `0.0..=1.0`.
    pub fn voiced_ratio(&mut self, pcm: &[i16]) -> f32 {
        let (mut voiced, mut total) = (0usize, 0usize);
        for frame in pcm.chunks(FRAME) {
            if frame.len() < FRAME {
                break;
            }
            total += 1;
            if self.frame_voiced(frame) {
                voiced += 1;
            }
        }
        if total == 0 {
            0.0
        } else {
            voiced as f32 / total as f32
        }
    }
}

impl Default for EarshotVad {
    fn default() -> Self {
        Self::new()
    }
}

impl VadGate for EarshotVad {
    /// Advance through the held audio one frame at a time, returning the next speech
    /// boundary. When the audio is exhausted, close any open utterance with
    /// [`Event::SpeechEnded`].
    async fn next_activity(&mut self) -> Event {
        while self.pos + FRAME <= self.audio.len() {
            let frame: Vec<i16> = self.audio[self.pos..self.pos + FRAME].to_vec();
            self.pos += FRAME;
            let v = self.frame_voiced(&frame);
            if v && !self.voiced {
                self.voiced = true;
                return Event::SpeechStarted;
            }
            if !v && self.voiced {
                self.voiced = false;
                return Event::SpeechEnded;
            }
        }
        self.voiced = false;
        Event::SpeechEnded
    }
}
