//! Frozen tests for T-01.10 — "Type the bridge errors".
//!
//! Criterion coverage:
//!
//!   C1 -> test_bridge_error_implements_error_trait,
//!          test_bridge_error_implements_display,
//!          test_bridge_error_spawn_failed_variant_exists,
//!          test_bridge_error_non_zero_exit_variant_exists,
//!          test_bridge_error_missing_result_variant_exists
//!   C2 -> test_display_spawn_failed_non_empty_names_failure,
//!          test_display_non_zero_exit_non_empty_names_failure,
//!          test_display_missing_result_non_empty_names_failure

use zira_bridge::BridgeError;

// Compile-time proof that BridgeError: std::error::Error + std::fmt::Display.
fn assert_error_trait<E: std::error::Error + std::fmt::Display>(_: &E) {}

// ---- C1 — BridgeError enum with three distinct variants ----------------------------------

/// The SpawnFailed variant must be constructible with a spawn-cause description.
#[test]
fn test_bridge_error_spawn_failed_variant_exists() {
    let e = BridgeError::SpawnFailed("permission denied".to_string());
    // Just constructing the variant is the assertion — if this compiles it exists.
    let _ = e;
}

/// The NonZeroExit variant must be constructible with an i32 exit code.
#[test]
fn test_bridge_error_non_zero_exit_variant_exists() {
    let e = BridgeError::NonZeroExit(1);
    let _ = e;
}

/// The MissingResult variant must be constructible (unit variant).
#[test]
fn test_bridge_error_missing_result_variant_exists() {
    let e = BridgeError::MissingResult;
    let _ = e;
}

/// BridgeError must implement std::error::Error (verified at compile time via the
/// assert_error_trait bound).
#[test]
fn test_bridge_error_implements_error_trait() {
    assert_error_trait(&BridgeError::MissingResult);
    assert_error_trait(&BridgeError::NonZeroExit(0));
    assert_error_trait(&BridgeError::SpawnFailed(String::new()));
}

/// BridgeError must implement Display (covered by assert_error_trait above, but also
/// confirmed by calling to_string() at runtime).
#[test]
fn test_bridge_error_implements_display() {
    let _ = BridgeError::MissingResult.to_string();
    let _ = BridgeError::NonZeroExit(0).to_string();
    let _ = BridgeError::SpawnFailed(String::new()).to_string();
}

// ---- C2 — Display text is non-empty and names the failure --------------------------------

/// SpawnFailed's Display must be non-empty and contain a spawn-related keyword so a
/// reader knows what failed (mutation survivor prevention: each arm is distinctly asserted).
#[test]
fn test_display_spawn_failed_non_empty_names_failure() {
    let msg = BridgeError::SpawnFailed("no such file".to_string()).to_string();
    assert!(!msg.is_empty(), "SpawnFailed Display must not be empty");
    // The message must mention the failure kind. "spawn" is the natural word; "launch"
    // or "start" are acceptable synonyms. We pin on lowercase to avoid case sensitivity.
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("spawn") || lower.contains("launch") || lower.contains("start"),
        "SpawnFailed Display must name the spawn failure, got: {msg:?}"
    );
}

/// NonZeroExit's Display must be non-empty and embed the exit code so the caller knows
/// which code was returned.
#[test]
fn test_display_non_zero_exit_non_empty_names_failure() {
    let msg = BridgeError::NonZeroExit(42).to_string();
    assert!(!msg.is_empty(), "NonZeroExit Display must not be empty");
    // The exit code must appear in the message so the variant is distinguishable from
    // MissingResult and SpawnFailed under mutation.
    assert!(
        msg.contains("42"),
        "NonZeroExit Display must include the exit code, got: {msg:?}"
    );
}

/// MissingResult's Display must be non-empty and reference the missing terminal result
/// event so the failure is self-describing.
#[test]
fn test_display_missing_result_non_empty_names_failure() {
    let msg = BridgeError::MissingResult.to_string();
    assert!(!msg.is_empty(), "MissingResult Display must not be empty");
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("result") || lower.contains("terminal") || lower.contains("missing"),
        "MissingResult Display must name the missing-result failure, got: {msg:?}"
    );
}

/// All three Display messages must be distinct — a mutation that returns one constant
/// string for every variant would otherwise satisfy the per-variant non-empty checks.
#[test]
fn test_display_all_variants_produce_distinct_messages() {
    let spawn_msg = BridgeError::SpawnFailed("x".to_string()).to_string();
    let exit_msg = BridgeError::NonZeroExit(7).to_string();
    let missing_msg = BridgeError::MissingResult.to_string();
    assert_ne!(spawn_msg, exit_msg, "SpawnFailed and NonZeroExit must have distinct Display");
    assert_ne!(spawn_msg, missing_msg, "SpawnFailed and MissingResult must have distinct Display");
    assert_ne!(exit_msg, missing_msg, "NonZeroExit and MissingResult must have distinct Display");
}
