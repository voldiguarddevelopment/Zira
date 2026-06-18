//! Frozen tests for T-00.20 — "Define the stage traits".
//!
//! The seam that lets real devices be swapped behind a mock: `zira_core` defines the
//! seven stage traits the orchestrator drives, and a deterministic mock of each emits
//! scripted [`Event`]s without touching any mic / speaker / GPU / FFI model.
//!
//! Criterion → test mapping:
//!
//!   C1 (traits defined, each with the minimal async method(s) the orchestrator needs)
//!        -> c1_wake_source_trait_drives_mock,
//!           c1_vad_gate_trait_emits_speech_boundaries,
//!           c1_stt_engine_trait_emits_transcript,
//!           c1_brain_trait_emits_response_stream,
//!           c1_tts_engine_trait_emits_visemes,
//!           c1_avatar_sink_trait_emits_expression_change,
//!           c1_memory_store_trait_round_trips_event
//!   C2 (a mock of each trait exists, emits scripted events, no hardware, deterministic)
//!        -> c2_mock_stt_is_deterministic,
//!           c2_mock_brain_is_deterministic,
//!           (plus every c1_* test, each of which drives a mock)
//!   C3 (this repo-root integration test drives each mock through its trait method and
//!        asserts the expected scripted Event(s))
//!        -> every test in this file
//!
//! Design note: every stage is exercised **through a trait-bounded generic driver**
//! (the `drive_*` helpers below), never through an inherent method on the concrete
//! mock. Because the driver only knows `S: SttEngine` (etc.), the call can ONLY resolve
//! to the trait method — proving the trait exists with that signature (C1) and that the
//! orchestrator can depend on the trait, never the concrete engine. The mocks are
//! reached via `zira_core::mock::*`, the test-support module GREEN must expose.

use zira_core::mock::{
    MockAvatarSink, MockBrain, MockMemoryStore, MockSttEngine, MockTtsEngine, MockVadGate,
    MockWakeSource,
};
use zira_core::{AvatarSink, Brain, MemoryStore, SttEngine, TtsEngine, VadGate, WakeSource};
use zira_proto::{Emotion, Event, Segment, Transcript, VisemeFrame};

// ── Trait-bounded generic drivers ────────────────────────────────────────────
//
// Each driver is generic over ONLY the trait, so the method call cannot bind to an
// inherent method on the mock — it must be the trait method. This is what makes the
// invariant "the orchestrator depends on traits, never concrete engines" testable.

async fn drive_wake<W: WakeSource>(w: &mut W) -> Event {
    w.next_wake().await
}

async fn drive_vad<V: VadGate>(v: &mut V) -> Event {
    v.next_activity().await
}

async fn drive_stt<S: SttEngine>(s: &mut S) -> Event {
    s.transcribe().await
}

async fn drive_brain<B: Brain>(b: &mut B) -> Vec<Event> {
    b.respond().await
}

async fn drive_tts<T: TtsEngine>(t: &mut T) -> Vec<Event> {
    t.speak().await
}

async fn drive_avatar<A: AvatarSink>(a: &mut A) -> Event {
    a.render().await
}

async fn drive_memory<M: MemoryStore>(m: &mut M, persist: Event) -> Vec<Event> {
    m.store(persist).await;
    m.recall().await
}

// ── C1: each trait exists and its mock, driven through it, emits the right Event ─────

/// `WakeSource::next_wake` yields the `WakeDetected` event that lifts Idle -> Listening.
#[tokio::test]
async fn c1_wake_source_trait_drives_mock() {
    let mut wake = MockWakeSource::new();
    let event = drive_wake(&mut wake).await;
    assert!(
        matches!(event, Event::WakeDetected),
        "WakeSource mock should emit WakeDetected, got {event:?}",
    );
}

/// `VadGate::next_activity` yields the speech boundary events in order:
/// `SpeechStarted` on the first call, then `SpeechEnded` on the second.
#[tokio::test]
async fn c1_vad_gate_trait_emits_speech_boundaries() {
    let mut vad = MockVadGate::new();

    let first = drive_vad(&mut vad).await;
    assert!(
        matches!(first, Event::SpeechStarted),
        "first VAD activity should be SpeechStarted, got {first:?}",
    );

    let second = drive_vad(&mut vad).await;
    assert!(
        matches!(second, Event::SpeechEnded),
        "second VAD activity should be SpeechEnded, got {second:?}",
    );
}

/// `SttEngine::transcribe` yields a `TranscriptReady` carrying the scripted text,
/// proving the payload flows through the trait (not a hardcoded constant).
#[tokio::test]
async fn c1_stt_engine_trait_emits_transcript() {
    let mut stt = MockSttEngine::new("hello zira");
    let event = drive_stt(&mut stt).await;
    match event {
        Event::TranscriptReady(Transcript { text }) => {
            assert_eq!(text, "hello zira", "transcript text should match the script");
        }
        other => panic!("SttEngine mock should emit TranscriptReady, got {other:?}"),
    }
}

