//! Frozen tests for T-01.02 — "Strip the emotion tags".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_single_marker_removed, c1_surrounding_text_preserved,
//!          c1_multiple_markers_all_removed, c1_marker_at_start_removed,
//!          c1_marker_at_end_removed, c1_only_marker_becomes_empty
//!   C2 -> c2_no_marker_returns_equal_string, c2_empty_input_returns_empty_string

use zira_emotion::strip_tags;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_single_marker_removed() {
    // A string with exactly one marker has the marker excised; everything else is intact.
    let result = strip_tags("[emotion:Happy]hello");
    assert_eq!(result, "hello");
}

#[test]
fn c1_surrounding_text_preserved() {
    // Text before and after the marker must survive unchanged (including any spaces).
    // "no trimming or normalising prose beyond marker removal" — the double space stays.
    let result = strip_tags("before [emotion:Sad] after");
    assert_eq!(result, "before  after");
}

#[test]
fn c1_multiple_markers_all_removed() {
    // Every marker is stripped, leaving all prose intact.
    let result = strip_tags("[emotion:Happy]Hello! [emotion:Sad]Goodbye.");
    assert_eq!(result, "Hello! Goodbye.");
}

#[test]
fn c1_marker_at_start_removed() {
    let result = strip_tags("[emotion:Excited]Let's go");
    assert_eq!(result, "Let's go");
}

#[test]
fn c1_marker_at_end_removed() {
    let result = strip_tags("Wrapping up[emotion:Calm]");
    assert_eq!(result, "Wrapping up");
}

#[test]
fn c1_only_marker_becomes_empty() {
    // A string that is nothing but a marker collapses to an empty String.
    let result = strip_tags("[emotion:Neutral]");
    assert_eq!(result, "");
}

#[test]
fn c1_consecutive_markers_all_removed() {
    // Multiple adjacent markers are all stripped; no residual brackets remain.
    let result = strip_tags("[emotion:Happy][emotion:Sad]text");
    assert_eq!(result, "text");
}

#[test]
fn c1_all_known_variant_markers_removed() {
    // Every variant name in a tagged string is stripped correctly.
    let input = "[emotion:Neutral]a[emotion:Happy]b[emotion:Sad]c\
                 [emotion:Angry]d[emotion:Excited]e[emotion:Calm]f\
                 [emotion:Curious]g[emotion:Concerned]h[emotion:Playful]i\
                 [emotion:Tired]j";
    let result = strip_tags(input);
    assert_eq!(result, "abcdefghij");
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_no_marker_returns_equal_string() {
    let input = "just plain text with no markers";
    let result = strip_tags(input);
    assert_eq!(result, input);
}

#[test]
fn c2_empty_input_returns_empty_string() {
    let result = strip_tags("");
    assert_eq!(result, "");
}
