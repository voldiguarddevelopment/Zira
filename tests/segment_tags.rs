//! Frozen tests for T-01.03 — "Segment the tagged reply".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_single_marker_splits_into_segments,
//!          c1_multiple_markers_emit_ordered_spans,
//!          c1_emotion_in_effect_for_each_span,
//!          c1_all_known_emotions_segmented,
//!          c1_concatenated_text_equals_stripped_reply
//!   C2 -> c2_leading_untagged_text_becomes_neutral_segment,
//!          c2_empty_input_returns_empty_vec,
//!          c2_only_tagged_text_no_leading_prose
//!   C3 -> c3_consecutive_markers_emit_no_empty_segment,
//!          c3_marker_at_end_emits_no_empty_segment,
//!          c3_multiple_consecutive_markers_all_dropped,
//!          c3_only_markers_returns_empty_vec

use zira_emotion::segment;
use zira_proto::Emotion;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_single_marker_splits_into_segments() {
    let result = segment("[emotion:Happy]Hello");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].emotion, Emotion::Happy);
    assert_eq!(result[0].text, "Hello");
}

#[test]
fn c1_multiple_markers_emit_ordered_spans() {
    // Leading text → Neutral, then each marker opens a new span.
    let result = segment("A[emotion:Happy]B[emotion:Sad]C");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].emotion, Emotion::Neutral);
    assert_eq!(result[0].text, "A");
    assert_eq!(result[1].emotion, Emotion::Happy);
    assert_eq!(result[1].text, "B");
    assert_eq!(result[2].emotion, Emotion::Sad);
    assert_eq!(result[2].text, "C");
}

#[test]
fn c1_emotion_in_effect_for_each_span() {
    // Each segment carries the emotion that was opened by the tag before its text.
    let result = segment("[emotion:Excited]Go![emotion:Calm]Relax.");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].emotion, Emotion::Excited);
    assert_eq!(result[0].text, "Go!");
    assert_eq!(result[1].emotion, Emotion::Calm);
    assert_eq!(result[1].text, "Relax.");
}

#[test]
fn c1_all_known_emotions_segmented() {
    let input = "[emotion:Neutral]n[emotion:Happy]h[emotion:Sad]s\
                 [emotion:Angry]a[emotion:Excited]e[emotion:Calm]c\
                 [emotion:Curious]u[emotion:Concerned]o[emotion:Playful]p\
                 [emotion:Tired]t";
    let result = segment(input);
    let expected = [
        (Emotion::Neutral, "n"),
        (Emotion::Happy, "h"),
        (Emotion::Sad, "s"),
        (Emotion::Angry, "a"),
        (Emotion::Excited, "e"),
        (Emotion::Calm, "c"),
        (Emotion::Curious, "u"),
        (Emotion::Concerned, "o"),
        (Emotion::Playful, "p"),
        (Emotion::Tired, "t"),
    ];
    assert_eq!(result.len(), expected.len());
    for (i, (emotion, text)) in expected.iter().enumerate() {
        assert_eq!(result[i].emotion, *emotion, "segment {i}");
        assert_eq!(result[i].text, *text, "segment {i}");
    }
}

#[test]
fn c1_concatenated_text_equals_stripped_reply() {
    // Invariant: joining all segment texts reconstructs the stripped reply.
    let input = "Hello [emotion:Happy]world! [emotion:Sad]Goodbye.";
    let result = segment(input);
    let joined: String = result.iter().map(|s| s.text.as_str()).collect();
    let stripped = zira_emotion::strip_tags(input);
    assert_eq!(joined, stripped);
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_leading_untagged_text_becomes_neutral_segment() {
    // Text before the first marker becomes a Segment with Emotion::Neutral.
    let result = segment("Hello [emotion:Happy]world");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].emotion, Emotion::Neutral);
    assert_eq!(result[0].text, "Hello ");
    assert_eq!(result[1].emotion, Emotion::Happy);
    assert_eq!(result[1].text, "world");
}

#[test]
fn c2_empty_input_returns_empty_vec() {
    let result = segment("");
    assert!(result.is_empty());
}

#[test]
fn c2_only_tagged_text_no_leading_prose() {
    // No leading untagged text → no spurious Neutral prefix segment.
    let result = segment("[emotion:Happy]hello");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].emotion, Emotion::Happy);
    assert_eq!(result[0].text, "hello");
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_consecutive_markers_emit_no_empty_segment() {
    // [emotion:Happy][emotion:Sad]text — the Happy span is empty so it is dropped.
    let result = segment("[emotion:Happy][emotion:Sad]text");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].emotion, Emotion::Sad);
    assert_eq!(result[0].text, "text");
}

#[test]
fn c3_marker_at_end_emits_no_empty_segment() {
    // A trailing marker with no text after it produces no empty segment.
    let result = segment("text[emotion:Happy]");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].emotion, Emotion::Neutral);
    assert_eq!(result[0].text, "text");
}

#[test]
fn c3_multiple_consecutive_markers_all_dropped() {
    // A run of markers with no text between them — only the last one's span survives.
    let result = segment("[emotion:Happy][emotion:Sad][emotion:Calm]final");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].emotion, Emotion::Calm);
    assert_eq!(result[0].text, "final");
}

#[test]
fn c3_only_markers_returns_empty_vec() {
    // A string consisting only of markers has no text spans → empty Vec.
    let result = segment("[emotion:Happy][emotion:Sad]");
    assert!(result.is_empty());
}
