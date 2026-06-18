//! Frozen tests for T-05.06 — "Validate the emotion vocabulary".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_all_ten_variants_resolve, c1_case_insensitive
//!   C2 -> c2_first_unknown_tag_rejected, c2_stops_at_first_unknown
//!   C3 -> c3_empty_slice_is_ok

use zira_config::{validate_vocab, VocabError};
use zira_proto::Emotion;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_all_ten_variants_resolve() {
    let tags: Vec<String> = [
        "Neutral", "Happy", "Sad", "Angry", "Excited", "Calm", "Curious", "Concerned",
        "Playful", "Tired",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let result = validate_vocab(&tags);
    assert!(result.is_ok(), "all known tags should resolve; got: {result:?}");
    let emotions = result.unwrap();
    assert_eq!(emotions.len(), 10, "one Emotion per input tag");
    assert_eq!(emotions[0], Emotion::Neutral);
    assert_eq!(emotions[1], Emotion::Happy);
    assert_eq!(emotions[2], Emotion::Sad);
    assert_eq!(emotions[3], Emotion::Angry);
    assert_eq!(emotions[4], Emotion::Excited);
    assert_eq!(emotions[5], Emotion::Calm);
    assert_eq!(emotions[6], Emotion::Curious);
    assert_eq!(emotions[7], Emotion::Concerned);
    assert_eq!(emotions[8], Emotion::Playful);
    assert_eq!(emotions[9], Emotion::Tired);
}

#[test]
fn c1_case_insensitive() {
    let tags: Vec<String> = vec![
        "HAPPY".to_string(),
        "neutral".to_string(),
        "Excited".to_string(),
    ];
    let result = validate_vocab(&tags);
    assert!(
        result.is_ok(),
        "case-insensitive match must succeed; got: {result:?}"
    );
    let emotions = result.unwrap();
    assert_eq!(
        emotions,
        vec![Emotion::Happy, Emotion::Neutral, Emotion::Excited]
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_first_unknown_tag_rejected() {
    let tags: Vec<String> = vec!["unknown_mood".to_string()];
    let result = validate_vocab(&tags);
    match result {
        Err(VocabError::UnknownTag { tag }) => {
            assert_eq!(tag, "unknown_mood", "error must name the offending tag");
        }
        other => panic!("expected Err(UnknownTag), got: {other:?}"),
    }
}

#[test]
fn c2_stops_at_first_unknown() {
    // "happy" is known, "mystery" is not — error must name "mystery", not "sad".
    let tags: Vec<String> = vec![
        "happy".to_string(),
        "mystery".to_string(),
        "sad".to_string(),
    ];
    let result = validate_vocab(&tags);
    match result {
        Err(VocabError::UnknownTag { tag }) => {
            assert_eq!(tag, "mystery", "must report the FIRST unknown tag");
        }
        other => panic!(
            "expected Err(UnknownTag {{ tag: \"mystery\" }}), got: {other:?}"
        ),
    }
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_empty_slice_is_ok() {
    let result = validate_vocab(&[]);
    assert!(result.is_ok(), "empty slice must yield Ok; got: {result:?}");
    assert_eq!(result.unwrap(), Vec::<Emotion>::new(), "empty input → empty Vec");
}
