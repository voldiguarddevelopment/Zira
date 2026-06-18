//! Frozen tests for T-05.10 — "Audit the memory budget".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_ok_when_episodes_at_ceiling
//!   C1 -> c1_ok_when_episodes_below_ceiling
//!   C2 -> c2_err_too_high_when_episodes_exceed_ceiling
//!   C2 -> c2_err_too_high_carries_value_and_max
//!   C3 -> c3_err_zero_when_episodes_is_zero

use zira_config::{audit_memory_budget, BudgetError, MemoryConfig};

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_ok_when_episodes_at_ceiling() {
    let cfg = MemoryConfig { max_episodes: 100 };
    let result = audit_memory_budget(&cfg, 100);
    assert!(
        result.is_ok(),
        "expected Ok when max_episodes == ceiling (100 == 100); got: {result:?}"
    );
}

#[test]
fn c1_ok_when_episodes_below_ceiling() {
    let cfg = MemoryConfig { max_episodes: 50 };
    let result = audit_memory_budget(&cfg, 100);
    assert!(
        result.is_ok(),
        "expected Ok when max_episodes < ceiling (50 < 100); got: {result:?}"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_err_too_high_when_episodes_exceed_ceiling() {
    let cfg = MemoryConfig { max_episodes: 101 };
    let result = audit_memory_budget(&cfg, 100);
    assert!(
        matches!(result, Err(BudgetError::EpisodesTooHigh { .. })),
        "expected Err(EpisodesTooHigh) when max_episodes > ceiling (101 > 100); got: {result:?}"
    );
}

#[test]
fn c2_err_too_high_carries_value_and_max() {
    let cfg = MemoryConfig { max_episodes: 200 };
    let ceiling = 50_usize;
    let result = audit_memory_budget(&cfg, ceiling);
    match result {
        Err(BudgetError::EpisodesTooHigh { value, max }) => {
            assert_eq!(value, 200, "EpisodesTooHigh.value must be the configured max_episodes");
            assert_eq!(max, ceiling, "EpisodesTooHigh.max must be the supplied ceiling");
        }
        other => panic!("expected Err(EpisodesTooHigh {{ value: 200, max: 50 }}); got: {other:?}"),
    }
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_err_zero_when_episodes_is_zero() {
    let cfg = MemoryConfig { max_episodes: 0 };
    let result = audit_memory_budget(&cfg, 100);
    assert!(
        matches!(result, Err(BudgetError::EpisodesZero)),
        "expected Err(EpisodesZero) when max_episodes == 0; got: {result:?}"
    );
}
