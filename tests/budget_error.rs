//! Frozen tests for T-05.09 — "Define the budget error".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_episodes_too_high_variant_exists_and_derives_debug
//!   C1 -> c1_episodes_zero_variant_exists_and_derives_debug
//!   C2 -> c2_display_episodes_too_high_contains_value
//!   C2 -> c2_display_episodes_zero_indicates_zero_condition

use zira_config::BudgetError;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_episodes_too_high_variant_exists_and_derives_debug() {
    let err = BudgetError::EpisodesTooHigh { value: 200, max: 100 };
    // Debug is derived — formats without panic.
    let debug = format!("{err:?}");
    assert!(
        debug.contains("EpisodesTooHigh"),
        "Debug output must name the variant; got: {debug:?}"
    );
    assert!(
        debug.contains("200"),
        "Debug output must include the over-limit value; got: {debug:?}"
    );
    assert!(
        debug.contains("100"),
        "Debug output must include the max value; got: {debug:?}"
    );
}

#[test]
fn c1_episodes_zero_variant_exists_and_derives_debug() {
    let err = BudgetError::EpisodesZero;
    let debug = format!("{err:?}");
    assert!(
        debug.contains("EpisodesZero"),
        "Debug output must name the variant; got: {debug:?}"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_display_episodes_too_high_contains_value() {
    let value = 999_usize;
    let err = BudgetError::EpisodesTooHigh { value, max: 500 };
    let msg = err.to_string();
    assert!(
        msg.contains(&value.to_string()),
        "Display must name the over-limit value ({value}); got: {msg:?}"
    );
}

#[test]
fn c2_display_episodes_zero_indicates_zero_condition() {
    let err = BudgetError::EpisodesZero;
    let msg = err.to_string();
    // The message must indicate the zero condition — "zero" or "0" must appear.
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("zero") || lower.contains(" 0 ") || msg.contains('0'),
        "Display must indicate the zero condition; got: {msg:?}"
    );
}
