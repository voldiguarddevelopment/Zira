//! Frozen tests for T-00.19 — "Add the silence timeout".
//!
//! All tests use `tokio::time::pause()` + `tokio::time::advance()` to control
//! the virtual clock deterministically — no real sleeping occurs.
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_silence_timeout_elapses_drives_listening_to_idle
//!   C2 -> c2_speech_started_cancels_silence_timeout
//!   C3 -> c3_full_scenario_silence_fires_and_activity_prevents

use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use zira_core::Orchestrator;
use zira_proto::{Event, State};

/// Helper: build an `Orchestrator` with a silence timeout and return the
/// command sender alongside.
fn make_orch_with_timeout(
    silence_ms: u64,
) -> (Orchestrator, mpsc::Sender<Event>) {
    let (event_tx, _) = broadcast::channel::<Event>(16);
    let (cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let orch = Orchestrator::new(cmd_rx, event_tx)
        .with_silence_timeout(Duration::from_millis(silence_ms));
    (orch, cmd_tx)
}

// ── C1: silence timeout fires Listening → Idle ────────────────────────────────

/// C1: after entering `Listening`, if no `SpeechStarted` or `SpeechEnded` event
/// arrives within the configured silence timeout, the orchestrator must transition
/// from `Listening` to `Idle`.
///
/// The virtual clock is advanced past the timeout duration with no speech events
/// injected.  The state must be `Idle` immediately after `advance()` returns.
///
/// RED: `run()` has no timer logic, so the state remains `Listening` after
/// `advance()` — the assertion fails.
#[tokio::test]
async fn c1_silence_timeout_elapses_drives_listening_to_idle() {
    tokio::time::pause();

    let (mut orch, cmd_tx) = make_orch_with_timeout(500);
    let mut state_rx = orch.subscribe_state();

    let handle = tokio::spawn(async move { orch.run().await });

    // Drive into Listening.
    cmd_tx.send(Event::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("WakeDetected must advance state to Listening");
    assert_eq!(
        *state_rx.borrow(),
        State::Listening,
        "prerequisite: state must be Listening after WakeDetected"
    );

    // Advance past the 500 ms silence timeout with no speech events.
    tokio::time::advance(Duration::from_millis(501)).await;

    // The silence timeout must have fired and driven Listening -> Idle.
    // RED: no timer logic exists, so state is still Listening — assertion fails.
    assert_eq!(
        *state_rx.borrow(),
        State::Idle,
        "silence timeout must drive Listening -> Idle after 500 ms with no speech activity"
    );

    drop(cmd_tx);
    handle.await.unwrap();
}

// ── C2: speech activity cancels the silence timer ─────────────────────────────

/// C2: when `SpeechStarted` arrives before the silence timeout fires, the timer
/// must be cancelled so the orchestrator does NOT transition to `Idle` during the
/// utterance.
///
/// Timeline: silence_timeout = 500 ms.  At virtual t = 200 ms, `SpeechStarted`
/// is injected (activity arrives).  The clock is then advanced to t = 600 ms —
/// past the original 500 ms deadline.  The state must still be `Listening`, not
/// `Idle`, because the activity cancelled the timer.
///
/// RED: no timer is implemented, so the state naturally stays `Listening` and
/// both assertions trivially pass.  This test becomes a mutation-killing guard in
/// the GREEN phase: any implementation that fires the timeout despite activity
/// will drive the state to `Idle`, breaking `assert_ne!`.
#[tokio::test]
async fn c2_speech_started_cancels_silence_timeout() {
    tokio::time::pause();

    let (mut orch, cmd_tx) = make_orch_with_timeout(500);
    let mut state_rx = orch.subscribe_state();

    let handle = tokio::spawn(async move { orch.run().await });

    // Enter Listening.
    cmd_tx.send(Event::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("WakeDetected must advance state to Listening");
    assert_eq!(*state_rx.borrow(), State::Listening);

    // At t = 200 ms: speech activity arrives before the 500 ms deadline.
    tokio::time::advance(Duration::from_millis(200)).await;
    cmd_tx.send(Event::SpeechStarted).await.unwrap();
    // Yield so the orchestrator task can process SpeechStarted and cancel its timer.
    tokio::task::yield_now().await;

    // Advance to t = 600 ms — past the original 500 ms deadline.
    // If the timer was not cancelled, we would now be in Idle.
    tokio::time::advance(Duration::from_millis(400)).await;

    assert_ne!(
        *state_rx.borrow(),
        State::Idle,
        "SpeechStarted must cancel the silence timer; state must not be Idle after the original deadline"
    );
    assert_eq!(
        *state_rx.borrow(),
        State::Listening,
        "state must remain Listening while speech is active"
    );

    drop(cmd_tx);
    handle.await.unwrap();
}

// ── C3: integration — fires on silence, silent on activity ────────────────────

/// C3: integration test with a paused clock covering both scenarios in sequence.
///
/// Scenario A — silence fires the timeout:
///   Drive to `Listening`, advance the clock past the timeout with no events,
///   assert `Idle`.
///
/// Scenario B — speech activity prevents the timeout:
///   Drive a fresh orchestrator to `Listening`, inject `SpeechStarted` before
///   the timeout, advance past the original deadline, assert still NOT `Idle`.
///
/// RED: Scenario A's assertion (`State::Idle`) fails because `run()` has no
/// timer logic — the state remains `Listening` after `advance()`.
#[tokio::test]
async fn c3_full_scenario_silence_fires_and_activity_prevents() {
    // ── Scenario A: silence → Idle ──────────────────────────────────────────
    {
        tokio::time::pause();

        let (mut orch, cmd_tx) = make_orch_with_timeout(300);
        let mut state_rx = orch.subscribe_state();
        let handle = tokio::spawn(async move { orch.run().await });

        cmd_tx.send(Event::WakeDetected).await.unwrap();
        state_rx
            .changed()
            .await
            .expect("WakeDetected must reach Listening");
        assert_eq!(*state_rx.borrow(), State::Listening);

        // No speech events — advance past the 300 ms timeout.
        tokio::time::advance(Duration::from_millis(301)).await;

        // RED: this assertion fails — state is still Listening.
        assert_eq!(
            *state_rx.borrow(),
            State::Idle,
            "Scenario A: silence timeout must fire Listening -> Idle"
        );

        drop(cmd_tx);
        handle.await.unwrap();

        tokio::time::resume();
    }

    // ── Scenario B: activity → no timeout ──────────────────────────────────
    {
        tokio::time::pause();

        let (mut orch, cmd_tx) = make_orch_with_timeout(300);
        let mut state_rx = orch.subscribe_state();
        let handle = tokio::spawn(async move { orch.run().await });

        cmd_tx.send(Event::WakeDetected).await.unwrap();
        state_rx
            .changed()
            .await
            .expect("WakeDetected must reach Listening");
        assert_eq!(*state_rx.borrow(), State::Listening);

        // Activity arrives at t = 100 ms — before the 300 ms deadline.
        tokio::time::advance(Duration::from_millis(100)).await;
        cmd_tx.send(Event::SpeechStarted).await.unwrap();
        tokio::task::yield_now().await;

        // Advance past the original 300 ms deadline.
        tokio::time::advance(Duration::from_millis(250)).await;

        assert_ne!(
            *state_rx.borrow(),
            State::Idle,
            "Scenario B: SpeechStarted must prevent the silence timeout from firing"
        );

        drop(cmd_tx);
        handle.await.unwrap();

        tokio::time::resume();
    }
}
