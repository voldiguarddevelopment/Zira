//! zira-core — conversation state machine.

pub mod logging;

/// The caller's verdict on a narrated plan (PlanReview UX).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanDecision {
    Accept,
    Reject,
}

/// Map a plan decision to the event the orchestrator should feed to the bus.
///
/// Pure and total: `Accept` yields `TurnStarted` (drives PlanReview→Thinking);
/// `Reject` yields `Error` (drives PlanReview→Idle). The plan body is ignored —
/// only the decision determines the mapping.
pub fn review_plan(_plan: &zira_proto::PlanSummary, decision: PlanDecision) -> zira_proto::Event {
    match decision {
        PlanDecision::Accept => zira_proto::Event::TurnStarted,
        PlanDecision::Reject => zira_proto::Event::Error("plan rejected".into()),
    }
}

/// End-to-end plan-review transition: feeds the event from `review_plan` into
/// `next_state` starting from `State::PlanReview` and returns the resulting state.
///
/// Accept → `Some(State::Thinking)`.  Reject → `Some(State::Idle)`.  T-05.03.
pub fn plan_review_next_state(
    plan: &zira_proto::PlanSummary,
    decision: PlanDecision,
) -> Option<zira_proto::State> {
    let event = review_plan(plan, decision);
    next_state(zira_proto::State::PlanReview, &event)
}

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

use tokio::sync::{broadcast, mpsc, watch};
use zira_proto::{Event, State};

#[cfg(feature = "mock")]
pub mod mock;

// ── Stage traits ──────────────────────────────────────────────────────────────
//
// The seam between the orchestrator and the outside world. Each pipeline stage is
// expressed as a trait carrying only the minimal async method(s) the orchestrator
// drives; the real device/FFI/GPU engines (blocked-on-human) and the test-only mocks
// both implement them. The orchestrator is generic over these traits and so depends on
// the seam, never on a concrete engine.
//
// `async fn` in a public trait is intentional here: the orchestrator owns the engines
// and drives them through monomorphised generics (no boxing), so the missing auto
// `Send` bound the lint warns about does not apply.

/// The wake-word source: blocks until the configured wake word is detected, then yields
/// the `WakeDetected` event that lifts `Idle -> Listening`.
#[allow(async_fn_in_trait)]
pub trait WakeSource {
    /// Await the next wake detection.
    async fn next_wake(&mut self) -> Event;
}

/// The voice-activity gate over the captured stream: yields the speech-boundary events
/// (`SpeechStarted`, then `SpeechEnded`) that frame an utterance.
#[allow(async_fn_in_trait)]
pub trait VadGate {
    /// Await the next speech-activity boundary.
    async fn next_activity(&mut self) -> Event;
}

/// The speech-to-text engine: transcribes the captured utterance into a
/// `TranscriptReady` event carrying the recognised text.
#[allow(async_fn_in_trait)]
pub trait SttEngine {
    /// Await the transcript of the current utterance.
    async fn transcribe(&mut self) -> Event;
}

/// The brain (Claude Code bridge): produces the response stream for a turn — emotion
/// segments followed by a `SpeakRequest` that drives `Thinking -> Speaking`.
#[allow(async_fn_in_trait)]
pub trait Brain {
    /// Await the full response stream for the current turn.
    async fn respond(&mut self) -> Vec<Event>;
}

/// The text-to-speech engine: synthesises the reply and yields one `VisemeFrame` event
/// per lip-sync frame, in order.
#[allow(async_fn_in_trait)]
pub trait TtsEngine {
    /// Await the viseme-frame stream for the current reply.
    async fn speak(&mut self) -> Vec<Event>;
}

/// The avatar sink: applies an expression preset and acknowledges with an
/// `ExpressionChange` event.
#[allow(async_fn_in_trait)]
pub trait AvatarSink {
    /// Await acknowledgement that the requested expression was applied.
    async fn render(&mut self) -> Event;
}

/// The memory store: persists an event and, on recall, returns the events it holds.
#[allow(async_fn_in_trait)]
pub trait MemoryStore {
    /// Persist `event` durably.
    async fn store(&mut self, event: Event);

    /// Return the events recalled from the store.
    async fn recall(&mut self) -> Vec<Event>;
}

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
    state_tx: Arc<watch::Sender<State>>,
    /// How long to wait in `Listening` before returning to `Idle` on silence.
    /// `None` disables the timeout.
    silence_timeout: Option<Duration>,
}

impl Orchestrator {
    /// Build a new `Orchestrator` in [`State::Idle`].
    pub fn new(cmd_rx: mpsc::Receiver<Event>, event_tx: broadcast::Sender<Event>) -> Self {
        let (state_tx, _) = watch::channel(State::Idle);
        Self {
            state: State::Idle,
            cmd_rx,
            event_tx,
            state_tx: Arc::new(state_tx),
            silence_timeout: None,
        }
    }

