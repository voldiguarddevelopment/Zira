//! Frozen tests for T-00.14 — "Define the Orchestrator".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_orchestrator_accepts_channel_handles
//!   C2 -> c2_new_builds_orchestrator_in_idle, c2_state_accessor_is_read_only
//!   C3 -> c3_initial_state_is_idle

use tokio::sync::{broadcast, mpsc};
use zira_core::Orchestrator;
use zira_proto::{Event, State};

// ---- C1 -------------------------------------------------------------------------------

/// C1: `Orchestrator` holds the current `State` and channel handles for the command bus
/// (mpsc::Receiver<Event>) and the event bus (broadcast::Sender<Event>).
///
/// Verified structurally: the compiler rejects this call if `new` does not accept those
/// handle types, proving the struct is wired to hold them.
#[test]
fn c1_orchestrator_accepts_channel_handles() {
    let (event_tx, _event_rx) = broadcast::channel::<Event>(16);
    let (_cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let _orch = Orchestrator::new(cmd_rx, event_tx);
}

// ---- C2 -------------------------------------------------------------------------------

/// C2: the constructor builds an `Orchestrator` in `Idle`; `state()` returns it.
#[test]
fn c2_new_builds_orchestrator_in_idle() {
    let (event_tx, _event_rx) = broadcast::channel::<Event>(16);
    let (_cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let orch = Orchestrator::new(cmd_rx, event_tx);
    assert_eq!(orch.state(), State::Idle, "fresh Orchestrator must start in Idle");
}

/// C2: `state()` is a read-only `&self` accessor — callable on a non-mut binding.
#[test]
fn c2_state_accessor_is_read_only() {
    let (event_tx, _event_rx) = broadcast::channel::<Event>(16);
    let (_cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let orch = Orchestrator::new(cmd_rx, event_tx); // non-mut binding
    let _s: State = orch.state(); // &self access — no mut needed
}

// ---- C3 -------------------------------------------------------------------------------

/// C3: constructing an `Orchestrator` and reading `state()` yields `State::Idle`.
#[test]
fn c3_initial_state_is_idle() {
    let (event_tx, _event_rx) = broadcast::channel::<Event>(16);
    let (_cmd_tx, cmd_rx) = mpsc::channel::<Event>(16);
    let orch = Orchestrator::new(cmd_rx, event_tx);
    assert_eq!(
        orch.state(),
        State::Idle,
        "a freshly constructed Orchestrator must report State::Idle"
    );
}
