//! Frozen tests for T-03.02 — "Map emotion to expression".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_expression_for_total_and_bounded
//!   C2 -> c2_neutral_maps_to_neutral_preset
//!   C3 -> c3_distinct_emotions_map_to_distinct_presets

use zira_avatar::{expression_for, ExpressionPreset};
use zira_proto::Emotion;

// ---- C1 — expression_for is total and all returned presets are already clamped -------

#[test]
fn c1_expression_for_total_and_bounded() {
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
    for emotion in variants {
        let preset = expression_for(emotion);
        let clamped = preset.clamped();
        assert_eq!(
            preset, clamped,
            "expression_for({emotion:?}) returned a preset with out-of-range weights"
        );
    }
}

// ---- C2 — Neutral maps to the all-zeros preset -------------------------------------

#[test]
fn c2_neutral_maps_to_neutral_preset() {
    assert_eq!(
        expression_for(Emotion::Neutral),
        ExpressionPreset::neutral(),
        "expression_for(Neutral) must equal ExpressionPreset::neutral() (all weights 0.0)"
    );
}

// ---- C3 — the table is not a constant (at least two emotions differ) ---------------

#[test]
fn c3_distinct_emotions_map_to_distinct_presets() {
    let happy = expression_for(Emotion::Happy);
    let sad = expression_for(Emotion::Sad);
    assert_ne!(
        happy, sad,
        "expression_for(Happy) and expression_for(Sad) must be distinct presets"
    );
}
