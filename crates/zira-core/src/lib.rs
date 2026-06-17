//! zira-core — conversation state machine.

use tokio::sync::{broadcast, mpsc, watch};
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
pub fn next_state(current: State, event: &Event) -> Option<State> {
    match (current, event) {
        (State::Idle,       Event::WakeDetected)     => Some(State::Listening),
        (State::Listening,  Event::SpeechEnded)      => Some(State::Transcribing),
        (State::Transcribing, Event::TranscriptReady(_)) => Some(State::Thinking),
        (State::Thinking,   Event::SpeakRequest)     => Some(State::Speaking),
        (State::Thinking,   Event::PlanReady)        => Some(State::PlanReview),
        (State::Thinking,   Event::BargeIn)          => Some(State::Listening),
        (State::PlanReview, Event::TurnStarted)      => Some(State::Thinking),
        (State::PlanReview, Event::Error(_))         => Some(State::Idle),
        (State::Speaking,   Event::TurnComplete(_))  => Some(State::Idle),
        (State::Speaking,   Event::BargeIn)          => Some(State::Listening),
        _                                            => None,
    }
}

/// The runtime owner of conversation state.
///
/// Holds the current [`State`] (initially [`State::Idle`]) and the channel handles for
/// the command bus (mpsc receiver) and the event bus (broadcast sender).
pub struct Orchestrator {
    state: State,
    cmd_rx: mpsc::Receiver<Event>,
    event_tx: broadcast::Sender<Event>,
    state_tx: watch::Sender<State>,
}

impl Orchestrator {
    /// Build a new `Orchestrator` in [`State::Idle`].
    pub fn new(cmd_rx: mpsc::Receiver<Event>, event_tx: broadcast::Sender<Event>) -> Self {
        let (state_tx, _) = watch::channel(State::Idle);
        Self {
            state: State::Idle,
            cmd_rx,
            event_tx,
            state_tx,
        }
    }

    /// Return the current conversation state (read-only).
    pub fn state(&self) -> State {
        self.state
    }

    /// Return a receiver that is notified whenever the orchestrator's state changes.
    ///
    /// The initial value is [`State::Idle`]; only sends after subscription are visible.
    pub fn subscribe_state(&self) -> watch::Receiver<State> {
        self.state_tx.subscribe()
    }

    /// Drive the orchestrator's select-loop until the command channel is closed.
    ///
    /// On each iteration, one [`Event`] is consumed from the command bus.  If
    /// [`next_state`] returns `Some(s)` for the current `(state, event)` pair, the
    /// held state is updated to `s` and all [`subscribe_state`] receivers are notified;
    /// otherwise the event is silently ignored and the loop continues.  The loop exits
    /// cleanly when all [`mpsc::Sender`] handles for the command channel are dropped.
    pub async fn run(&mut self) {
        while let Some(event) = self.cmd_rx.recv().await {
            if let Some(new_state) = next_state(self.state, &event) {
                let from = self.state;
                self.state = new_state;
                let _ = self.state_tx.send(new_state);
                tracing::info!(from = ?from, to = ?new_state, trigger = ?event);
            }
        }
    }
}
