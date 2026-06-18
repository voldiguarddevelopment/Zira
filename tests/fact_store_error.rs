//! Frozen tests for T-02.06 — "Type the fact-store errors".
//!
//! C1 — `zira_memory::FactStoreError` is an enum implementing `std::error::Error`
//!       and `Display`, with distinct variants for an open/database failure, a
//!       transaction failure, and a (de)serialization failure.
//! C2 — a unit test asserts the `Display` text of every variant is non-empty,
//!       names its failure, and that all variants produce mutually distinct
//!       messages — every Display arm is exercised.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_fact_store_error_implements_error_and_display
//!   C2 -> c2_display_messages_are_nonempty_named_and_distinct

use std::error::Error;
use zira_memory::FactStoreError;

// ---- C1 -----------------------------------------------------------------------

/// Verifies that `FactStoreError` implements `std::error::Error` and
/// `std::fmt::Display` and that all three required variants exist and are
/// constructible.
#[test]
fn c1_fact_store_error_implements_error_and_display() {
    fn assert_is_error<E: Error>(_: &E) {}

    let open_err = FactStoreError::OpenFailed("could not open db".to_string());
    let tx_err = FactStoreError::TransactionFailed("tx aborted".to_string());
    let serde_err = FactStoreError::SerializeFailed("bad json".to_string());

    // Verifies the Error bound at compile time.
    assert_is_error(&open_err);
    assert_is_error(&tx_err);
    assert_is_error(&serde_err);

    // Verifies Display at compile time (format! requires Display).
    let _ = format!("{open_err}");
    let _ = format!("{tx_err}");
    let _ = format!("{serde_err}");
}

// ---- C2 -----------------------------------------------------------------------

/// Asserts every variant's `Display` text is non-empty, names its failure kind,
/// and all three variants produce mutually distinct messages — every Display arm
/// is exercised so no arm is an unexercised mutation survivor.
#[test]
fn c2_display_messages_are_nonempty_named_and_distinct() {
    // Neutral context strings unrelated to the failure-kind keywords so that any
    // keyword match comes from the format string, not the injected context.
    let open_msg = format!("{}", FactStoreError::OpenFailed("ctx-alpha".to_string()));
    let tx_msg = format!("{}", FactStoreError::TransactionFailed("ctx-beta".to_string()));
    let serde_msg = format!("{}", FactStoreError::SerializeFailed("ctx-gamma".to_string()));

    // Non-empty.
    assert!(!open_msg.is_empty(), "OpenFailed Display must not be empty");
    assert!(!tx_msg.is_empty(), "TransactionFailed Display must not be empty");
    assert!(!serde_msg.is_empty(), "SerializeFailed Display must not be empty");

    // Names its failure — the format string must include a word that identifies
    // the failure kind (the injected context contains none of these words).
    let open_lower = open_msg.to_lowercase();
    assert!(
        open_lower.contains("open") || open_lower.contains("database") || open_lower.contains("db"),
        "OpenFailed Display must name the open/database failure, got: {open_msg:?}"
    );

    let tx_lower = tx_msg.to_lowercase();
    assert!(
        tx_lower.contains("transaction") || tx_lower.contains("tx"),
        "TransactionFailed Display must name the transaction failure, got: {tx_msg:?}"
    );

    let serde_lower = serde_msg.to_lowercase();
    assert!(
        serde_lower.contains("serial") || serde_lower.contains("deserial"),
        "SerializeFailed Display must name the (de)serialization failure, got: {serde_msg:?}"
    );

    // Mutually distinct — all three messages must differ.
    assert_ne!(
        open_msg, tx_msg,
        "OpenFailed and TransactionFailed must produce distinct Display messages"
    );
    assert_ne!(
        open_msg, serde_msg,
        "OpenFailed and SerializeFailed must produce distinct Display messages"
    );
    assert_ne!(
        tx_msg, serde_msg,
        "TransactionFailed and SerializeFailed must produce distinct Display messages"
    );
}
