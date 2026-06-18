//! Frozen tests for T-00.21 — "Integrate the mock cycle".
//!
//! Phase-0 acceptance: the whole loop cycles on mocks. This proves the *wiring* — the
//! state machine + event bus + stage traits compose correctly — not the devices.
//!
//! The seam under test is [`zira_core::mock::MockCycle`]: it **assembles** a real
//! [`Orchestrator`](zira_core::Orchestrator) from the seven mock stages, spawning the
//! genuine `Orchestrator::run` select-loop over the genuine mpsc command bus + broadcast
//! event bus. The test then *drives* a full conversation turn by pumping each mock
//! through its stage **trait** method (never an inherent method — the calls below resolve
//! through `WakeSource`/`VadGate`/`SttEngine`/`Brain`/`TtsEngine`/`AvatarSink`/
//! `MemoryStore`) and feeding every emitted event across the bus, while injecting the
//! control events the stages do not themselves emit (`TurnComplete` closes a turn,
//! `BargeIn` interrupts one). The exact state path is recorded losslessly with the proven
//! one-event-per-`changed()` pattern, so "the path is exactly that sequence" is decidable.
//!
//! Criterion → test mapping:
//!
//!   C1 (the orchestrator can be assembled from the seven mock stages and run
//!        end-to-end on injected events)
//!        -> c1_orchestrator_assembled_from_mocks_runs_end_to_end
//!   C2 (a full Idle -> Listening -> Transcribing -> Thinking -> Speaking -> Idle cycle
//!        through the mocked stages, asserting the state path is exactly that sequence)
//!        -> c2_happy_path_state_sequence_is_exact
//!   C3 (a barge-in during Speaking drives the mocked cycle back to Listening)
//!        -> c3_barge_in_during_speaking_returns_to_listening

use zira_core::mock::MockCycle;
use zira_core::{AvatarSink, Brain, MemoryStore, SttEngine, TtsEngine, VadGate, WakeSource};
use zira_proto::{Event, State, Transcript, Usage};

/// Drive the assembled mock cycle from `Idle` through to `Speaking` by pumping the
/// stages whose mocks source the happy-path events, then inject `at_speaking` (the one
/// control event the stages do not emit) and await its transition. Returns the exact
/// ordered sequence of states the orchestrator entered, beginning with the initial
/// `Idle`.
///
/// Every `changed().await` is preceded by sending exactly the events that cause exactly
/// one transition before the next await; events with no row in the transition table
/// (`SpeechStarted`, `EmotionSegment`, the viseme/expression side-effects) are sent
/// without awaiting, because they bump no state and so are skipped by `changed()`.
async fn run_scenario(at_speaking: Event) -> Vec<State> {
    // C1: assemble the orchestrator from the seven mock stages (real run-loop + bus).
    let mut cycle = MockCycle::new();
    let cmd = cycle.cmd();
    let mut state_rx = cycle.subscribe_state();

    let mut path = vec![*state_rx.borrow()]; // initial Idle

    // ── Idle -> Listening : WakeSource emits WakeDetected ───────────────────────────
    let wake = cycle.wake.next_wake().await;
    cmd.send(wake).await.unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    // ── Listening -> Transcribing : VadGate frames the utterance ────────────────────
    // First boundary (SpeechStarted) has no row for Listening → ignored, no transition.
    let started = cycle.vad.next_activity().await;
    cmd.send(started).await.unwrap();
    let ended = cycle.vad.next_activity().await;
    cmd.send(ended).await.unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    // ── Transcribing -> Thinking : SttEngine emits the transcript ───────────────────
    let transcript = cycle.stt.transcribe().await;
    cmd.send(transcript).await.unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    // ── Thinking -> Speaking : Brain emits EmotionSegment then SpeakRequest ─────────
    // EmotionSegment has no row for Thinking → ignored; SpeakRequest drives the change.
    for event in cycle.brain.respond().await {
        cmd.send(event).await.unwrap();
    }
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    // ── Side-effect stages while Speaking : prove all seven mocks are wired. None of
    //    these events has a row for Speaking, so they pass through without transition. ─
    for frame in cycle.tts.speak().await {
        cmd.send(frame).await.unwrap();
    }
    let expression = cycle.avatar.render().await;
    cmd.send(expression).await.unwrap();
    cycle
        .memory
        .store(Event::TranscriptReady(Transcript {
            text: "hello zira".to_string(),
        }))
        .await;
    let _recalled = cycle.memory.recall().await;

    // ── Speaking -> (injected) : the control event the stages do not emit ───────────
    cmd.send(at_speaking).await.unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    path
}

/// C1: the orchestrator assembles from the seven mock stages and runs a full turn
/// end-to-end on the injected events — a complete round trip back to `Idle`, passing
/// through every pipeline state in between.
#[tokio::test]
async fn c1_orchestrator_assembled_from_mocks_runs_end_to_end() {
    let path = run_scenario(Event::TurnComplete(Usage {
        input_tokens: 5,
        output_tokens: 10,
    }))
    .await;

    assert_eq!(
        path.first(),
        Some(&State::Idle),
        "the assembled cycle must start in Idle"
    );
    assert_eq!(
        path.last(),
        Some(&State::Idle),
        "a full turn driven end-to-end must return to Idle"
    );
    assert_eq!(
        path.len(),
        6,
        "a full round trip enters exactly six states (the closing Idle included)"
    );
    for expected in [
        State::Listening,
        State::Transcribing,
        State::Thinking,
        State::Speaking,
    ] {
        assert!(
            path.contains(&expected),
            "end-to-end run must pass through {expected:?}; path was {path:?}"
        );
    }
}

/// C2: the happy-path state sequence is exactly
/// `Idle -> Listening -> Transcribing -> Thinking -> Speaking -> Idle`.
#[tokio::test]
async fn c2_happy_path_state_sequence_is_exact() {
    let path = run_scenario(Event::TurnComplete(Usage {
        input_tokens: 5,
        output_tokens: 10,
    }))
    .await;

    assert_eq!(
        path,
        vec![
            State::Idle,
            State::Listening,
            State::Transcribing,
            State::Thinking,
            State::Speaking,
            State::Idle,
        ],
        "the mocked cycle must follow exactly the canonical happy path"
    );
}

/// C3: a `BargeIn` injected while `Speaking` drives the mocked cycle back to `Listening`
/// (not `Idle`), proving barge-in re-enters Listening. Asserted as the exact path so the
/// final transition is unambiguously `Speaking -> Listening`.
#[tokio::test]
async fn c3_barge_in_during_speaking_returns_to_listening() {
    let path = run_scenario(Event::BargeIn).await;

    assert_eq!(
        path,
        vec![
            State::Idle,
            State::Listening,
            State::Transcribing,
            State::Thinking,
            State::Speaking,
            State::Listening,
        ],
        "barge-in during Speaking must re-enter Listening"
    );
    assert_eq!(
        path.last(),
        Some(&State::Listening),
        "the cycle must end in Listening after a barge-in, never Idle"
    );
    assert_eq!(
        path[path.len() - 2],
        State::Speaking,
        "the barge-in must fire from Speaking",
    );
}
