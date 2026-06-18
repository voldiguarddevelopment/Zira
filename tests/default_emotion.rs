//! Frozen tests for T-05.04 — "Resolve the default emotion".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_known_tag_maps_to_matching_emotion_variant,
//!          c1_known_tag_is_case_insensitive
//!   C2 -> c2_empty_string_maps_to_neutral,
//!          c2_unknown_tag_maps_to_neutral

use zira_config::{resolve_default_emotion, EmotionConfig};
use zira_proto::Emotion;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_known_tag_maps_to_matching_emotion_variant() {
    let config = EmotionConfig {
        default_emotion: "happy".to_string(),
    };
    assert_eq!(resolve_default_emotion(&config), Emotion::Happy);
}

#[test]
fn c1_known_tag_is_case_insensitive() {
    let upper = EmotionConfig {
        default_emotion: "HAPPY".to_string(),
    };
    assert_eq!(resolve_default_emotion(&upper), Emotion::Happy);

    let mixed = EmotionConfig {
        default_emotion: "Excited".to_string(),
    };
    assert_eq!(resolve_default_emotion(&mixed), Emotion::Excited);
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_empty_string_maps_to_neutral() {
    let config = EmotionConfig {
        default_emotion: String::new(),
    };
    assert_eq!(resolve_default_emotion(&config), Emotion::Neutral);
}

#[test]
fn c2_unknown_tag_maps_to_neutral() {
    let config = EmotionConfig {
        default_emotion: "no_such_emotion".to_string(),
    };
    assert_eq!(resolve_default_emotion(&config), Emotion::Neutral);
}
