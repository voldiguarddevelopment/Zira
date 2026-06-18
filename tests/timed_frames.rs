//! Frozen tests for T-03.06 — "Order the viseme frames".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_single_frame_time_is_zero, c1_three_frames_monotonic_times,
//!          c1_count_matches_input, c1_viseme_order_preserved
//!   C2 -> c2_weight_above_one_clamped, c2_weight_below_zero_clamped,
//!          c2_in_range_weight_unchanged, c2_nan_weight_collapsed
//!   C3 -> c3_empty_input_returns_empty

use zira_avatar::{clamp_weight, timed_frames, Viseme, VisemeFrame};

fn frame(viseme: Viseme, weight: f32) -> VisemeFrame {
    VisemeFrame { viseme, weight }
}

// ---- C1 — one entry per input frame, in input order, monotonically increasing start times ----

#[test]
fn c1_single_frame_time_is_zero() {
    let frames = [frame(Viseme::A, 0.5)];
    let result = timed_frames(&frames, 40);
    assert_eq!(result.len(), 1, "one input frame must produce one output entry");
    assert_eq!(result[0].0, 0, "first frame must start at 0 ms");
}

#[test]
fn c1_three_frames_monotonic_times() {
    let frames = [frame(Viseme::A, 0.5), frame(Viseme::I, 0.3), frame(Viseme::U, 0.7)];
    let result = timed_frames(&frames, 100);
    assert_eq!(result.len(), 3, "three input frames must produce three output entries");
    assert_eq!(result[0].0, 0, "first start_ms must be 0");
    assert_eq!(result[1].0, 100, "second start_ms must be frame_ms (100)");
    assert_eq!(result[2].0, 200, "third start_ms must be 2 * frame_ms (200)");
}

#[test]
fn c1_count_matches_input() {
    let frames: Vec<VisemeFrame> = (0..7_u8).map(|i| frame(Viseme::Sil, f32::from(i) / 10.0)).collect();
    let result = timed_frames(&frames, 33);
    assert_eq!(result.len(), frames.len(), "output count must equal input count");
}

#[test]
fn c1_viseme_order_preserved() {
    let frames = [frame(Viseme::A, 0.5), frame(Viseme::E, 0.4), frame(Viseme::Sil, 0.1)];
    let result = timed_frames(&frames, 80);
    assert_eq!(result.len(), 3, "output length must equal input length");
    assert_eq!(result[0].1.viseme, Viseme::A, "first output viseme must match first input");
    assert_eq!(result[1].1.viseme, Viseme::E, "second output viseme must match second input");
    assert_eq!(result[2].1.viseme, Viseme::Sil, "third output viseme must match third input");
}

// ---- C2 — returned weight equals clamp_weight applied to the input weight ----

#[test]
fn c2_weight_above_one_clamped() {
    let frames = [frame(Viseme::A, 1.5)];
    let result = timed_frames(&frames, 40);
    assert_eq!(
        result[0].1.weight,
        clamp_weight(1.5),
        "weight 1.5 must be clamped to 1.0"
    );
}

#[test]
fn c2_weight_below_zero_clamped() {
    let frames = [frame(Viseme::I, -0.3)];
    let result = timed_frames(&frames, 40);
    assert_eq!(
        result[0].1.weight,
        clamp_weight(-0.3),
        "weight -0.3 must be clamped to 0.0"
    );
}

#[test]
fn c2_in_range_weight_unchanged() {
    let frames = [frame(Viseme::U, 0.6)];
    let result = timed_frames(&frames, 40);
    assert_eq!(
        result[0].1.weight,
        clamp_weight(0.6),
        "in-range weight 0.6 must pass through unchanged"
    );
}

#[test]
fn c2_nan_weight_collapsed() {
    let frames = [frame(Viseme::E, f32::NAN)];
    let result = timed_frames(&frames, 40);
    let got = result[0].1.weight;
    assert!(
        got == 0.0,
        "NaN weight must collapse to rest weight 0.0 via clamp_weight, got {got}"
    );
}

// ---- C3 — empty input returns empty Vec ----

#[test]
fn c3_empty_input_returns_empty() {
    let result = timed_frames(&[], 40);
    assert!(result.is_empty(), "empty input must return an empty Vec");
}
