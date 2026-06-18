//! Frozen tests for T-02.10 — "Delete a fact".
//!
//! C1 — `zira_memory::FactStore::delete(&self, key: &str) -> Result<(), zira_memory::FactStoreError>`
//!       removes the entry for `key`; deleting an absent key is `Ok(())` (idempotent).
//! C2 — a test puts a fact, deletes it, and asserts a subsequent `get` of that key
//!       returns `Ok(None)`; a second test asserts deleting an absent key returns `Ok(())`.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> test_delete_present_returns_ok, test_delete_absent_returns_ok
//!   C2 -> test_delete_then_get_returns_none, test_delete_absent_is_idempotent

use zira_memory::{FactStore, FactStoreError};

// ---- C1 -----------------------------------------------------------------------

/// Verifies that `FactStore::delete` has the correct signature:
///   `delete(&self, key: &str) -> Result<(), FactStoreError>`
/// and that deleting an existing key returns `Ok(())`.
#[test]
fn test_delete_present_returns_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");
    store.put("key", "value").expect("put");

    let result: Result<(), FactStoreError> = store.delete("key");
    assert!(
        result.is_ok(),
        "delete of a present key must return Ok(()); got: {:?}",
        result.err()
    );
}

/// Verifies that deleting an absent key returns `Ok(())` — the operation is
/// idempotent and a missing key is never an error variant.
#[test]
fn test_delete_absent_returns_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");

    let result: Result<(), FactStoreError> = store.delete("no-such-key");
    assert!(
        result.is_ok(),
        "delete of an absent key must return Ok(()); got: {:?}",
        result.err()
    );
}

// ---- C2 -----------------------------------------------------------------------

/// Puts a fact, deletes it, then asserts that a subsequent `get` of that key
/// returns `Ok(None)` — the entry is gone after a successful delete.
#[test]
fn test_delete_then_get_returns_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");
    store.put("alpha", "one").expect("put alpha -> one");

    store.delete("alpha").expect("delete alpha must return Ok");

    let result: Result<Option<String>, FactStoreError> = store.get("alpha");
    assert_eq!(
        result.expect("get after delete must return Ok"),
        None,
        "get of a deleted key must return Ok(None)"
    );
}

/// Asserts that deleting an absent key returns `Ok(())` — a second delete of a
/// key that was never inserted (or was already deleted) is a no-op success.
#[test]
fn test_delete_absent_is_idempotent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");

    // First call on a key that never existed.
    let first: Result<(), FactStoreError> = store.delete("ghost");
    assert!(
        first.is_ok(),
        "first delete of absent key must return Ok(()); got: {:?}",
        first.err()
    );

    // Second call — still absent — must also return Ok.
    let second: Result<(), FactStoreError> = store.delete("ghost");
    assert!(
        second.is_ok(),
        "second delete of absent key must return Ok(()); got: {:?}",
        second.err()
    );
}
