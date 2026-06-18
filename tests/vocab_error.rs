//! Frozen tests for T-05.05 — "Define the vocabulary error".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_unknown_tag_variant_exists_and_derives_debug
//!   C2 -> c2_display_contains_offending_tag

use zira_config::VocabError;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_unknown_tag_variant_exists_and_derives_debug() {
    let err = VocabError::UnknownTag {
        tag: "mystery".to_string(),
    };
    // Debug is derived — formats without panic.
    let debug = format!("{err:?}");
    assert!(
        debug.contains("UnknownTag"),
        "Debug output must name the variant; got: {debug:?}"
    );
    assert!(
        debug.contains("mystery"),
        "Debug output must include the tag value; got: {debug:?}"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_display_contains_offending_tag() {
    let tag = "bogus_emotion".to_string();
    let err = VocabError::UnknownTag { tag: tag.clone() };
    let msg = err.to_string();
    assert!(
        msg.contains(&tag),
        "Display must name the offending tag; got: {msg:?}"
    );
}
