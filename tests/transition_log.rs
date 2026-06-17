//! Frozen tests for T-00.18 — "Log the transitions".
//!
//! Criterion → test mapping:
//!
//!   C1 -> test_valid_transition_emits_one_log_record,
//!          test_valid_transition_log_has_correct_from_to
//!   C2 -> test_noop_transition_emits_no_log_record
//!   C3 -> test_one_valid_one_noop_emits_exactly_one_record

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tracing::field::{Field, Visit};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Layer;
use zira_core::Orchestrator;
use zira_proto::{Event as ZiraEvent, State};

// ── Capturing subscriber infrastructure ──────────────────────────────────────

/// One captured tracing record that looked like a state-transition event
/// (contained at least a "from" or "to" field).
#[derive(Debug, Clone, Default)]
struct TransitionRecord {
    from: Option<String>,
    to: Option<String>,
    /// The event discriminant / trigger field, if present.
    trigger: Option<String>,
}

struct FieldVisitor {
    record: TransitionRecord,
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let s = format!("{value:?}");
        match field.name() {
            "from" => self.record.from = Some(s),
            "to" => self.record.to = Some(s),
            "trigger" | "event" | "event_kind" => self.record.trigger = Some(s),
            _ => {}
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "from" => self.record.from = Some(value.to_string()),
            "to" => self.record.to = Some(value.to_string()),
            "trigger" | "event" | "event_kind" => {
                self.record.trigger = Some(value.to_string());
            }
            _ => {}
        }
    }
}

struct CapturingLayer {
    records: Arc<Mutex<Vec<TransitionRecord>>>,
}

impl<S: tracing::Subscriber> Layer<S> for CapturingLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor {
            record: TransitionRecord::default(),
        };
        event.record(&mut visitor);
        // Only keep records that carry a "from" or "to" field — these are
        // the structured state-transition log lines.
        if visitor.record.from.is_some() || visitor.record.to.is_some() {
            self.records.lock().unwrap().push(visitor.record);
        }
    }
}

/// Build a capturing subscriber and return it together with the shared record
/// store.  Install as the thread-local default via
/// `tracing::subscriber::set_default`.
fn make_capturing() -> (
    impl tracing::Subscriber + Send + Sync + 'static,
    Arc<Mutex<Vec<TransitionRecord>>>,
) {
    let records: Arc<Mutex<Vec<TransitionRecord>>> = Arc::new(Mutex::new(Vec::new()));
    let layer = CapturingLayer {
        records: records.clone(),
    };
    let subscriber = tracing_subscriber::registry().with(layer);
    (subscriber, records)
}

/// Convenience: build a fresh `Orchestrator` plus the command sender.
fn new_orchestrator() -> (Orchestrator, mpsc::Sender<ZiraEvent>) {
    let (event_tx, _) = broadcast::channel::<ZiraEvent>(16);
    let (cmd_tx, cmd_rx) = mpsc::channel::<ZiraEvent>(16);
    let orch = Orchestrator::new(cmd_rx, event_tx);
    (orch, cmd_tx)
}

// ── C1: valid transition emits a log record ───────────────────────────────────

/// C1: a single valid transition (WakeDetected: Idle → Listening) must produce
/// exactly one tracing record.
///
/// RED: the orchestrator currently has no tracing calls, so `records.len()` is
/// 0, not 1 — this test fails.
#[tokio::test(flavor = "current_thread")]
async fn test_valid_transition_emits_one_log_record() {
    let (subscriber, records) = make_capturing();
    let _guard = tracing::subscriber::set_default(subscriber);

    let (mut orch, cmd_tx) = new_orchestrator();
    let mut state_rx = orch.subscribe_state();
    let handle = tokio::spawn(async move { orch.run().await });

    cmd_tx.send(ZiraEvent::WakeDetected).await.unwrap();
    // Wait until the orchestrator has applied the transition (it sends on
    // state_tx before returning to recv, so this is a happens-before
    // guarantee: if changed() fires, the tracing call already happened).
    state_rx
        .changed()
        .await
        .expect("WakeDetected must advance the state");

    drop(cmd_tx);
    handle.await.unwrap();

    let captured = records.lock().unwrap();
    assert_eq!(
        captured.len(),
        1,
        "expected exactly 1 tracing record for WakeDetected (Idle→Listening); got {}",
        captured.len(),
    );
}

