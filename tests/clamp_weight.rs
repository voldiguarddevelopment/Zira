//! Frozen tests for T-03.05 — "Clamp the viseme weight".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_below_zero_returns_zero, c1_above_one_returns_one,
//!          c1_in_range_returns_unchanged, c1_exactly_zero_returns_zero,
//!          c1_exactly_one_returns_one
//!   C2 -> c2_nan_returns_zero

use zira_avatar::clamp_weight;

// ---- C1 — in-range and boundary values are clamped correctly ---------------------

#[test]
fn c1_below_zero_returns_zero() {
    assert_eq!(clamp_weight(-0.5), 0.0, "weight below 0.0 must be clamped to 0.0");
    assert_eq!(clamp_weight(-1.0), 0.0, "weight -1.0 must be clamped to 0.0");
    assert_eq!(clamp_weight(f32::NEG_INFINITY), 0.0, "-inf must be clamped to 0.0");
}

#[test]
fn c1_above_one_returns_one() {
    assert_eq!(clamp_weight(1.5), 1.0, "weight above 1.0 must be clamped to 1.0");
    assert_eq!(clamp_weight(2.0), 1.0, "weight 2.0 must be clamped to 1.0");
    assert_eq!(clamp_weight(f32::INFINITY), 1.0, "+inf must be clamped to 1.0");
}

#[test]
fn c1_in_range_returns_unchanged() {
    assert_eq!(clamp_weight(0.5), 0.5, "in-range weight 0.5 must pass through unchanged");
    assert_eq!(clamp_weight(0.25), 0.25, "in-range weight 0.25 must pass through unchanged");
    assert_eq!(clamp_weight(0.75), 0.75, "in-range weight 0.75 must pass through unchanged");
}

#[test]
fn c1_exactly_zero_returns_zero() {
    assert_eq!(clamp_weight(0.0), 0.0, "exactly 0.0 is the inclusive lower bound");
}

#[test]
fn c1_exactly_one_returns_one() {
    assert_eq!(clamp_weight(1.0), 1.0, "exactly 1.0 is the inclusive upper bound");
}

// ---- C2 — NaN collapses to the rest weight 0.0 instead of propagating -----------

#[test]
fn c2_nan_returns_zero() {
    let result = clamp_weight(f32::NAN);
    assert!(
        result == 0.0,
        "NaN input must collapse to rest weight 0.0, got {result}"
    );
}
