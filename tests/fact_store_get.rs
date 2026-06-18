//! Frozen tests for T-02.09 — "Get a fact".
//!
//! C1 — `zira_memory::FactStore::get(&self, key: &str) -> Result<Option<String>, zira_memory::FactStoreError>`
//!       returns `Ok(Some(value))` for a stored key and `Ok(None)` for an absent key
//!       (a miss is not an error variant).
//! C2 — a test puts `"a" -> "1"`, asserts `get("a")` returns `Ok(Some("1".into()))`
//!       and `get("absent")` returns `Ok(None)`.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> test_get_hit_returns_ok_some, test_get_miss_returns_ok_none
//!   C2 -> test_get_hit_and_miss

use zira_memory::{FactStore, FactStoreError};

// ---- C1 -----------------------------------------------------------------------

/// Verifies that `FactStore::get` has the correct signature:
///   `get(&self, key: &str) -> Result<Option<String>, FactStoreError>`
/// and that a stored key returns `Ok(Some(value))`.
#[test]
fn test_get_hit_returns_ok_some() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");
    store.put("hello", "world").expect("put");

    let result: Result<Option<String>, FactStoreError> = store.get("hello");
    assert!(
        result.is_ok(),
        "get of a stored key must return Ok; got: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        Some("world".to_owned()),
        "get of a stored key must return Ok(Some(value))"
    );
}

/// Verifies that `get` returns `Ok(None)` for a key that was never stored —
/// a miss is not an error variant.
#[test]
fn test_get_miss_returns_ok_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");

    let result: Result<Option<String>, FactStoreError> = store.get("no-such-key");
    assert!(
        result.is_ok(),
        "get of an absent key must return Ok (not an error); got: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        None,
        "get of an absent key must return Ok(None)"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// Puts `"a" -> "1"`, then asserts:
///   - `get("a")` returns `Ok(Some("1".to_owned()))`
///   - `get("absent")` returns `Ok(None)`
///
/// This is the combined hit-and-miss acceptance check from the task spec.
#[test]
fn test_get_hit_and_miss() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open");
    store.put("a", "1").expect("put a -> 1");

    let hit: Result<Option<String>, FactStoreError> = store.get("a");
    assert_eq!(
        hit.expect("get(\"a\") must return Ok"),
        Some("1".to_owned()),
        "get(\"a\") must return Ok(Some(\"1\".into()))"
    );

    let miss: Result<Option<String>, FactStoreError> = store.get("absent");
    assert_eq!(
        miss.expect("get(\"absent\") must return Ok"),
        None,
        "get(\"absent\") must return Ok(None)"
    );
}
