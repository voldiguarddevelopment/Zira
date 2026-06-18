//! Frozen tests for T-03.01 — "Define the expression preset".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_struct_fields, c1_neutral_all_zeros, c1_neutral_debug,
//!          c1_neutral_clone, c1_neutral_partial_eq
//!   C2 -> c2_clamped_in_range_unchanged, c2_clamped_above_one,
//!          c2_clamped_below_zero, c2_clamped_mixed, c2_clamped_neutral_is_noop

use zira_avatar::ExpressionPreset;

// ---- C1 — struct shape + neutral() constructor -------------------------------------

#[test]
fn c1_struct_fields() {
    // All five named fields must exist and accept f32 literals.
    let p = ExpressionPreset { joy: 0.0, sorrow: 0.0, anger: 0.0, surprise: 0.0, fun: 0.0 };
    assert_eq!(p.joy, 0.0_f32);
    assert_eq!(p.sorrow, 0.0_f32);
    assert_eq!(p.anger, 0.0_f32);
    assert_eq!(p.surprise, 0.0_f32);
    assert_eq!(p.fun, 0.0_f32);
}

#[test]
fn c1_neutral_all_zeros() {
    let n = ExpressionPreset::neutral();
    assert_eq!(n.joy, 0.0, "joy must be 0.0 in neutral()");
    assert_eq!(n.sorrow, 0.0, "sorrow must be 0.0 in neutral()");
    assert_eq!(n.anger, 0.0, "anger must be 0.0 in neutral()");
    assert_eq!(n.surprise, 0.0, "surprise must be 0.0 in neutral()");
    assert_eq!(n.fun, 0.0, "fun must be 0.0 in neutral()");
}

#[test]
fn c1_neutral_debug() {
    // Debug must be derived — formatting must not panic and must name the struct.
    let s = format!("{:?}", ExpressionPreset::neutral());
    assert!(s.contains("ExpressionPreset"), "Debug output must include the struct name");
}

#[test]
fn c1_neutral_clone() {
    let a = ExpressionPreset::neutral();
    let b = a.clone();
    assert_eq!(a, b, "Clone must produce a value equal to the original");
}

#[test]
fn c1_neutral_partial_eq() {
    let a = ExpressionPreset::neutral();
    let b = ExpressionPreset::neutral();
    assert_eq!(a, b, "two neutral() calls must be equal via PartialEq");
}

// ---- C2 — clamped() constrains weights to 0.0..=1.0 -------------------------------

#[test]
fn c2_clamped_in_range_unchanged() {
    let p = ExpressionPreset { joy: 0.5, sorrow: 0.0, anger: 1.0, surprise: 0.25, fun: 0.75 };
    let c = p.clamped();
    assert_eq!(c.joy, 0.5, "in-range joy must be unchanged by clamped()");
    assert_eq!(c.sorrow, 0.0, "boundary 0.0 must be unchanged by clamped()");
    assert_eq!(c.anger, 1.0, "boundary 1.0 must be unchanged by clamped()");
    assert_eq!(c.surprise, 0.25, "in-range surprise must be unchanged by clamped()");
    assert_eq!(c.fun, 0.75, "in-range fun must be unchanged by clamped()");
}

#[test]
fn c2_clamped_above_one() {
    let p = ExpressionPreset { joy: 1.5, sorrow: 2.0, anger: 1.1, surprise: 99.0, fun: 1.01 };
    let c = p.clamped();
    assert_eq!(c.joy, 1.0, "joy 1.5 must clamp to 1.0");
    assert_eq!(c.sorrow, 1.0, "sorrow 2.0 must clamp to 1.0");
    assert_eq!(c.anger, 1.0, "anger 1.1 must clamp to 1.0");
    assert_eq!(c.surprise, 1.0, "surprise 99.0 must clamp to 1.0");
    assert_eq!(c.fun, 1.0, "fun 1.01 must clamp to 1.0");
}

#[test]
fn c2_clamped_below_zero() {
    let p =
        ExpressionPreset { joy: -0.1, sorrow: -1.0, anger: -0.5, surprise: -99.0, fun: -0.01 };
    let c = p.clamped();
    assert_eq!(c.joy, 0.0, "joy -0.1 must clamp to 0.0");
    assert_eq!(c.sorrow, 0.0, "sorrow -1.0 must clamp to 0.0");
    assert_eq!(c.anger, 0.0, "anger -0.5 must clamp to 0.0");
    assert_eq!(c.surprise, 0.0, "surprise -99.0 must clamp to 0.0");
    assert_eq!(c.fun, 0.0, "fun -0.01 must clamp to 0.0");
}

#[test]
fn c2_clamped_mixed() {
    // Mix of below-zero, in-range, and above-one values across fields.
    let p = ExpressionPreset { joy: -0.5, sorrow: 0.5, anger: 1.5, surprise: 0.0, fun: 1.0 };
    let c = p.clamped();
    assert_eq!(c.joy, 0.0, "joy -0.5 must clamp to 0.0");
    assert_eq!(c.sorrow, 0.5, "sorrow 0.5 in-range must be unchanged");
    assert_eq!(c.anger, 1.0, "anger 1.5 must clamp to 1.0");
    assert_eq!(c.surprise, 0.0, "surprise 0.0 boundary must be unchanged");
    assert_eq!(c.fun, 1.0, "fun 1.0 boundary must be unchanged");
}

#[test]
fn c2_clamped_neutral_is_noop() {
    // All weights in neutral() are 0.0 which is already in [0.0, 1.0].
    let n = ExpressionPreset::neutral();
    let c = n.clone().clamped();
    assert_eq!(
        c, n,
        "clamping a neutral preset must return an equal preset (all weights already in range)"
    );
}
