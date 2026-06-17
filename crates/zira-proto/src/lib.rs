//! zira-proto — shared types: Emotion, State, Event, payloads.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Emotion {
    #[default]
    Neutral,
    Happy,
    Sad,
    Angry,
    Excited,
    Calm,
    Curious,
    Concerned,
    Playful,
    Tired,
}

impl Emotion {
    pub fn from_tag(tag: &str) -> Self {
        match tag.to_ascii_lowercase().as_str() {
            "neutral" => Self::Neutral,
            "happy" => Self::Happy,
            "sad" => Self::Sad,
            "angry" => Self::Angry,
            "excited" => Self::Excited,
            "calm" => Self::Calm,
            "curious" => Self::Curious,
            "concerned" => Self::Concerned,
            "playful" => Self::Playful,
            "tired" => Self::Tired,
            _ => Self::Neutral,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum State {
    #[default]
    Idle,
    Listening,
    Transcribing,
    Thinking,
    PlanReview,
    Speaking,
}
