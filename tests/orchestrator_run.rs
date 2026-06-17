//! Frozen tests for T-00.17 — "Run the orchestrator loop".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_run_advances_state_on_defined_transition
//!   C2 -> c2_undefined_transition_leaves_state_unchanged,
//!          c2_channel_close_exits_loop_cleanly
//!   C3 -> c3_scripted_sequence_advances_through_expected_states

use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use zira_core::Orchestrator;
use zira_proto::{Event, State, Transcript, Usage};

// ── C1: run() consumes events and updates state on each defined transition ───────────

/// C1: a single defined event sent on the command channel causes `run()` to apply
/// `next_state` and update the held `State`.
///
/// WakeDetected is sent while the orchestrator is in `Idle`; the state must become
/// `Listening` and the change must be observable via `subscribe_state()`.
#[tokio::test]
async fn c1_run_advances_state_on_defined_transition() {
    let (event_tx, _) = broadcast::channel::<Event>(16);
    let (cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let mut orch = Orchestrator::new(cmd_rx, event_tx);
    let mut state_rx = orch.subscribe_state();

    let handle = tokio::spawn(async move {
        orch.run().await;
    });

    cmd_tx.send(Event::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("state did not change after WakeDetected");
    assert_eq!(
        *state_rx.borrow(),
        State::Listening,
        "WakeDetected from Idle must transition to Listening"
    );

    drop(cmd_tx);
    handle.await.unwrap();
}

// ── C2: undefined transition is ignored; channel close exits cleanly ─────────────────

/// C2 (undefined transition): an event with no entry in the transition table for the
/// current state must be silently ignored — the state must remain unchanged and the
/// select-loop must keep running.
///
/// SpeechEnded has no row in the table for `Idle`.  The test confirms no state change
/// is published within 100 ms, then confirms the loop is still alive by processing a
/// subsequent WakeDetected.
#[tokio::test]
async fn c2_undefined_transition_leaves_state_unchanged() {
    let (event_tx, _) = broadcast::channel::<Event>(16);
    let (cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let mut orch = Orchestrator::new(cmd_rx, event_tx);
    let mut state_rx = orch.subscribe_state();

    let handle = tokio::spawn(async move {
        orch.run().await;
    });

    // SpeechEnded from Idle is not in the transition table → must be silently ignored.
    cmd_tx.send(Event::SpeechEnded).await.unwrap();

    // No state change notification must arrive within a short window.
    let no_change = tokio::time::timeout(Duration::from_millis(100), state_rx.changed()).await;
    assert!(
        no_change.is_err(),
        "undefined transition must not emit a state change notification"
    );
    assert_eq!(
        *state_rx.borrow(),
        State::Idle,
        "state must remain Idle after an undefined transition"
    );

    // The loop must still be running: a subsequent defined event is processed normally.
    cmd_tx.send(Event::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("loop must continue after an undefined transition");
    assert_eq!(
        *state_rx.borrow(),
        State::Listening,
        "WakeDetected after an ignored undefined event must still advance to Listening"
    );

    drop(cmd_tx);
    handle.await.unwrap();
}

/// C2 (shutdown): closing the command channel (all `cmd_tx` handles dropped) causes
/// `run()` to exit cleanly — it must return within a bounded time without panicking or
/// hanging.
#[tokio::test]
async fn c2_channel_close_exits_loop_cleanly() {
    let (event_tx, _) = broadcast::channel::<Event>(16);
    let (cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let mut orch = Orchestrator::new(cmd_rx, event_tx);

    let handle = tokio::spawn(async move {
        orch.run().await;
    });

    // Dropping the only sender closes the mpsc channel; `run()` must return.
    drop(cmd_tx);

    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("run() did not exit within 1 s after the command channel was closed")
        .unwrap();
}

// ── C3: full scripted sequence + shutdown ────────────────────────────────────────────

/// C3: a scripted sequence of events drives the orchestrator through every node of the
/// canonical happy path in order, with `state()` asserted at each step; the loop then
/// exits cleanly when the command channel is closed.
///
/// Path: Idle → Listening → Transcribing → Thinking → Speaking → Idle
#[tokio::test]
async fn c3_scripted_sequence_advances_through_expected_states() {
    let (event_tx, _) = broadcast::channel::<Event>(16);
    let (cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let mut orch = Orchestrator::new(cmd_rx, event_tx);
    let mut state_rx = orch.subscribe_state();

    let handle = tokio::spawn(async move {
        orch.run().await;
    });

    // ── Idle → Listening ──────────────────────────────────────────────────────────────
    cmd_tx.send(Event::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("WakeDetected must produce a state change");
    assert_eq!(
        *state_rx.borrow(),
        State::Listening,
        "expected State::Listening after WakeDetected"
    );

    // ── Listening → Transcribing ──────────────────────────────────────────────────────
    cmd_tx.send(Event::SpeechEnded).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("SpeechEnded must produce a state change");
    assert_eq!(
        *state_rx.borrow(),
        State::Transcribing,
        "expected State::Transcribing after SpeechEnded"
    );

    // ── Transcribing → Thinking ───────────────────────────────────────────────────────
    cmd_tx
        .send(Event::TranscriptReady(Transcript {
            text: "hello zira".into(),
        }))
        .await
        .unwrap();
    state_rx
        .changed()
        .await
        .expect("TranscriptReady must produce a state change");
    assert_eq!(
        *state_rx.borrow(),
        State::Thinking,
        "expected State::Thinking after TranscriptReady"
    );

    // ── Thinking → Speaking ───────────────────────────────────────────────────────────
    cmd_tx.send(Event::SpeakRequest).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("SpeakRequest must produce a state change");
    assert_eq!(
        *state_rx.borrow(),
        State::Speaking,
        "expected State::Speaking after SpeakRequest"
    );

    // ── Speaking → Idle ───────────────────────────────────────────────────────────────
    cmd_tx
        .send(Event::TurnComplete(Usage {
            input_tokens: 5,
            output_tokens: 10,
        }))
        .await
        .unwrap();
    state_rx
        .changed()
        .await
        .expect("TurnComplete must produce a state change");
    assert_eq!(
        *state_rx.borrow(),
        State::Idle,
        "expected State::Idle after TurnComplete"
    );

    // ── Clean exit via channel close ──────────────────────────────────────────────────
    drop(cmd_tx);
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("run() did not exit within 1 s after the command channel was closed")
        .unwrap();
}
