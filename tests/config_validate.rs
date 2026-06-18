//! Frozen tests for T-00.13 — "Validate the config".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_validate_returns_result_unit_config_error,
//!          c1_error_names_offending_field_and_reason
//!   C2 -> c2_zero_sample_rate_is_invalid_sample_rate,
//!          c2_empty_binary_path_is_empty_path,
//!          c2_threshold_above_range_is_out_of_range,
//!          c2_threshold_below_range_is_out_of_range,
//!          c2_three_invalid_fields_yield_distinct_variants
//!   C3 -> c3_default_config_validates_ok,
//!          c3_valid_custom_config_validates_ok,
//!          c3_zero_sample_rate_is_invalid_sample_rate (shared with C2),
//!          c3_empty_binary_path_is_empty_path (shared with C2),
//!          c3_threshold_above_range_is_out_of_range (shared with C2)
//!
//! Design note: every invalid fixture is built by taking a *default* `ZiraConfig`
//! (which must validate `Ok` per C3) and corrupting exactly ONE field. A test that
//! breaks field X and then observes the error variant for X therefore proves that
//! `validate()` actually inspects field X — and nothing else is to blame.

use std::mem::discriminant;

use zira_config::{ConfigError, ZiraConfig};

/// A `ZiraConfig` with every validated field set to a concrete, *valid* value.
///
/// Built from `ZiraConfig::default()` and then overridden so the test does not depend
/// on the precise default values — only on validity. Used to prove that `validate()`
/// accepts arbitrary in-range input, not merely the default singleton.
fn valid_config() -> ZiraConfig {
    let mut cfg = ZiraConfig::default();
    cfg.model.model_id = "claude-3-opus".to_string();
    cfg.model.binary_path = "/usr/bin/claude".to_string();
    cfg.wakeword.threshold = 0.5;
    cfg.vad.threshold = 0.5;
    cfg.tts.sample_rate = 22_050;
    cfg
}

// ---- C1: typed Result<(), ConfigError> naming the offending field --------------------

#[test]
fn c1_validate_returns_result_unit_config_error() {
    // The exact signature is part of the contract: `validate(&self) -> Result<(), ConfigError>`.
    // A type-annotated binding fails to compile if the return type differs, and a valid
    // config must produce `Ok(())`.
    let cfg = valid_config();
    let result: Result<(), ConfigError> = cfg.validate();
    assert!(
        result.is_ok(),
        "a fully-valid config must validate Ok; got: {result:?}"
    );
}

#[test]
fn c1_error_names_offending_field_and_reason() {
    // The error's Display must name the offending field so a human can find it. We break
    // the sample rate and require the message to mention `sample_rate`.
    let mut cfg = valid_config();
    cfg.tts.sample_rate = 0;
    let err = cfg
        .validate()
        .expect_err("a zero sample rate must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("sample_rate"),
        "ConfigError must name the offending field `sample_rate`; got: {msg:?}"
    );
    assert!(
        msg.len() > "sample_rate".len(),
        "ConfigError must give a reason in addition to the field name; got: {msg:?}"
    );
}

// ---- C2: distinct typed variants for each invalid field ------------------------------

#[test]
fn c2_zero_sample_rate_is_invalid_sample_rate() {
    // u32 cannot be negative, so "non-positive" means zero: a sample rate of 0 is invalid.
    let mut cfg = valid_config();
    cfg.tts.sample_rate = 0;
    match cfg.validate() {
        Err(ConfigError::InvalidSampleRate { .. }) => {}
        other => panic!(
            "a zero sample rate must yield ConfigError::InvalidSampleRate; got: {other:?}"
        ),
    }
}

#[test]
fn c2_empty_binary_path_is_empty_path() {
    // The model binary path is required (it is the brain). Empty must be rejected.
    let mut cfg = valid_config();
    cfg.model.binary_path = String::new();
    match cfg.validate() {
        Err(ConfigError::EmptyPath { .. }) => {}
        other => panic!(
            "an empty required binary path must yield ConfigError::EmptyPath; got: {other:?}"
        ),
    }
}

#[test]
fn c2_threshold_above_range_is_out_of_range() {
    // A threshold lives in [0.0, 1.0]; 1.5 is above the range and must be rejected.
    let mut cfg = valid_config();
    cfg.wakeword.threshold = 1.5;
    match cfg.validate() {
        Err(ConfigError::ThresholdOutOfRange { .. }) => {}
        other => panic!(
            "a threshold above 1.0 must yield ConfigError::ThresholdOutOfRange; got: {other:?}"
        ),
    }
}

#[test]
fn c2_threshold_below_range_is_out_of_range() {
    // The lower bound matters too: a negative threshold is out of range. This kills a
    // mutant that only checks the upper bound.
    let mut cfg = valid_config();
    cfg.wakeword.threshold = -0.1;
    match cfg.validate() {
        Err(ConfigError::ThresholdOutOfRange { .. }) => {}
        other => panic!(
            "a threshold below 0.0 must yield ConfigError::ThresholdOutOfRange; got: {other:?}"
        ),
    }
}

#[test]
fn c2_three_invalid_fields_yield_distinct_variants() {
    // C2 requires the three rejection categories to be *distinct* variants. Compare their
    // discriminants pairwise — equal discriminants would mean a single catch-all variant.
    let mut bad_rate = valid_config();
    bad_rate.tts.sample_rate = 0;
    let rate_err = bad_rate
        .validate()
        .expect_err("zero sample rate must be rejected");

    let mut bad_path = valid_config();
    bad_path.model.binary_path = String::new();
    let path_err = bad_path
        .validate()
        .expect_err("empty binary path must be rejected");

    let mut bad_threshold = valid_config();
    bad_threshold.wakeword.threshold = 1.5;
    let threshold_err = bad_threshold
        .validate()
        .expect_err("out-of-range threshold must be rejected");

    let d_rate = discriminant(&rate_err);
    let d_path = discriminant(&path_err);
    let d_threshold = discriminant(&threshold_err);

    assert_ne!(
        d_rate, d_path,
        "sample-rate and empty-path errors must be distinct ConfigError variants"
    );
    assert_ne!(
        d_rate, d_threshold,
        "sample-rate and threshold errors must be distinct ConfigError variants"
    );
    assert_ne!(
        d_path, d_threshold,
        "empty-path and threshold errors must be distinct ConfigError variants"
    );
}

// ---- C3: default validates Ok; each fixture yields its specific error ----------------

#[test]
fn c3_default_config_validates_ok() {
    // The shipped default must be usable as-is: it validates without error.
    let cfg = ZiraConfig::default();
    assert!(
        cfg.validate().is_ok(),
        "ZiraConfig::default() must validate Ok; got: {:?}",
        cfg.validate()
    );
}

#[test]
fn c3_valid_custom_config_validates_ok() {
    // A non-default but in-range config must also validate — proving validate() checks
    // value validity, not equality with the default singleton.
    let cfg = valid_config();
    assert!(
        cfg.validate().is_ok(),
        "a valid custom config must validate Ok; got: {:?}",
        cfg.validate()
    );
}
