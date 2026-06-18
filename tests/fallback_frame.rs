//! Frozen tests for T-03.08 — "Describe the fallback frame".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_fallback_frame_derives_debug, c1_fallback_frame_derives_clone,
//!          c1_fallback_frame_derives_partial_eq, c1_mouth_passthrough_sil,
//!          c1_mouth_passthrough_non_sil
//!   C2 -> c2_neutral_expression_yields_neutral_sprite,
//!          c2_joy_dominant_yields_happy_sprite

use zira_avatar::{fallback_frame, AvatarState, ExpressionPreset, FallbackFrame, Viseme};

// ---- C1 — FallbackFrame struct shape + derives + mouth passthrough -----------------

#[test]
fn c1_fallback_frame_derives_debug() {
    let f = FallbackFrame { sprite: "neutral".to_string(), mouth: Viseme::Sil };
    let out = format!("{f:?}");
    assert!(out.contains("FallbackFrame"), "Debug output must name the struct");
}

#[test]
fn c1_fallback_frame_derives_clone() {
    let a = FallbackFrame { sprite: "neutral".to_string(), mouth: Viseme::Sil };
    let b = a.clone();
    assert_eq!(a, b, "Clone must produce a value equal to the original");
}

#[test]
fn c1_fallback_frame_derives_partial_eq() {
    let a = FallbackFrame { sprite: "neutral".to_string(), mouth: Viseme::Sil };
    let b = FallbackFrame { sprite: "neutral".to_string(), mouth: Viseme::Sil };
    assert_eq!(a, b, "identical FallbackFrames must be equal");
    let c = FallbackFrame { sprite: "happy".to_string(), mouth: Viseme::Sil };
    assert_ne!(a, c, "differing sprite must make FallbackFrames unequal");
}

#[test]
fn c1_mouth_passthrough_sil() {
    let state = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::Sil };
    let frame = fallback_frame(&state);
    assert_eq!(frame.mouth, Viseme::Sil, "mouth must equal the state's mouth (Sil)");
}

#[test]
fn c1_mouth_passthrough_non_sil() {
    let state = AvatarState { expression: ExpressionPreset::neutral(), mouth: Viseme::A };
    let frame = fallback_frame(&state);
    assert_eq!(frame.mouth, Viseme::A, "mouth must pass through unchanged from the state");
}

// ---- C2 — sprite name chosen from dominant expression weight -----------------------

#[test]
fn c2_neutral_expression_yields_neutral_sprite() {
    // AvatarState::resting() has all blendshape weights at zero.
    let state = AvatarState::resting();
    let frame = fallback_frame(&state);
    assert_eq!(frame.sprite, "neutral", "all-zero expression must yield sprite \"neutral\"");
}

#[test]
fn c2_joy_dominant_yields_happy_sprite() {
    let state = AvatarState {
        expression: ExpressionPreset {
            joy: 0.8,
            sorrow: 0.0,
            anger: 0.0,
            surprise: 0.0,
            fun: 0.0,
        },
        mouth: Viseme::Sil,
    };
    let frame = fallback_frame(&state);
    assert_eq!(frame.sprite, "happy", "joy-dominant expression must yield sprite \"happy\"");
}