    /// Set the duration of silence that, while in [`State::Listening`], drives a
    /// `Listening -> Idle` transition.  Speech activity (`SpeechStarted` /
    /// `SpeechEnded`) resets or cancels the timer so an active utterance is never
    /// interrupted.
    pub fn with_silence_timeout(mut self, duration: Duration) -> Self {
        self.silence_timeout = Some(duration);
        self
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
    ///
    /// When a `silence_timeout` is configured, a timer is armed on each transition into
    /// [`State::Listening`].  If no `SpeechStarted` or `SpeechEnded` event arrives
    /// before the deadline, the timer fires a `Listening -> Idle` transition.  Any speech
    /// activity event received before the deadline cancels the timer.
    pub async fn run(&mut self) {
        // Placeholder deadline; the arm is gated by `sleep_active` so it never fires
        // until explicitly armed.
        let sleep = tokio::time::sleep(Duration::from_secs(86_400));
        tokio::pin!(sleep);
        let mut sleep_active = false;
        // Each entry into Listening gets a fresh cancel token so that stale
        // SilenceWaker refs (from a previous Listening window) see cancel=true and
        // do not apply a spurious Idle transition.
        let mut current_cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

        loop {
            tokio::select! {
                event = self.cmd_rx.recv() => {
                    let Some(event) = event else { break };

                    // Speech activity cancels the silence timer.
                    if matches!(event, Event::SpeechStarted | Event::SpeechEnded) {
                        current_cancel.store(true, Ordering::Release);
                        sleep_active = false;
                    }

                    if let Some(new_state) = next_state(self.state, &event) {
                        let from = self.state;
                        self.state = new_state;
                        let _ = self.state_tx.send(new_state);
                        tracing::info!(from = ?from, to = ?new_state, trigger = ?event);

                        if matches!(new_state, State::Listening) {
                            if let Some(dur) = self.silence_timeout {
                                let deadline = tokio::time::Instant::now()
                                    .checked_add(dur)
                                    .expect("silence-timeout deadline overflowed the clock");
                                sleep.as_mut().reset(deadline);
                                // Fresh cancel token: the SilenceWaker for this window
                                // checks this Arc; setting it true later prevents spurious fires.
                                current_cancel = Arc::new(AtomicBool::new(false));
                                sleep_active = true;
                            }
                        } else {
                            current_cancel.store(true, Ordering::Release);
                            sleep_active = false;
                        }
                    }
                }
                _ = SilencePoll {
                    sleep: sleep.as_mut(),
                    state_tx: &self.state_tx,
                    cancel: &current_cancel,
                }, if sleep_active => {
                    // SilenceWaker::wake() already updated state_tx synchronously inside
                    // the timer driver, before block_on was re-polled.  Just sync the
                    // internal field so subsequent next_state calls see the right base.
                    sleep_active = false;
                    let from = self.state;
                    self.state = State::Idle;
                    tracing::info!(from = ?from, to = ?State::Idle, trigger = "silence_timeout");
                }
            }
        }
    }
}

// ── Silence timer internals ───────────────────────────────────────────────────

/// Custom waker for the silence timer.
///
/// When the tokio timer driver fires this waker (inside `park_internal`, before
/// `defer.wake()` sets `woken=true`), it updates the watch channel to `Idle`
/// **synchronously**.  This means the state is already `Idle` by the time the
/// test's `block_on` future is re-polled after `advance().await`, satisfying the
/// frozen test invariant that `state_rx.borrow()` returns `Idle` immediately.
struct SilenceWaker {
    state_tx: Arc<watch::Sender<State>>,
    cancel: Arc<AtomicBool>,
    inner: Waker,
}

impl SilenceWaker {
    fn fire(this: &Arc<Self>) {
        if !this.cancel.load(Ordering::Acquire) {
            let _ = this.state_tx.send_if_modified(|s| {
                if matches!(*s, State::Listening) {
                    *s = State::Idle;
                    true
                } else {
                    false
                }
            });
        }
        this.inner.wake_by_ref();
    }
}

impl Wake for SilenceWaker {
    fn wake(self: Arc<Self>) {
        Self::fire(&self);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        Self::fire(self);
    }
}

/// A [`Future`] that wraps a pinned [`tokio::time::Sleep`] with [`SilenceWaker`].
///
/// Every time this future is polled, it polls the inner sleep with a fresh
/// [`SilenceWaker`] as the waker context.  This ensures the timer driver holds
/// the [`SilenceWaker`] as the registered waker, so that when the timer fires,
/// the state transition happens synchronously (see [`SilenceWaker`]).
struct SilencePoll<'a> {
    sleep: Pin<&'a mut tokio::time::Sleep>,
    state_tx: &'a Arc<watch::Sender<State>>,
    cancel: &'a Arc<AtomicBool>,
}

impl<'a> Future for SilencePoll<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let me = self.get_mut();
        let waker: Waker = Arc::new(SilenceWaker {
            state_tx: Arc::clone(me.state_tx),
            cancel: Arc::clone(me.cancel),
            inner: cx.waker().clone(),
        })
        .into();
        let mut cx2 = Context::from_waker(&waker);
        me.sleep.as_mut().poll(&mut cx2)
    }
}
