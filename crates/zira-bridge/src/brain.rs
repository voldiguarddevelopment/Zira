//! `ClaudeBrain` — the real [`Brain`](zira_core::Brain) implementation.
//!
//! Drives the `claude` CLI via [`crate::ask`] and segments the answer through
//! `zira_emotion::segment`, emitting one [`Event::EmotionSegment`] per span
//! followed by exactly one [`Event::TurnComplete`].

use zira_config::ZiraConfig;
use zira_core::Brain;
use zira_proto::{Event, Transcript};

/// The production `Brain`: calls the Claude CLI for each turn and segments the
/// reply text into emotion-tagged spans.
pub struct ClaudeBrain {
    cfg: ZiraConfig,
    constitution: String,
    transcript: Transcript,
}

impl ClaudeBrain {
    /// Construct a `ClaudeBrain` ready to respond to one transcript turn.
    pub fn new(cfg: ZiraConfig, constitution: &str, transcript: Transcript) -> Self {
        Self {
            cfg,
            constitution: constitution.to_string(),
            transcript,
        }
    }
}

impl Brain for ClaudeBrain {
    async fn respond(&mut self) -> Vec<Event> {
        match crate::ask(&self.cfg, &self.constitution, &self.transcript) {
            Ok(answer) => {
                let segments = zira_emotion::segment(&answer.text);
                let mut events: Vec<Event> = segments
                    .into_iter()
                    .map(Event::EmotionSegment)
                    .collect();
                events.push(Event::TurnComplete(answer.usage));
                events
            }
            Err(e) => {
                vec![Event::Error(e.to_string())]
            }
        }
    }
}
