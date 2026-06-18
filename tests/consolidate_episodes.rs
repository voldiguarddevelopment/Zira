//! Frozen tests for T-02.18 — "Consolidate the episodes".
//!
//! C1 — `zira_memory::consolidate(episode_path: &std::path::Path, store: &zira_memory::FactStore)
//!        -> Result<usize, zira_memory::FactStoreError>`
//!       runs a stateless pass that derives deduplicated facts from the episodes,
//!       `put`s each into `store`, and returns the count of facts written.
//! C2 — Episodes containing a duplicated piece of information collapse to a single fact;
//!       the returned count and a follow-up `get` confirm dedup.
//! C3 — `consolidate` over an empty (or missing) episode file writes zero facts and
//!       returns `Ok(0)`.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_consolidate_signature_and_count
//!   C2 -> c2_consolidate_deduplicates_repeated_text
//!   C3 -> c3_consolidate_empty_file_returns_zero, c3_consolidate_missing_file_returns_zero

use zira_memory::{append_episode, consolidate, Episode, FactStore, FactStoreError};

fn make_episode(role: &str, text: &str, ts: u64) -> Episode {
    Episode {
        role: role.to_string(),
        text: text.to_string(),
        timestamp: ts,
    }
}

// ---- C1 -----------------------------------------------------------------------

/// C1 — verifies that `consolidate` has the correct signature:
///   `consolidate(episode_path: &Path, store: &FactStore) -> Result<usize, FactStoreError>`
/// and that a single unique episode yields a returned count of 1.
#[test]
fn c1_consolidate_signature_and_count() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ep_path = dir.path().join("episodes.jsonl");
    let db_path = dir.path().join("facts.redb");

    let ep = make_episode("user", "the sky is blue", 1_000);
    append_episode(&ep_path, &ep).expect("append episode");

    let store = FactStore::open(&db_path).expect("open fact store");
    let result: Result<usize, FactStoreError> = consolidate(&ep_path, &store);

    assert!(
        result.is_ok(),
        "consolidate must return Ok for a valid episode file; got: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        1,
        "one unique episode must yield count 1"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// C2 — Two episodes with the same text (duplicated information) must collapse to
/// a single fact.  The returned count is 1 and a follow-up `get` on the episode
/// text key confirms the fact is present in the store.
#[test]
fn c2_consolidate_deduplicates_repeated_text() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ep_path = dir.path().join("episodes.jsonl");
    let db_path = dir.path().join("facts.redb");

    let text = "cats are mammals";
    let ep1 = make_episode("user", text, 1_000);
    let ep2 = make_episode("assistant", text, 2_000);
    append_episode(&ep_path, &ep1).expect("append ep1");
    append_episode(&ep_path, &ep2).expect("append ep2");

    let store = FactStore::open(&db_path).expect("open fact store");
    let count = consolidate(&ep_path, &store).expect("consolidate must succeed");

    // Duplicate text collapses to exactly one fact.
    assert_eq!(
        count, 1,
        "two episodes with the same text must collapse to one fact; got count {}",
        count
    );

    // A follow-up get confirms the fact is present under the episode text key.
    let got: Option<String> = store.get(text).expect("get must not error");
    assert!(
        got.is_some(),
        "store.get(text) after consolidate must return Some(_); the fact key \
         must be the episode text; got None"
    );
}

// ---- C3 -----------------------------------------------------------------------

/// C3 — `consolidate` over an explicitly empty episode file writes zero facts
/// and returns `Ok(0)`.
#[test]
fn c3_consolidate_empty_file_returns_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ep_path = dir.path().join("episodes.jsonl");
    let db_path = dir.path().join("facts.redb");

    std::fs::write(&ep_path, b"").expect("create empty episode file");

    let store = FactStore::open(&db_path).expect("open fact store");
    let result = consolidate(&ep_path, &store);

    assert!(
        result.is_ok(),
        "consolidate on an empty file must return Ok; got: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        0,
        "empty episode file must yield count 0"
    );
}

/// C3 — `consolidate` over a missing episode file (not-found path) writes zero
/// facts and returns `Ok(0)`, consistent with `load_episodes` which treats a
/// missing file as an empty log.
#[test]
fn c3_consolidate_missing_file_returns_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ep_path = dir.path().join("does_not_exist.jsonl");
    let db_path = dir.path().join("facts.redb");

    let store = FactStore::open(&db_path).expect("open fact store");
    let result = consolidate(&ep_path, &store);

    assert!(
        result.is_ok(),
        "consolidate on a missing file must return Ok; got: {:?}",
        result.err()
    );
    assert_eq!(
        result.unwrap(),
        0,
        "missing episode file must yield count 0"
    );
}
