//! Frozen tests for T-02.08 — "Put a fact".
//!
//! C1 — `zira_memory::FactStore::put(&self, key: &str, value: &str) -> Result<(), zira_memory::FactStoreError>`
//!       commits a `key -> value` entry to the redb store durably.
//! C2 — a test puts a fact, opens a fresh `FactStore` over the same path, and
//!       (via the underlying redb table read used by the get primitive) confirms
//!       the committed value is present after the write transaction returns `Ok`.
//!
//! The internal table name "facts" is frozen here; the implementation must match.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_put_returns_ok, c1_put_overwrite_returns_ok
//!   C2 -> c2_put_persists_across_reopen

use zira_memory::{FactStore, FactStoreError};

// The redb table definition frozen as part of this test — key: &str, value: &str,
// table name "facts".  The implementation must use this exact definition.
const FACTS_TABLE: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("facts");

// ---- C1 -----------------------------------------------------------------------

/// Verifies that `FactStore::put` has the correct signature:
///   `put(&self, key: &str, value: &str) -> Result<(), FactStoreError>`
/// and that inserting a new key returns `Ok(())`.
#[test]
fn c1_put_returns_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");
    let result: Result<(), FactStoreError> = store.put("key", "value");
    assert!(
        result.is_ok(),
        "put of a new key must return Ok(()); got: {:?}",
        result.err()
    );
}

/// Verifies that putting the same key a second time (overwrite) also returns
/// `Ok(())` — a duplicate put is not an error.
#[test]
fn c1_put_overwrite_returns_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");
    store.put("key", "first").expect("first put");
    let result: Result<(), FactStoreError> = store.put("key", "second");
    assert!(
        result.is_ok(),
        "put of an existing key (overwrite) must return Ok(()); got: {:?}",
        result.err()
    );
}

// ---- C2 -----------------------------------------------------------------------

/// Puts a fact, drops the FactStore handle, opens a fresh `FactStore` over the
/// same path (confirming on-disk persistence), then reads back via the underlying
/// redb table to assert the committed value is present.
///
/// This exercises the invariant: a put that returns `Ok` is persisted past the
/// transaction boundary and survives a full close/reopen cycle.
#[test]
fn c2_put_persists_across_reopen() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    // Write the fact and commit.
    {
        let store = FactStore::open(&db_path).expect("open");
        store.put("greeting", "hello").expect("put must return Ok");
    }
    // Handle dropped: transaction committed, database closed.

    // Open a fresh FactStore over the same path — confirms on-disk persistence.
    {
        let fresh = FactStore::open(&db_path).expect("reopen after put");
        drop(fresh);
    }

    // Read back via the underlying redb table used by the get primitive — confirms
    // the committed value is present after the write transaction returned Ok.
    let db = redb::Database::create(&db_path).expect("open redb for verification");
    let read_txn = db.begin_read().expect("begin read transaction");
    let table = read_txn
        .open_table(FACTS_TABLE)
        .expect("open facts table for verification");
    let stored = table
        .get("greeting")
        .expect("read key from facts table")
        .map(|guard| guard.value().to_owned());
    assert_eq!(
        stored.as_deref(),
        Some("hello"),
        "put value must survive commit and full close/reopen; \
         expected Some(\"hello\"), got {:?}",
        stored
    );
}
