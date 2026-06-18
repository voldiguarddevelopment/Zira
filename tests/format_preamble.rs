//! Frozen tests for T-02.17 — "Format the context preamble".
//!
//! C1 — `zira_memory::format_preamble(episodes: &[zira_memory::Episode]) -> String`
//!       renders the retrieved episodes into a single prompt-preamble string;
//!       an empty slice returns an empty string (no preamble).
//! C2 — a test asserts the preamble for two episodes contains both episodes'
//!       `text` substrings, and that an empty slice yields exactly an empty string.
//!
//! Acceptance criterion mapping:
//!   C1 -> c1_empty_slice_returns_empty_string, c1_nonempty_slice_returns_nonempty_string
//!   C2 -> c2_preamble_contains_both_episode_texts, c2_empty_slice_yields_exact_empty_string

use zira_memory::{format_preamble, Episode};

fn ep(role: &str, text: &str, ts: u64) -> Episode {
    Episode { role: role.to_string(), text: text.to_string(), timestamp: ts }
}

// ---- C1 -----------------------------------------------------------------------

/// An empty episode slice must produce exactly an empty string — no header, no
/// whitespace, no noise injected into the prompt.
#[test]
fn c1_empty_slice_returns_empty_string() {
    let result = format_preamble(&[]);
    assert_eq!(result, "", "format_preamble(&[]) must return an empty string");
}

/// A non-empty slice must produce a non-empty preamble string — something is
/// rendered for the caller to prepend.
#[test]
fn c1_nonempty_slice_returns_nonempty_string() {
    let episodes = vec![ep("user", "hello world", 1)];
    let result = format_preamble(&episodes);
    assert!(
        !result.is_empty(),
        "format_preamble with one episode must return a non-empty string"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// The preamble for two episodes must contain both episodes' `text` substrings
/// so the injected context carries the actual conversation content.
#[test]
fn c2_preamble_contains_both_episode_texts() {
    let text_a = "the quick brown fox";
    let text_b = "jumped over the lazy dog";
    let episodes = vec![ep("user", text_a, 1), ep("assistant", text_b, 2)];

    let result = format_preamble(&episodes);

    assert!(
        result.contains(text_a),
        "preamble must contain the first episode's text {:?}; got: {:?}",
        text_a,
        result,
    );
    assert!(
        result.contains(text_b),
        "preamble must contain the second episode's text {:?}; got: {:?}",
        text_b,
        result,
    );
}

/// An empty slice must yield exactly an empty string — not a header-only string,
/// not whitespace, not a newline.
#[test]
fn c2_empty_slice_yields_exact_empty_string() {
    let result = format_preamble(&[]);
    assert_eq!(
        result, "",
        "format_preamble(&[]) must be exactly \"\" (len={})",
        result.len(),
    );
}
