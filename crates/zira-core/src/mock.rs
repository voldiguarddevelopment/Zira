//! Deterministic, hardware-free mocks of the seven stage traits.
//!
//! Each mock is a pure function of the script handed to its constructor — no mic,
//! speaker, GPU, FFI model, clock, or randomness is touched. Driving a mock through its
//! trait method replays the scripted [`Event`](zira_proto::Event)s the orchestrator
//! would see from the real engine, so the full Idle→…→Idle cycle can be exercised
//! offline. Gated behind the `mock` feature so the production library never ships them.

use zira_proto::{Event, Segment, Transcript, VisemeFrame};

use crate::{AvatarSink, Brain, MemoryStore, SttEngine, TtsEngine, VadGate, WakeSource};

/// Scripted [`WakeSource`]: every `next_wake` yields [`Event::WakeDetected`].
#[derive(Debug, Default)]
pub struct MockWakeSource;

impl MockWakeSource {
    /// Construct the mock.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl WakeSource for MockWakeSource {
    async fn next_wake(&mut self) -> Event {
        Event::WakeDetected
    }
}

/// Scripted [`VadGate`]: first call yields [`Event::SpeechStarted`], the next
/// [`Event::SpeechEnded`], framing exactly one utterance.
#[derive(Debug, Default)]
pub struct MockVadGate {
    started: bool,
}

impl MockVadGate {
    /// Construct the mock at the start of an utterance.
    #[must_use]
    pub fn new() -> Self {
        Self { started: false }
    }
}

impl VadGate for MockVadGate {
    async fn next_activity(&mut self) -> Event {
        if self.started {
            Event::SpeechEnded
        } else {
            self.started = true;
            Event::SpeechStarted
        }
    }
}

/// Scripted [`SttEngine`]: `transcribe` yields the scripted text as a
/// [`Event::TranscriptReady`].
#[derive(Debug, Clone)]
pub struct MockSttEngine {
    text: String,
}

impl MockSttEngine {
    /// Construct the mock with the transcript it will emit.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl SttEngine for MockSttEngine {
    async fn transcribe(&mut self) -> Event {
        Event::TranscriptReady(Transcript {
            text: self.text.clone(),
        })
    }
}

/// Scripted [`Brain`]: `respond` yields the scripted reply as an
/// [`Event::EmotionSegment`] followed by a [`Event::SpeakRequest`].
#[derive(Debug, Clone)]
pub struct MockBrain {
    reply: Segment,
}

impl MockBrain {
    /// Construct the mock with the reply segment it will emit.
    #[must_use]
    pub fn new(reply: Segment) -> Self {
        Self { reply }
    }
}

impl Brain for MockBrain {
    async fn respond(&mut self) -> Vec<Event> {
        vec![
            Event::EmotionSegment(self.reply.clone()),
            Event::SpeakRequest,
        ]
    }
}

/// Scripted [`TtsEngine`]: `speak` yields one [`Event::VisemeFrame`] per scripted frame,
/// in order.
#[derive(Debug, Clone)]
pub struct MockTtsEngine {
    frames: Vec<VisemeFrame>,
}

impl MockTtsEngine {
    /// Construct the mock with the viseme frames it will emit.
    #[must_use]
    pub fn new(frames: Vec<VisemeFrame>) -> Self {
        Self { frames }
    }
}

impl TtsEngine for MockTtsEngine {
    async fn speak(&mut self) -> Vec<Event> {
        self.frames
            .iter()
            .cloned()
            .map(Event::VisemeFrame)
            .collect()
    }
}

/// Scripted [`AvatarSink`]: every `render` yields [`Event::ExpressionChange`].
#[derive(Debug, Default)]
pub struct MockAvatarSink;

impl MockAvatarSink {
    /// Construct the mock.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl AvatarSink for MockAvatarSink {
    async fn render(&mut self) -> Event {
        Event::ExpressionChange
    }
}

/// In-memory [`MemoryStore`]: `store` appends, `recall` returns everything appended, so
/// what goes in round-trips out unchanged.
#[derive(Debug, Default)]
pub struct MockMemoryStore {
    stored: Vec<Event>,
}

impl MockMemoryStore {
    /// Construct an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self { stored: Vec::new() }
    }
}

impl MemoryStore for MockMemoryStore {
    async fn store(&mut self, event: Event) {
        self.stored.push(event);
    }

    async fn recall(&mut self) -> Vec<Event> {
        self.stored.clone()
    }
}
