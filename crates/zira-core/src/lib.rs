//! zira-core — conversation state machine.

use tokio::sync::{broadcast, mpsc};
use zira_proto::{Event, State};

/// Handles returned by [`create_bus`].
pub struct BusHandles {
    /// Send a command to the orchestrator's single consumer.
    pub cmd_tx: mpsc::Sender<Event>,
    /// Receive commands — owned by the [`Orchestrator`].
    pub cmd_rx: mpsc::Receiver<Event>,
    /// Publish events to all subscribers; call `.subscribe()` for each receiver.
    pub event_tx: broadcast::Sender<Event>,
}

/// Construct the event bus: one mpsc command channel and one broadcast event channel,
/// both typed over [`Event`]. Returns all handles via [`BusHandles`].
pub fn create_bus() -> BusHandles {
    let (cmd_tx, cmd_rx) = mpsc::channel(64);
    let (event_tx, _) = broadcast::channel(64);
    BusHandles {
        cmd_tx,
        cmd_rx,
        event_tx,
    }
}

/// Return the next [`State`] for the given `(current, event)` pair, or `None` when no
/// transition is defined for that pair (a pure, side-effect-free function).
///
/// Implements the PLAN.md §5 table. T-00.16.
pub fn next_state(_current: State, _event: &Event) -> Option<State> {
    todo!("T-00.16 GREEN: implement the transition table")
}

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
        Self {
            state: State::Idle,
            cmd_rx,
            event_tx,
        }
    }

    /// Return the current conversation state (read-only).
    pub fn state(&self) -> State {
        self.state
    }
}
