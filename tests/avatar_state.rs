//! Frozen tests for T-03.07 — "Define the avatar state".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_struct_fields, c1_derives_debug, c1_derives_clone,
//!          c1_derives_partial_eq, c1_resting_expression,
//!          c1_resting_mouth, c1_resting_full
//!   C2 -> c2_for_emotion_expression_tracks_emotion,
//!          c2_for_emotion_mouth_is_sil,
//!          c2_for_emotion_neutral,
//!          c2_for_emotion_all_variants

use zira_avatar::{expression_for, AvatarState, ExpressionPreset, Viseme};
use zira_proto::Emotion;

// ---- C1 — struct shape + derives + resting() constructor ---------------------------

#[test]
fn c1_struct_fields() {
    // Both named fields must exist and accept the correct types.
    let s = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    assert_eq!(s.expression, ExpressionPreset::neutral(), "expression field must be accessible");
    assert_eq!(s.mouth, Viseme::Sil, "mouth field must be accessible and hold Viseme::Sil");
}

#[test]
fn c1_derives_debug() {
    let s = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    let out = format!("{:?}", s);
    assert!(out.contains("AvatarState"), "Debug output must name the struct");
}

#[test]
fn c1_derives_clone() {
    let a = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    let b = a.clone();
    assert_eq!(a, b, "Clone must produce a value equal to the original");
}

#[test]
fn c1_derives_partial_eq() {
    let a = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    let b = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    assert_eq!(a, b, "two identical AvatarState values must be equal via PartialEq");
    let c = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::A };
    assert_ne!(a, c, "differing mouth fields must make AvatarState values unequal");
}

#[test]
fn c1_resting_expression() {
    let r = AvatarState::resting();
    assert_eq!(
        r.expression,
        ExpressionPreset::neutral(),
        "resting().expression must equal ExpressionPreset::neutral()"
    );
}

#[test]
fn c1_resting_mouth() {
    let r = AvatarState::resting();
    assert_eq!(r.mouth, Viseme::Sil, "resting().mouth must be Viseme::Sil");
}

#[test]
fn c1_resting_full() {
    let expected = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    assert_eq!(
        AvatarState::resting(),
        expected,
        "resting() must equal {{ expression: neutral(), mouth: Sil }}"
    );
}

// ---- C2 — for_emotion(e) builds a state from the emotion map -----------------------

#[test]
fn c2_for_emotion_expression_tracks_emotion() {
    let state = AvatarState::for_emotion(Emotion::Happy);
    let expected_expr = expression_for(Emotion::Happy);
    assert_eq!(
        state.expression,
        expected_expr,
        "for_emotion(Happy).expression must equal expression_for(Happy)"
    );
}

#[test]
fn c2_for_emotion_mouth_is_sil() {
    let state = AvatarState::for_emotion(Emotion::Happy);
    assert_eq!(state.mouth, Viseme::Sil, "for_emotion(Happy).mouth must be Viseme::Sil");
}

#[test]
fn c2_for_emotion_neutral() {
    let state = AvatarState::for_emotion(Emotion::Neutral);
    assert_eq!(
        state.expression,
        expression_for(Emotion::Neutral),
        "for_emotion(Neutral).expression must equal expression_for(Neutral)"
    );
    assert_eq!(state.mouth, Viseme::Sil, "for_emotion(Neutral).mouth must be Viseme::Sil");
}

#[test]
fn c2_for_emotion_all_variants() {
    let emotions = [
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
    for e in emotions {
        let state = AvatarState::for_emotion(e);
        assert_eq!(
            state.expression,
            expression_for(e),
            "for_emotion({e:?}).expression must equal expression_for({e:?})"
        );
        assert_eq!(
            state.mouth,
            Viseme::Sil,
            "for_emotion({e:?}).mouth must be Viseme::Sil"
        );
    }
}
