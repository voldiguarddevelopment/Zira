//! Frozen tests for T-02.04 — "Load the episodes".
//!
//! C1 — `zira_memory::load_episodes(path: &std::path::Path) -> std::io::Result<Vec<zira_memory::Episode>>`
//!       reads the JSONL file and returns its episodes in file order; a non-existent
//!       path returns `Ok(vec![])` rather than an error.
//! C2 — a test writes three episodes via `append_episode`, calls `load_episodes`, and
//!       asserts the returned vec equals the three originals in append order.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_missing_file_returns_empty_vec, c1_load_episodes_reads_file_in_order
//!   C2 -> c2_append_then_load_three_episodes

use zira_memory::{append_episode, load_episodes, Episode};

fn ep(role: &str, text: &str, ts: u64) -> Episode {
    Episode {
        role: role.to_string(),
        text: text.to_string(),
        timestamp: ts,
    }
}

// ---- C1 -----------------------------------------------------------------------

/// A path that does not exist must yield `Ok(vec![])`, not an error.
#[test]
fn c1_missing_file_returns_empty_vec() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("no_such_file.jsonl");

    assert!(!path.exists(), "path must not exist before the call");

    let result = load_episodes(&path).expect("load_episodes on a missing path must return Ok");
    assert!(
        result.is_empty(),
        "load_episodes on a missing path must return an empty vec, got {:?}",
        result
    );
}

/// A file containing valid JSONL episodes must be read back in file order.
/// Written manually (not via append_episode) to isolate load from append.
#[test]
fn c1_load_episodes_reads_file_in_order() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep1 = ep("user", "alpha", 1);
    let ep2 = ep("assistant", "beta", 2);

    let line1 = serde_json::to_string(&ep1).expect("serialize ep1");
    let line2 = serde_json::to_string(&ep2).expect("serialize ep2");
    let content = format!("{}\n{}\n", line1, line2);
    std::fs::write(&path, content).expect("write jsonl file");

    let loaded = load_episodes(&path).expect("load_episodes must succeed on valid file");
    assert_eq!(loaded.len(), 2, "must return exactly two episodes");
    assert_eq!(loaded[0], ep1, "first episode must match file order");
    assert_eq!(loaded[1], ep2, "second episode must match file order");
}

// ---- C2 -----------------------------------------------------------------------

/// Append three episodes then load them; the returned vec must equal the originals
/// in append order.
#[test]
fn c2_append_then_load_three_episodes() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep1 = ep("user", "first", 100);
    let ep2 = ep("assistant", "second", 200);
    let ep3 = ep("user", "third", 300);

    append_episode(&path, &ep1).expect("append ep1");
    append_episode(&path, &ep2).expect("append ep2");
    append_episode(&path, &ep3).expect("append ep3");

    let loaded = load_episodes(&path).expect("load_episodes must succeed after appends");
    assert_eq!(
        loaded.len(),
        3,
        "must return exactly three episodes, got {}",
        loaded.len()
    );
    assert_eq!(loaded[0], ep1, "first loaded episode must equal first appended");
    assert_eq!(loaded[1], ep2, "second loaded episode must equal second appended");
    assert_eq!(loaded[2], ep3, "third loaded episode must equal third appended");
}
