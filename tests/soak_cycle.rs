//! T-05.14 — soak proxy: hammer the mocked `Idle -> ... -> Idle` cycle thousands of
//! times and assert it always returns to Idle with no panic/deadlock/state wedge. This is
//! the gateable stress proxy; the multi-hour soak on target hardware with the real voice
//! stack stays device-bound.
//!
//!   C1 -> c1_soak_two_thousand_cycles_return_to_idle
//!   C2 -> c2_full_count_completes_and_ends_idle

use zira_core::mock::MockCycle;
use zira_core::{Brain, SttEngine, VadGate, WakeSource};
use zira_proto::{Event, State, Usage};

/// Drive one full mocked turn `Idle -> Listening -> Transcribing -> Thinking -> Speaking
/// -> Idle` and return the ordered state path (starting at the initial Idle).
async fn one_cycle() -> Vec<State> {
    let mut cycle = MockCycle::new();
    let cmd = cycle.cmd();
    let mut state_rx = cycle.subscribe_state();
    let mut path = vec![*state_rx.borrow()];

    let wake = cycle.wake.next_wake().await;
    cmd.send(wake).await.unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    cmd.send(cycle.vad.next_activity().await).await.unwrap(); // SpeechStarted (no transition)
    cmd.send(cycle.vad.next_activity().await).await.unwrap(); // SpeechEnded
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    cmd.send(cycle.stt.transcribe().await).await.unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    for event in cycle.brain.respond().await {
        cmd.send(event).await.unwrap();
    }
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    cmd.send(Event::TurnComplete(Usage {
        input_tokens: 1,
        output_tokens: 1,
    }))
    .await
    .unwrap();
    state_rx.changed().await.unwrap();
    path.push(*state_rx.borrow());

    path
}

const SOAK_CYCLES: usize = 2000;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c1_soak_two_thousand_cycles_return_to_idle() {
    for i in 0..SOAK_CYCLES {
        let path = one_cycle().await;
        assert_eq!(
            path.first(),
            Some(&State::Idle),
            "cycle {i} must start in Idle"
        );
        assert_eq!(
            path.last(),
            Some(&State::Idle),
            "cycle {i} must return to Idle (no wedge); path {path:?}"
        );
        assert_eq!(path.len(), 6, "cycle {i} must pass through all six states");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn c2_full_count_completes_and_ends_idle() {
    let mut completed = 0usize;
    let mut last = State::Idle;
    for _ in 0..SOAK_CYCLES {
        let path = one_cycle().await;
        last = *path.last().unwrap();
        completed += 1;
    }
    assert_eq!(completed, SOAK_CYCLES, "every requested cycle must complete");
    assert_eq!(last, State::Idle, "the soak must end back in Idle");
}
