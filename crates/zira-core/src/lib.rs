//! zira-core — conversation state machine.

use tokio::sync::{broadcast, mpsc};
use zira_proto::{Event, State};

/// The runtime owner of conversation state.
///
/// Holds the current [`State`] (initially [`State::Idle`]) and the channel handles for
/// the command bus (mpsc receiver) and the event bus (broadcast sender).
/// Transition logic and the run-loop are added in later tasks (T-00.16, T-00.17).
pub struct Orchestrator {
    state: State,
    cmd_rx: mpsc::Receiver<Event>,
    event_tx: broadcast::Sender<Event>,
}

impl Orchestrator {
    /// Build a new `Orchestrator` in [`State::Idle`].
    pub fn new(cmd_rx: mpsc::Receiver<Event>, event_tx: broadcast::Sender<Event>) -> Self {
        todo!("T-00.14 GREEN: implement Orchestrator::new")
    }

    /// Return the current conversation state (read-only).
    pub fn state(&self) -> State {
        todo!("T-00.14 GREEN: implement Orchestrator::state")
    }
}
