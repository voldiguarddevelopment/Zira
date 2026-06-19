//! T-05.13 — barge-in energy cue + default threshold (the gateable half; on-hardware
//! tuning of the threshold value is device-bound).
//!   C1 -> c1_energy_is_rms,           c1_empty_frame_is_zero
//!   C2 -> c2_should_barge_in_compares_to_threshold
//!   C3 -> c3_default_threshold_is_sane, c3_silence_quiet_loud

use zira_core::{barge_in_energy, should_barge_in, DEFAULT_BARGE_IN_THRESHOLD};

#[test]
fn c1_energy_is_rms() {
    // RMS of [0.5, -0.5, 0.5, -0.5] = sqrt(mean(0.25)) = 0.5.
    let e = barge_in_energy(&[0.5, -0.5, 0.5, -0.5]);
    assert!((e - 0.5).abs() < 1e-6, "rms was {e}");
}

#[test]
fn c1_empty_frame_is_zero() {
    assert_eq!(barge_in_energy(&[]), 0.0);
}

#[test]
fn c2_should_barge_in_compares_to_threshold() {
    let loud = [0.5f32; 160];
    let quiet = [0.001f32; 160];
    assert!(should_barge_in(&loud, 0.1));
    assert!(!should_barge_in(&quiet, 0.1));
    // Exactly-at-threshold does not trigger (strict `>`).
    let half = [0.1f32; 160];
    assert!(!should_barge_in(&half, barge_in_energy(&half)));
}

#[test]
fn c3_default_threshold_is_sane() {
    assert!((0.005..=0.5).contains(&DEFAULT_BARGE_IN_THRESHOLD));
}

#[test]
fn c3_silence_quiet_loud() {
    let silence = [0.0f32; 160];
    let loud = [0.5f32; 160];
    assert!(!should_barge_in(&silence, DEFAULT_BARGE_IN_THRESHOLD));
    assert!(should_barge_in(&loud, DEFAULT_BARGE_IN_THRESHOLD));
}
