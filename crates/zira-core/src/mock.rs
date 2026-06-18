//! Deterministic, hardware-free mocks of the seven stage traits.
//!
//! Each mock is a pure function of the script handed to its constructor — no mic,
//! speaker, GPU, FFI model, clock, or randomness is touched. Driving a mock through its
//! trait method replays the scripted [`Event`](zira_proto::Event)s the orchestrator
//! would see from the real engine, so the full Idle→…→Idle cycle can be exercised
//! offline. Gated behind the `mock` feature so the production library never ships them.

use tokio::sync::{mpsc, watch};
use zira_proto::{Emotion, Event, Segment, State, Transcript, VisemeFrame};

use crate::{
    create_bus, AvatarSink, Brain, BusHandles, MemoryStore, Orchestrator, SttEngine, TtsEngine,
    VadGate, WakeSource,
};

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

/// The assembled mock conversation cycle: a real [`Orchestrator`] wired to the seven
/// scripted stage mocks, ready to be driven through a full turn offline.
///
/// `new` constructs the genuine command/event bus, builds an [`Orchestrator`] over it,
/// and spawns its `run` select-loop on the current Tokio runtime. The seven mocks are
/// exposed as public fields so a test can pump each through its stage **trait** method
/// and feed every emitted [`Event`] back across the command bus via [`MockCycle::cmd`],
/// observing the resulting state path through [`MockCycle::subscribe_state`]. Dropping the
/// cycle drops the last command sender, which closes the channel and lets `run` exit.
pub struct MockCycle {
    /// Wake-word source mock (`Idle -> Listening`).
    pub wake: MockWakeSource,
    /// Voice-activity gate mock (`Listening -> Transcribing`).
    pub vad: MockVadGate,
    /// Speech-to-text mock (`Transcribing -> Thinking`).
    pub stt: MockSttEngine,
    /// Brain mock (`Thinking -> Speaking`).
    pub brain: MockBrain,
    /// Text-to-speech mock (viseme side-effects while `Speaking`).
    pub tts: MockTtsEngine,
    /// Avatar sink mock (expression side-effect while `Speaking`).
    pub avatar: MockAvatarSink,
    /// In-memory store mock (persist/recall side-effects).
    pub memory: MockMemoryStore,
    cmd_tx: mpsc::Sender<Event>,
    state_rx: watch::Receiver<State>,
}

impl MockCycle {
    /// Assemble the orchestrator from the seven mock stages and spawn its run-loop.
    ///
    /// Must be called from within a Tokio runtime (the run-loop is spawned as a task).
    #[must_use]
    pub fn new() -> Self {
        let BusHandles {
            cmd_tx,
            cmd_rx,
            event_tx,
        } = create_bus();
        let mut orchestrator = Orchestrator::new(cmd_rx, event_tx);
        let state_rx = orchestrator.subscribe_state();
        tokio::spawn(async move {
            orchestrator.run().await;
        });

        Self {
            wake: MockWakeSource::new(),
            vad: MockVadGate::new(),
            stt: MockSttEngine::new("hello zira"),
            brain: MockBrain::new(Segment {
                emotion: Emotion::Neutral,
                text: "hello there".to_string(),
            }),
            tts: MockTtsEngine::new(vec![VisemeFrame {
                viseme: "AA".to_string(),
                weight: 1.0,
            }]),
            avatar: MockAvatarSink::new(),
            memory: MockMemoryStore::new(),
            cmd_tx,
            state_rx,
        }
    }

    /// A cloneable sender on the orchestrator's command bus — send each event the mocks
    /// emit (and the control events they do not) here to drive the state machine.
    #[must_use]
    pub fn cmd(&self) -> mpsc::Sender<Event> {
        self.cmd_tx.clone()
    }

    /// A receiver that observes the orchestrator's state, seeded with the initial
    /// [`State::Idle`]; `changed().await` resolves once per state transition.
    #[must_use]
    pub fn subscribe_state(&self) -> watch::Receiver<State> {
        self.state_rx.clone()
    }
}

impl Default for MockCycle {
    fn default() -> Self {
        Self::new()
    }
}
