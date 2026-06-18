//! Frozen tests for T-01.14 — "Test the thinking spine".
//!
//! Acceptance criteria for the gateable half of Phase 1:
//! transcript → ClaudeBrain::respond() → emotion-segmented events → TurnComplete.
//!
//! Criterion coverage:
//!
//!   C1 -> c1_thinking_spine_emits_segments_then_turn_complete
//!   C2 -> c2_multiple_emotion_spans_produce_segments_in_source_order,
//!          c2_bridge_failure_produces_single_error_event

use std::fs;
use std::os::unix::fs::PermissionsExt;

use zira_bridge::ClaudeBrain;
use zira_config::ZiraConfig;
use zira_core::Brain;
use zira_proto::{Emotion, Event, Segment, Transcript};

// ---- helpers ---------------------------------------------------------------

fn make_stub(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join("zira_thinking_spine_tests");
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

fn emit_segments(events: &[Event]) -> Vec<&Segment> {
    events
        .iter()
        .filter_map(|e| {
            if let Event::EmotionSegment(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .collect()
}

// ---- C1 — thinking spine: transcript in → EmotionSegment(s) then TurnComplete ----

/// Full thinking-spine acceptance test: a transcript fed into `ClaudeBrain::respond()`
/// against a stub `claude` script must yield a sequence where every event before the
/// last is an `EmotionSegment` and the last event is `TurnComplete`.
///
/// This is the primary gate for T-01.14: it verifies the three components of the spine
/// (bridge `ask`, `zira_emotion::segment`, event emission) are wired correctly in order.
#[tokio::test]
async fn c1_thinking_spine_emits_segments_then_turn_complete() {
    // Stub emits a tagged reply so that `zira_emotion::segment` produces two spans.
    let stub = make_stub(
        "spine_c1.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"[emotion:Happy]Hello there![emotion:Calm]How can I help?","num_turns":1,"usage":{"input_tokens":6,"output_tokens":7}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "greet me".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    // There must be at least one event.
    assert!(
        !events.is_empty(),
        "thinking spine must yield at least one event; got none"
    );

    let n = events.len();

    // Every event before the last must be EmotionSegment.
    for (i, ev) in events[..n - 1].iter().enumerate() {
        assert!(
            matches!(ev, Event::EmotionSegment(_)),
            "event[{i}] before TurnComplete must be EmotionSegment; got {ev:?}"
        );
    }

    // The final event must be TurnComplete.
    assert!(
        matches!(events[n - 1], Event::TurnComplete(_)),
        "last event emitted by thinking spine must be TurnComplete; got {:?}",
        events[n - 1]
    );

    // Exactly one TurnComplete in the whole sequence.
    let tc_count = events
        .iter()
        .filter(|e| matches!(e, Event::TurnComplete(_)))
        .count();
    assert_eq!(
        tc_count, 1,
        "exactly one TurnComplete must terminate the turn; got {tc_count}"
    );
}

// ---- C2 — multiple spans: one EmotionSegment per span in source order ---------

/// When the stub reply contains multiple `[emotion:NAME]` spans, `respond()` must emit
/// exactly one `Event::EmotionSegment` per span, in the same order they appear in the
/// text, before the final `TurnComplete`.
#[tokio::test]
async fn c2_multiple_emotion_spans_produce_segments_in_source_order() {
    // Three distinct emotion spans in a specific order.
    let stub = make_stub(
        "spine_c2_multi.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"[emotion:Excited]Wow![emotion:Curious]Tell me more.[emotion:Calm]I see.","num_turns":1,"usage":{"input_tokens":9,"output_tokens":6}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "say something interesting".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    let segments = emit_segments(&events);

    // Three spans → three EmotionSegment events.
    assert_eq!(
        segments.len(),
        3,
        "three emotion spans must produce exactly three EmotionSegment events; got {} \
         (full event list: {events:?})",
        segments.len()
    );

    // Verify source order: Excited → Curious → Calm.
    let expected: &[(Emotion, &str)] = &[
        (Emotion::Excited, "Wow!"),
        (Emotion::Curious, "Tell me more."),
        (Emotion::Calm, "I see."),
    ];
    for (i, (seg, (exp_emotion, exp_text))) in segments.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            seg.emotion, *exp_emotion,
            "EmotionSegment[{i}] emotion: expected {exp_emotion:?}, got {:?}",
            seg.emotion
        );
        assert_eq!(
            seg.text, *exp_text,
            "EmotionSegment[{i}] text: expected {exp_text:?}, got {:?}",
            seg.text
        );
    }

    // Segments appear before TurnComplete (i.e., TurnComplete is last).
    assert!(
        matches!(events.last(), Some(Event::TurnComplete(_))),
        "TurnComplete must be the final event after all EmotionSegments; got: {events:?}"
    );
}

// ---- C2 — failure path: bridge failure → single Event::Error ------------------

/// When the stub `claude` process exits with a non-zero status (simulating any bridge
/// failure), `respond()` must return exactly one `Event::Error` and nothing else —
/// no `EmotionSegment`, no `TurnComplete`.
#[tokio::test]
async fn c2_bridge_failure_produces_single_error_event() {
    // Stub exits 1 — any BridgeError triggers the same behaviour.
    let stub = make_stub("spine_c2_fail.sh", "exit 1");
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "this will fail".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    assert_eq!(
        events.len(),
        1,
        "a bridge failure must produce exactly one event; got {} events: {events:?}",
        events.len()
    );

    assert!(
        matches!(events[0], Event::Error(_)),
        "the single event on bridge failure must be Event::Error; got {:?}",
        events[0]
    );

    // No TurnComplete must leak alongside the Error.
    let has_tc = events.iter().any(|e| matches!(e, Event::TurnComplete(_)));
    assert!(
        !has_tc,
        "respond() must not emit TurnComplete when the bridge fails; events: {events:?}"
    );

    // No EmotionSegment must leak alongside the Error.
    let has_seg = events.iter().any(|e| matches!(e, Event::EmotionSegment(_)));
    assert!(
        !has_seg,
        "respond() must not emit EmotionSegment when the bridge fails; events: {events:?}"
    );
}