/// C1 (field values): the tracing record must carry `from = "Idle"` and
/// `to = "Listening"`.
///
/// RED: no record is emitted, so the first assertion panics.
#[tokio::test(flavor = "current_thread")]
async fn test_valid_transition_log_has_correct_from_to() {
    let (subscriber, records) = make_capturing();
    let _guard = tracing::subscriber::set_default(subscriber);

    let (mut orch, cmd_tx) = new_orchestrator();
    let mut state_rx = orch.subscribe_state();
    let handle = tokio::spawn(async move { orch.run().await });

    cmd_tx.send(ZiraEvent::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("WakeDetected must advance the state");

    drop(cmd_tx);
    handle.await.unwrap();

    let captured = records.lock().unwrap();
    assert!(
        !captured.is_empty(),
        "no tracing record was emitted for a valid transition",
    );
    let record = &captured[0];
    // State::Idle Debug-formats as "Idle"; State::Listening as "Listening".
    assert_eq!(record.from.as_deref(), Some("Idle"), "from field must be Idle");
    assert_eq!(
        record.to.as_deref(),
        Some("Listening"),
        "to field must be Listening",
    );
}

// ── C2: no-op transition emits no log record ──────────────────────────────────

/// C2: SpeechEnded from Idle has no entry in the transition table; the
/// orchestrator must not emit any tracing record for it.
///
/// RED: no tracing calls exist, so 0 records are captured — this trivially
/// passes, but a mutation that adds unconditional logging would cause it to
/// fail in the GREEN phase.
#[tokio::test(flavor = "current_thread")]
async fn test_noop_transition_emits_no_log_record() {
    let (subscriber, records) = make_capturing();
    let _guard = tracing::subscriber::set_default(subscriber);

    let (mut orch, cmd_tx) = new_orchestrator();
    let mut state_rx = orch.subscribe_state();
    let handle = tokio::spawn(async move { orch.run().await });

    // SpeechEnded from Idle is not in the transition table → silent no-op.
    cmd_tx.send(ZiraEvent::SpeechEnded).await.unwrap();
    let no_change =
        tokio::time::timeout(Duration::from_millis(100), state_rx.changed()).await;
    assert!(
        no_change.is_err(),
        "undefined transition must not publish a state change",
    );

    drop(cmd_tx);
    handle.await.unwrap();

    let captured = records.lock().unwrap();
    assert_eq!(
        captured.len(),
        0,
        "no-op transition must emit zero tracing records; got {}",
        captured.len(),
    );
}

// ── C3: one valid + one no-op → exactly one record ───────────────────────────

/// C3: drives one valid transition (WakeDetected: Idle→Listening) followed by
/// one no-op (WakeDetected again, now from Listening — undefined in the table),
/// then asserts exactly one state-change tracing record with the correct
/// `from = "Idle"` and `to = "Listening"`.
///
/// RED: no tracing calls exist, so `records.len()` is 0 — the length assertion
/// fails.
#[tokio::test(flavor = "current_thread")]
async fn test_one_valid_one_noop_emits_exactly_one_record() {
    let (subscriber, records) = make_capturing();
    let _guard = tracing::subscriber::set_default(subscriber);

    let (mut orch, cmd_tx) = new_orchestrator();
    let mut state_rx = orch.subscribe_state();
    let handle = tokio::spawn(async move { orch.run().await });

    // 1. Valid: Idle + WakeDetected → Listening.
    cmd_tx.send(ZiraEvent::WakeDetected).await.unwrap();
    state_rx
        .changed()
        .await
        .expect("WakeDetected must produce a state change");
    assert_eq!(
        *state_rx.borrow(),
        State::Listening,
        "state must be Listening after WakeDetected",
    );

    // 2. No-op: Listening + WakeDetected → None (not in the table).
    cmd_tx.send(ZiraEvent::WakeDetected).await.unwrap();
    let no_change =
        tokio::time::timeout(Duration::from_millis(100), state_rx.changed()).await;
    assert!(
        no_change.is_err(),
        "WakeDetected from Listening must be a no-op and must not change state",
    );

    drop(cmd_tx);
    handle.await.unwrap();

    let captured = records.lock().unwrap();
    assert_eq!(
        captured.len(),
        1,
        "expected exactly one state-change tracing record; got {}",
        captured.len(),
    );
    let record = &captured[0];
    assert_eq!(
        record.from.as_deref(),
        Some("Idle"),
        "from field must be Idle",
    );
    assert_eq!(
        record.to.as_deref(),
        Some("Listening"),
        "to field must be Listening",
    );
}
