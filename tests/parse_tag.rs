//! Frozen tests for T-01.01 — "Parse the emotion tag".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_leading_marker_extracts_emotion, c1_leading_whitespace_trimmed,
//!          c1_case_insensitive_name, c1_unknown_name_resolves_to_neutral,
//!          c1_all_known_variants_parseable
//!   C2 -> c2_no_marker_returns_neutral_and_input,
//!          c2_empty_input_returns_neutral_and_empty,
//!          c2_no_leading_marker_slice_is_same_bytes,
//!          c2_marker_not_at_start_is_no_op

use zira_emotion::parse_tag;
use zira_proto::Emotion;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_leading_marker_extracts_emotion() {
    let (emotion, rest) = parse_tag("[emotion:Happy] hello");
    assert_eq!(emotion, Emotion::Happy);
    assert_eq!(rest, "hello");
}

#[test]
fn c1_leading_whitespace_trimmed() {
    // Multiple spaces after the closing bracket must be stripped.
    let (emotion, rest) = parse_tag("[emotion:Sad]   trimmed text");
    assert_eq!(emotion, Emotion::Sad);
    assert_eq!(rest, "trimmed text");

    // No trailing text at all → empty slice after trim.
    let (emotion2, rest2) = parse_tag("[emotion:Calm]");
    assert_eq!(emotion2, Emotion::Calm);
    assert_eq!(rest2, "");

    // Marker followed by only whitespace → empty after trim.
    let (emotion3, rest3) = parse_tag("[emotion:Angry]   ");
    assert_eq!(emotion3, Emotion::Angry);
    assert_eq!(rest3, "");
}

#[test]
fn c1_case_insensitive_name() {
    let (e1, _) = parse_tag("[emotion:HAPPY] x");
    assert_eq!(e1, Emotion::Happy);

    let (e2, _) = parse_tag("[emotion:happy] x");
    assert_eq!(e2, Emotion::Happy);

    let (e3, _) = parse_tag("[emotion:HaPpY] x");
    assert_eq!(e3, Emotion::Happy);

    let (e4, _) = parse_tag("[emotion:NEUTRAL] x");
    assert_eq!(e4, Emotion::Neutral);
}

#[test]
fn c1_unknown_name_resolves_to_neutral() {
    // An unrecognised tag name delegates to Emotion::from_tag which maps unknowns to Neutral.
    let (emotion, rest) = parse_tag("[emotion:FlyingPig] after");
    assert_eq!(emotion, Emotion::Neutral);
    assert_eq!(rest, "after");

    let (emotion2, rest2) = parse_tag("[emotion:] trailing");
    assert_eq!(emotion2, Emotion::Neutral);
    assert_eq!(rest2, "trailing");
}

#[test]
fn c1_all_known_variants_parseable() {
    let cases = [
        ("[emotion:Neutral] t", Emotion::Neutral),
        ("[emotion:Happy] t", Emotion::Happy),
        ("[emotion:Sad] t", Emotion::Sad),
        ("[emotion:Angry] t", Emotion::Angry),
        ("[emotion:Excited] t", Emotion::Excited),
        ("[emotion:Calm] t", Emotion::Calm),
        ("[emotion:Curious] t", Emotion::Curious),
        ("[emotion:Concerned] t", Emotion::Concerned),
        ("[emotion:Playful] t", Emotion::Playful),
        ("[emotion:Tired] t", Emotion::Tired),
    ];
    for (input, expected) in cases {
        let (emotion, rest) = parse_tag(input);
        assert_eq!(emotion, expected, "input: {input:?}");
        assert_eq!(rest, "t", "input: {input:?}");
    }
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_no_marker_returns_neutral_and_input() {
    let input = "just plain text";
    let (emotion, rest) = parse_tag(input);
    assert_eq!(emotion, Emotion::Neutral);
    assert_eq!(rest, input);
}

#[test]
fn c2_empty_input_returns_neutral_and_empty() {
    let (emotion, rest) = parse_tag("");
    assert_eq!(emotion, Emotion::Neutral);
    assert_eq!(rest, "");
}

#[test]
fn c2_no_leading_marker_slice_is_same_bytes() {
    // The returned slice must be byte-for-byte the original (same pointer + length,
    // no allocation or copy).
    let input = "no marker here";
    let (emotion, rest) = parse_tag(input);
    assert_eq!(emotion, Emotion::Neutral);
    // Pointer identity: rest must be the exact same memory region as input.
    assert!(
        std::ptr::eq(rest.as_ptr(), input.as_ptr()),
        "rest must share the same pointer as input when there is no marker"
    );
    assert_eq!(rest.len(), input.len());
}

#[test]
fn c2_marker_not_at_start_is_no_op() {
    // A marker embedded in the middle of the string is NOT at the start → no-op.
    let input = "hello [emotion:Happy] world";
    let (emotion, rest) = parse_tag(input);
    assert_eq!(emotion, Emotion::Neutral);
    assert_eq!(rest, input);
}
