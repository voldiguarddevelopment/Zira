//! Frozen tests for T-02.07 — "Open the fact store".
//!
//! C1 — `zira_memory::FactStore::open(path: &std::path::Path)`
//!       returns `Result<zira_memory::FactStore, zira_memory::FactStoreError>`,
//!       creating the database file when it is absent.
//! C2 — opening a fresh path returns `Ok`, and re-opening the same path also returns
//!       `Ok`, proving on-disk persistence across open handles.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_open_creates_database_when_absent
//!   C2 -> c2_reopen_same_path_returns_ok

use std::path::Path;
use zira_memory::{FactStore, FactStoreError};

// ---- C1 -----------------------------------------------------------------------

/// Verifies that `FactStore::open` compiles with the correct signature:
///   `open(path: &Path) -> Result<FactStore, FactStoreError>`
/// and that calling it on a path that does not yet exist returns `Ok` (it creates
/// the database file rather than erroring on absence).
#[test]
fn c1_open_creates_database_when_absent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    // The file must not pre-exist.
    assert!(
        !db_path.exists(),
        "precondition: db file must not exist before open"
    );

    let result: Result<FactStore, FactStoreError> = FactStore::open(&db_path);

    assert!(
        result.is_ok(),
        "FactStore::open on a non-existent path must create the database and return Ok; \
         got: {:?}",
        result.err()
    );

    // The file (or directory entry) must now exist on disk.
    assert!(
        db_path.exists(),
        "FactStore::open must create the database file at the given path"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// Opens a FactStore at a fresh temp-dir path, asserts `Ok`, then drops the handle
/// and re-opens the same path, asserting `Ok` again — proving the on-disk database
/// persists across independent open calls.
#[test]
fn c2_reopen_same_path_returns_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path: &Path = &dir.path().join("facts.redb");

    // First open — creates the database.
    let first = FactStore::open(db_path);
    assert!(
        first.is_ok(),
        "first open must return Ok; got: {:?}",
        first.err()
    );
    // Drop the handle to release any exclusive lock.
    drop(first);

    // Second open — the database already exists on disk.
    let second = FactStore::open(db_path);
    assert!(
        second.is_ok(),
        "second open of the same path must return Ok (database persists); \
         got: {:?}",
        second.err()
    );
}
