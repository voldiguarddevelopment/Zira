//! Frozen tests for T-01.12 — "Implement the claude brain".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_claude_brain_implements_brain,
//!          c1_respond_calls_bridge_ask
//!   C2 -> c2_respond_emits_segments_then_turn_complete,
//!          c2_exactly_one_turn_complete_terminates_turn,
//!          c2_turn_complete_carries_bridge_usage

use std::fs;
use std::os::unix::fs::PermissionsExt;

use zira_bridge::ClaudeBrain;
use zira_config::ZiraConfig;
use zira_core::Brain;
use zira_proto::{Emotion, Event, Transcript};

// ---- helpers ---------------------------------------------------------------

fn make_stub(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join("zira_claude_brain_tests");
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

// ---- C1 — ClaudeBrain implements Brain; respond() calls zira_bridge::ask ----

/// Compile-time and runtime check: `ClaudeBrain` satisfies the `Brain` bound and
/// `respond()` is callable, returning a non-empty `Vec<Event>` when the bridge stub
/// succeeds.
#[tokio::test]
async fn c1_claude_brain_implements_brain() {
    let stub = make_stub(
        "brain_impl.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"hello","num_turns":1,"usage":{"input_tokens":5,"output_tokens":3}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "hi".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    // Calling respond() through the Brain trait proves the impl at compile time
    // and exercises the method at runtime.
    let events: Vec<Event> = brain.respond().await;

    assert!(
        !events.is_empty(),
        "respond() must return at least one event when the bridge stub succeeds"
    );
}

/// `respond()` must internally invoke `zira_bridge::ask`: a valid stub must cause
/// `respond()` to return a `Vec<Event>` that contains `Event::TurnComplete`, proving
/// the bridge answer was processed.
#[tokio::test]
async fn c1_respond_calls_bridge_ask() {
    let stub = make_stub(
        "brain_calls_ask.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"Answer text","num_turns":1,"usage":{"input_tokens":10,"output_tokens":5}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "what is 2+2?".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    assert!(
        !events.is_empty(),
        "respond() must return at least one event when the bridge succeeds"
    );
    let has_turn_complete = events.iter().any(|e| matches!(e, Event::TurnComplete(_)));
    assert!(
        has_turn_complete,
        "respond() must emit TurnComplete, proving zira_bridge::ask was called and its \
         result was processed; got: {events:?}"
    );
}

// ---- C2 — success: EmotionSegment(s) in source order + exactly one TurnComplete ----

/// On a successful bridge call the answer text must be run through
/// `zira_emotion::segment` and each span emitted as `Event::EmotionSegment` in source
/// order, followed by exactly one `Event::TurnComplete` carrying the bridge usage.
#[tokio::test]
async fn c2_respond_emits_segments_then_turn_complete() {
    // Answer with two tagged spans.
    let stub = make_stub(
        "brain_segments.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"[emotion:Happy]Great![emotion:Calm]No problem.","num_turns":1,"usage":{"input_tokens":8,"output_tokens":4}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "how are you?".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    // Expect: 2 EmotionSegment events + 1 TurnComplete = 3 events.
    let n = events.len();
    assert!(
        n >= 3,
        "expected at least 2 EmotionSegments + 1 TurnComplete; got {n} events: {events:?}"
    );

    // Every event before the last must be EmotionSegment.
    for (i, ev) in events[..n - 1].iter().enumerate() {
        assert!(
            matches!(ev, Event::EmotionSegment(_)),
            "event[{i}] must be EmotionSegment; got {ev:?}"
        );
    }

    // The two segments must appear in source order with the correct emotion and text.
    let expected: &[(Emotion, &str)] = &[
        (Emotion::Happy, "Great!"),
        (Emotion::Calm, "No problem."),
    ];
    assert_eq!(
        events[..n - 1].len(),
        expected.len(),
        "number of EmotionSegment events must equal the number of tagged spans"
    );
    for (i, (ev, (exp_emotion, exp_text))) in
        events[..n - 1].iter().zip(expected.iter()).enumerate()
    {
        let Event::EmotionSegment(seg) = ev else {
            unreachable!("guarded by the loop above")
        };
        assert_eq!(
            seg.emotion, *exp_emotion,
            "event[{i}] emotion: expected {exp_emotion:?}, got {:?}",
            seg.emotion
        );
        assert_eq!(
            seg.text, *exp_text,
            "event[{i}] text: expected {exp_text:?}, got {:?}",
            seg.text
        );
    }

    // The final event must be TurnComplete.
    assert!(
        matches!(events[n - 1], Event::TurnComplete(_)),
        "last event must be TurnComplete; got {:?}",
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

/// The invariant holds for a plain (untagged) answer too: exactly one neutral
/// `EmotionSegment` followed by exactly one `TurnComplete`.
#[tokio::test]
async fn c2_exactly_one_turn_complete_terminates_turn() {
    let stub = make_stub(
        "brain_one_tc.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"plain answer","num_turns":1,"usage":{"input_tokens":3,"output_tokens":2}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "tell me something".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    let tc_count = events
        .iter()
        .filter(|e| matches!(e, Event::TurnComplete(_)))
        .count();
    assert_eq!(
        tc_count, 1,
        "exactly one TurnComplete must be emitted for a successful turn; got {tc_count}"
    );
    assert!(
        matches!(events.last(), Some(Event::TurnComplete(_))),
        "TurnComplete must be the last event in the sequence; got: {events:?}"
    );
}

/// The `Usage` in `TurnComplete` must carry the token counts reported by the bridge,
/// proving the usage flows end-to-end from `zira_bridge::ask` through `respond()`.
#[tokio::test]
async fn c2_turn_complete_carries_bridge_usage() {
    let stub = make_stub(
        "brain_usage.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"response text","num_turns":1,"usage":{"input_tokens":42,"output_tokens":17}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript { text: "usage check".to_string() };
    let mut brain = ClaudeBrain::new(cfg, "be helpful", transcript);

    let events = brain.respond().await;

    let usage = events.iter().find_map(|e| {
        if let Event::TurnComplete(u) = e {
            Some(u.clone())
        } else {
            None
        }
    });
    let usage = usage.expect("TurnComplete with Usage must be present in respond() output");
    assert_eq!(
        usage.input_tokens, 42,
        "TurnComplete must carry input_tokens from the bridge answer; got {}",
        usage.input_tokens
    );
    assert_eq!(
        usage.output_tokens, 17,
        "TurnComplete must carry output_tokens from the bridge answer; got {}",
        usage.output_tokens
    );
}
