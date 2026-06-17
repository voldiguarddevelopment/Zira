//! Frozen tests for T-00.05 — "Define the Emotion type".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_default_is_neutral, c1_all_ten_variants_exist
//!   C2 -> c2_serde_json_round_trip
//!   C3 -> c3_from_tag_unknown_maps_to_neutral, c3_from_tag_case_insensitive

use zira_proto::Emotion;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_default_is_neutral() {
    assert_eq!(Emotion::default(), Emotion::Neutral);
}

#[test]
fn c1_all_ten_variants_exist() {
    let variants = [
        Emotion::Neutral,
        Emotion::Happy,
        Emotion::Sad,
        Emotion::Angry,
        Emotion::Excited,
        Emotion::Calm,
        Emotion::Curious,
        Emotion::Concerned,
        Emotion::Playful,
        Emotion::Tired,
    ];
    assert_eq!(variants.len(), 10, "exactly ten Emotion variants must exist");
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_serde_json_round_trip() {
    let variants = [
        Emotion::Neutral,
        Emotion::Happy,
        Emotion::Sad,
        Emotion::Angry,
        Emotion::Excited,
        Emotion::Calm,
        Emotion::Curious,
        Emotion::Concerned,
        Emotion::Playful,
        Emotion::Tired,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant)
            .unwrap_or_else(|e| panic!("serialize {variant:?}: {e}"));
        let back: Emotion = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("deserialize {json:?}: {e}"));
        assert_eq!(
            variant, back,
            "serde round-trip must preserve variant identity"
        );
    }
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_from_tag_unknown_maps_to_neutral() {
    assert_eq!(Emotion::from_tag("unknown_xyz"), Emotion::Neutral);
    assert_eq!(Emotion::from_tag(""), Emotion::Neutral);
    assert_eq!(Emotion::from_tag("!!!"), Emotion::Neutral);
    assert_eq!(Emotion::from_tag("42"), Emotion::Neutral);
}

#[test]
fn c3_from_tag_case_insensitive() {
    assert_eq!(Emotion::from_tag("neutral"), Emotion::Neutral);
    assert_eq!(Emotion::from_tag("NEUTRAL"), Emotion::Neutral);
    assert_eq!(Emotion::from_tag("happy"), Emotion::Happy);
    assert_eq!(Emotion::from_tag("HAPPY"), Emotion::Happy);
    assert_eq!(Emotion::from_tag("Happy"), Emotion::Happy);
    assert_eq!(Emotion::from_tag("sad"), Emotion::Sad);
    assert_eq!(Emotion::from_tag("SAD"), Emotion::Sad);
    assert_eq!(Emotion::from_tag("angry"), Emotion::Angry);
    assert_eq!(Emotion::from_tag("ANGRY"), Emotion::Angry);
    assert_eq!(Emotion::from_tag("excited"), Emotion::Excited);
    assert_eq!(Emotion::from_tag("EXCITED"), Emotion::Excited);
    assert_eq!(Emotion::from_tag("calm"), Emotion::Calm);
    assert_eq!(Emotion::from_tag("CALM"), Emotion::Calm);
    assert_eq!(Emotion::from_tag("curious"), Emotion::Curious);
    assert_eq!(Emotion::from_tag("CURIOUS"), Emotion::Curious);
    assert_eq!(Emotion::from_tag("concerned"), Emotion::Concerned);
    assert_eq!(Emotion::from_tag("CONCERNED"), Emotion::Concerned);
    assert_eq!(Emotion::from_tag("playful"), Emotion::Playful);
    assert_eq!(Emotion::from_tag("PLAYFUL"), Emotion::Playful);
    assert_eq!(Emotion::from_tag("tired"), Emotion::Tired);
    assert_eq!(Emotion::from_tag("TIRED"), Emotion::Tired);
}
