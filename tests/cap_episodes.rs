//! Frozen tests for T-02.05 — "Enforce the episode cap".
//!
//! C1 — `zira_memory::cap_episodes(path: &std::path::Path, max_episodes: usize)
//!       -> std::io::Result<()>` rewrites the JSONL file to retain only the most
//!       recent `max_episodes` episodes, preserving their order; if the file already
//!       holds `<= max_episodes` it is left unchanged.
//! C2 — a test appends five episodes, calls `cap_episodes(path, 3)`, then
//!       `load_episodes`, and asserts exactly the last three episodes remain in order.
//! C3 — a test with `max_episodes` of 0 leaves the file empty (zero episodes) and
//!       returns `Ok(())`.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_under_cap_file_is_unchanged, c2_cap_five_to_three_retains_last_three
//!   C2 -> c2_cap_five_to_three_retains_last_three
//!   C3 -> c3_cap_zero_empties_the_file

use zira_memory::{append_episode, cap_episodes, load_episodes, Episode};

fn ep(role: &str, text: &str, ts: u64) -> Episode {
    Episode {
        role: role.to_string(),
        text: text.to_string(),
        timestamp: ts,
    }
}

// ---- C1 -----------------------------------------------------------------------

/// A file holding fewer episodes than `max_episodes` must be left unchanged after
/// a cap call (no truncation, same content as before).
#[test]
fn c1_under_cap_file_is_unchanged() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep1 = ep("user", "alpha", 1);
    let ep2 = ep("assistant", "beta", 2);

    append_episode(&path, &ep1).expect("append ep1");
    append_episode(&path, &ep2).expect("append ep2");

    // Two episodes, cap of 5 — file must be untouched.
    cap_episodes(&path, 5).expect("cap_episodes must return Ok");

    let loaded = load_episodes(&path).expect("load_episodes must succeed");
    assert_eq!(
        loaded.len(),
        2,
        "file with two episodes capped at 5 must still have two episodes"
    );
    assert_eq!(loaded[0], ep1, "first episode must be unchanged");
    assert_eq!(loaded[1], ep2, "second episode must be unchanged");
}

// ---- C2 -----------------------------------------------------------------------

/// Append five episodes, cap to three; only the three most-recent must survive,
/// in their original append order (oldest of the survivors first).
#[test]
fn c2_cap_five_to_three_retains_last_three() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep1 = ep("user", "one", 1);
    let ep2 = ep("assistant", "two", 2);
    let ep3 = ep("user", "three", 3);
    let ep4 = ep("assistant", "four", 4);
    let ep5 = ep("user", "five", 5);

    append_episode(&path, &ep1).expect("append ep1");
    append_episode(&path, &ep2).expect("append ep2");
    append_episode(&path, &ep3).expect("append ep3");
    append_episode(&path, &ep4).expect("append ep4");
    append_episode(&path, &ep5).expect("append ep5");

    cap_episodes(&path, 3).expect("cap_episodes must return Ok");

    let loaded = load_episodes(&path).expect("load_episodes must succeed after cap");
    assert_eq!(
        loaded.len(),
        3,
        "after capping five to three, exactly three episodes must remain, got {}",
        loaded.len()
    );
    assert_eq!(loaded[0], ep3, "first retained episode must be the third appended");
    assert_eq!(loaded[1], ep4, "second retained episode must be the fourth appended");
    assert_eq!(loaded[2], ep5, "third retained episode must be the fifth appended");
}

// ---- C3 -----------------------------------------------------------------------

/// A cap of zero must empty the file entirely and return `Ok(())`.
#[test]
fn c3_cap_zero_empties_the_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep1 = ep("user", "hello", 10);
    let ep2 = ep("assistant", "world", 20);

    append_episode(&path, &ep1).expect("append ep1");
    append_episode(&path, &ep2).expect("append ep2");

    let result = cap_episodes(&path, 0);
    assert!(
        result.is_ok(),
        "cap_episodes with max_episodes=0 must return Ok, got {:?}",
        result.unwrap_err()
    );

    let loaded = load_episodes(&path).expect("load_episodes must succeed after cap to zero");
    assert!(
        loaded.is_empty(),
        "after capping to zero, no episodes must remain, got {} episodes",
        loaded.len()
    );
}
