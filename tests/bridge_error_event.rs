//! Frozen tests for T-01.13 — "Emit the bridge error event".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_spawn_failed_emits_single_error_event,
//!          c1_non_zero_exit_emits_single_error_event,
//!          c1_missing_result_emits_single_error_event,
//!          c1_error_carries_display_message

use std::fs;
use std::os::unix::fs::PermissionsExt;

use zira_bridge::{BridgeError, ClaudeBrain};
use zira_config::ZiraConfig;
use zira_core::Brain;
use zira_proto::Event;

// ---- helpers ---------------------------------------------------------------

fn make_stub(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join("zira_bridge_error_tests");
    fs::create_dir_all(&dir).expect("create temp dir for stub scripts");
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write stub script");
    let mut perms = fs::metadata(&path).expect("stat stub").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod stub");
    path.to_string_lossy().into_owned()
}

fn make_cfg(binary_path: &str) -> ZiraConfig {
    let mut cfg = ZiraConfig::default();
    cfg.model.binary_path = binary_path.to_string();
    cfg.model.model_id = "stub-model".to_string();
    cfg
}

fn assert_single_error(events: &[Event], expected_msg: &str) {
    assert_eq!(
        events.len(),
        1,
        "respond() must return exactly one event on bridge failure; got {} events: {:?}",
        events.len(),
        events
    );
    let Event::Error(ref msg) = events[0] else {
        panic!(
            "the single event must be Event::Error; got {:?}",
            events[0]
        );
    };
    assert_eq!(
        msg, expected_msg,
        "Event::Error must carry the bridge error's Display message"
    );
}

// ---- C1 — spawn failure: non-existent binary path -------------------------

/// When the binary path does not exist, `ask` returns `BridgeError::SpawnFailed`.
/// `respond()` must emit exactly one `Event::Error` whose string matches that
/// error's `Display` representation, and must not panic.
#[tokio::test]
async fn c1_spawn_failed_emits_single_error_event() {
    let cfg = make_cfg("/no/such/claude/binary");
    let transcript = zira_proto::Transcript { text: "hello".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    assert_eq!(
        events.len(),
        1,
        "respond() must return exactly one event when the binary cannot be spawned; \
         got {} events: {:?}",
        events.len(),
        events
    );
    assert!(
        matches!(events[0], Event::Error(_)),
        "the event must be Event::Error when spawn fails; got {:?}",
        events[0]
    );

    // The message must start with the SpawnFailed prefix defined in BridgeError.
    let Event::Error(ref msg) = events[0] else {
        unreachable!()
    };
    let prefix = "failed to spawn claude process:";
    assert!(
        msg.starts_with(prefix),
        "Event::Error message must start with {:?} (the SpawnFailed Display prefix); \
         got {:?}",
        prefix,
        msg
    );
}

// ---- C1 — non-zero exit: stub exits 1 -------------------------------------

/// When the stub exits with a non-zero status, `ask` returns
/// `BridgeError::NonZeroExit(1)`.  `respond()` must emit exactly one
/// `Event::Error` whose string equals `BridgeError::NonZeroExit(1).to_string()`.
#[tokio::test]
async fn c1_non_zero_exit_emits_single_error_event() {
    let stub = make_stub("err_nonzero.sh", "exit 1");
    let cfg = make_cfg(&stub);
    let transcript = zira_proto::Transcript { text: "hello".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    let expected = BridgeError::NonZeroExit(1).to_string();
    assert_single_error(&events, &expected);
}

// ---- C1 — missing result: stub exits 0 but emits no result line -----------

/// When the stub exits 0 but the output contains no terminal `result` event,
/// `ask` returns `BridgeError::MissingResult`.  `respond()` must emit exactly
/// one `Event::Error` whose string equals `BridgeError::MissingResult.to_string()`.
#[tokio::test]
async fn c1_missing_result_emits_single_error_event() {
    // Emit valid-looking JSON but no "result"-typed event.
    let stub = make_stub(
        "err_missing.sh",
        r#"printf '%s\n' '{"type":"assistant","message":"hi"}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = zira_proto::Transcript { text: "hello".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    let expected = BridgeError::MissingResult.to_string();
    assert_single_error(&events, &expected);
}

// ---- C1 — message round-trip: Display string is preserved exactly ---------

/// The `Event::Error(String)` payload must be the exact `Display` output of the
/// `BridgeError` returned by `ask` — no truncation, no prefix, no suffix.
/// Verified here for `NonZeroExit` whose message is fully deterministic.
#[tokio::test]
async fn c1_error_carries_display_message() {
    let stub = make_stub("err_display.sh", "exit 2");
    let cfg = make_cfg(&stub);
    let transcript = zira_proto::Transcript { text: "msg check".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    assert_eq!(events.len(), 1, "exactly one event on bridge error");

    let Event::Error(ref got) = events[0] else {
        panic!("expected Event::Error; got {:?}", events[0]);
    };

    let expected = BridgeError::NonZeroExit(2).to_string();
    assert_eq!(
        got, &expected,
        "Event::Error payload must equal the BridgeError's Display string exactly; \
         expected {:?}, got {:?}",
        expected, got
    );

    // No TurnComplete must appear alongside the Error.
    let has_tc = events.iter().any(|e| matches!(e, Event::TurnComplete(_)));
    assert!(
        !has_tc,
        "respond() must not emit TurnComplete when ask() fails; events: {events:?}"
    );
}