/// `Brain::respond` yields the scripted response stream: an `EmotionSegment` carrying
/// the reply (emotion + text) followed by a `SpeakRequest` that drives Thinking ->
/// Speaking. Order and payload are both asserted.
#[tokio::test]
async fn c1_brain_trait_emits_response_stream() {
    let reply = Segment {
        emotion: Emotion::Happy,
        text: "hi there".to_string(),
    };
    let mut brain = MockBrain::new(reply);
    let events = drive_brain(&mut brain).await;

    assert_eq!(events.len(), 2, "brain should emit a 2-event response stream");

    match &events[0] {
        Event::EmotionSegment(Segment { emotion, text }) => {
            assert_eq!(*emotion, Emotion::Happy, "segment emotion should match script");
            assert_eq!(text, "hi there", "segment text should match script");
        }
        other => panic!("first brain event should be EmotionSegment, got {other:?}"),
    }

    assert!(
        matches!(events[1], Event::SpeakRequest),
        "second brain event should be SpeakRequest, got {:?}",
        events[1],
    );
}

/// `TtsEngine::speak` yields one `VisemeFrame` event per scripted frame, in order,
/// each carrying its viseme label and weight (lip-sync timing for the avatar).
#[tokio::test]
async fn c1_tts_engine_trait_emits_visemes() {
    let frames = vec![
        VisemeFrame {
            viseme: "AA".to_string(),
            weight: 0.8,
        },
        VisemeFrame {
            viseme: "OH".to_string(),
            weight: 0.5,
        },
    ];
    let mut tts = MockTtsEngine::new(frames);
    let events = drive_tts(&mut tts).await;

    assert_eq!(events.len(), 2, "tts should emit one event per scripted frame");

    match &events[0] {
        Event::VisemeFrame(VisemeFrame { viseme, weight }) => {
            assert_eq!(viseme, "AA", "first viseme label should match script");
            assert!((*weight - 0.8).abs() < f32::EPSILON, "first viseme weight should match");
        }
        other => panic!("first tts event should be VisemeFrame, got {other:?}"),
    }

    match &events[1] {
        Event::VisemeFrame(VisemeFrame { viseme, weight }) => {
            assert_eq!(viseme, "OH", "second viseme label should match script");
            assert!((*weight - 0.5).abs() < f32::EPSILON, "second viseme weight should match");
        }
        other => panic!("second tts event should be VisemeFrame, got {other:?}"),
    }
}

/// `AvatarSink::render` yields an `ExpressionChange` — the avatar acknowledging it
/// applied the requested expression preset.
#[tokio::test]
async fn c1_avatar_sink_trait_emits_expression_change() {
    let mut avatar = MockAvatarSink::new();
    let event = drive_avatar(&mut avatar).await;
    assert!(
        matches!(event, Event::ExpressionChange),
        "AvatarSink mock should emit ExpressionChange, got {event:?}",
    );
}

/// `MemoryStore::store` then `MemoryStore::recall` round-trips the persisted event:
/// what is stored is what is later recalled, proving the seam carries real data.
#[tokio::test]
async fn c1_memory_store_trait_round_trips_event() {
    let mut memory = MockMemoryStore::new();
    let persisted = Event::TranscriptReady(Transcript {
        text: "remember me".to_string(),
    });

    let recalled = drive_memory(&mut memory, persisted).await;

    assert_eq!(recalled.len(), 1, "recall should return the single stored event");
    match &recalled[0] {
        Event::TranscriptReady(Transcript { text }) => {
            assert_eq!(text, "remember me", "recalled event should match what was stored");
        }
        other => panic!("recalled event should be the stored TranscriptReady, got {other:?}"),
    }
}

// ── C2: the mocks are deterministic (no hardware, no clock, no randomness) ────────────

/// Two independently-constructed `MockSttEngine`s with the same script emit identical
/// output — the mock is a pure function of its script, not of any device state.
#[tokio::test]
async fn c2_mock_stt_is_deterministic() {
    let mut a = MockSttEngine::new("same input");
    let mut b = MockSttEngine::new("same input");

    let ea = drive_stt(&mut a).await;
    let eb = drive_stt(&mut b).await;

    let ta = match ea {
        Event::TranscriptReady(Transcript { text }) => text,
        other => panic!("expected TranscriptReady, got {other:?}"),
    };
    let tb = match eb {
        Event::TranscriptReady(Transcript { text }) => text,
        other => panic!("expected TranscriptReady, got {other:?}"),
    };
    assert_eq!(ta, tb, "identically-scripted STT mocks must produce identical output");
    assert_eq!(ta, "same input", "deterministic output must equal the script");
}

/// Two independently-constructed `MockBrain`s with the same script emit the same
/// length and the same leading `EmotionSegment` payload — deterministic, no randomness.
#[tokio::test]
async fn c2_mock_brain_is_deterministic() {
    let script = Segment {
        emotion: Emotion::Calm,
        text: "deterministic".to_string(),
    };
    let mut a = MockBrain::new(script.clone());
    let mut b = MockBrain::new(script);

    let ea = drive_brain(&mut a).await;
    let eb = drive_brain(&mut b).await;

    assert_eq!(ea.len(), eb.len(), "identically-scripted brains must emit equal-length streams");

    let pa = match &ea[0] {
        Event::EmotionSegment(seg) => (seg.emotion, seg.text.clone()),
        other => panic!("expected leading EmotionSegment, got {other:?}"),
    };
    let pb = match &eb[0] {
        Event::EmotionSegment(seg) => (seg.emotion, seg.text.clone()),
        other => panic!("expected leading EmotionSegment, got {other:?}"),
    };
    assert_eq!(pa, pb, "identically-scripted brains must emit identical leading segment");
    assert_eq!(pa, (Emotion::Calm, "deterministic".to_string()), "output must equal the script");
}
