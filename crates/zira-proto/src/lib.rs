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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub emotion: Emotion,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisemeFrame {
    pub viseme: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub description: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    WakeDetected,
    SpeechStarted,
    SpeechEnded,
    AudioChunk,
    TranscriptReady(Transcript),
    TurnStarted,
    TextDelta,
    EmotionSegment(Segment),
    PlanReady,
    SpeakRequest,
    VisemeFrame(VisemeFrame),
    ExpressionChange,
    BargeIn,
    TurnComplete(Usage),
    Error(String),
}
