//! Frozen tests for T-02.03 — "Append one episode".
//!
//! C1 — `zira_memory::append_episode` serializes `episode` to one JSON line and
//!       appends it (with a trailing newline) to the file at `path`, creating the
//!       file if absent.
//! C2 — appending two episodes and reading the raw file back yields exactly two
//!       newline-terminated lines each parsing as the corresponding `Episode`.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_append_creates_file_when_absent, c1_append_writes_valid_json_line
//!   C2 -> c2_append_two_episodes_exact_lines

use zira_memory::{append_episode, Episode};

fn make_episode(role: &str, text: &str, ts: u64) -> Episode {
    Episode {
        role: role.to_string(),
        text: text.to_string(),
        timestamp: ts,
    }
}

// ---- C1 -----------------------------------------------------------------------

/// Appending to a non-existent path must succeed (creating the file).
/// The resulting file must contain exactly one line that parses back as the episode.
#[test]
fn c1_append_creates_file_when_absent() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    assert!(!path.exists(), "file must not exist before append");

    let ep = make_episode("user", "hello world", 1_000_000);
    append_episode(&path, &ep).expect("append_episode must not return an error");

    assert!(path.exists(), "append_episode must create the file if absent");

    let content = std::fs::read_to_string(&path).expect("read back the file");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1, "exactly one line after one append");

    let recovered: Episode =
        serde_json::from_str(lines[0]).expect("line must parse as Episode");
    assert_eq!(recovered, ep);
}

/// After one append the file's content is a single JSON-serialized Episode
/// followed by exactly one trailing newline byte.
#[test]
fn c1_append_writes_valid_json_line() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep = make_episode("assistant", "response text", 9_999);
    append_episode(&path, &ep).expect("append_episode must succeed");

    let raw = std::fs::read(&path).expect("read raw bytes");
    assert!(raw.ends_with(b"\n"), "file must end with a trailing newline");

    let content = String::from_utf8(raw).expect("valid UTF-8");
    assert_eq!(
        content.chars().filter(|&c| c == '\n').count(),
        1,
        "exactly one newline in the file after one append"
    );

    let line = content.trim_end_matches('\n');
    let recovered: Episode =
        serde_json::from_str(line).expect("line parses as Episode");
    assert_eq!(recovered, ep);
}

// ---- C2 -----------------------------------------------------------------------

/// Append two distinct episodes, read the raw file, and assert exactly two
/// newline-terminated lines each parsing as the corresponding Episode in order.
#[test]
fn c2_append_two_episodes_exact_lines() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("episodes.jsonl");

    let ep1 = make_episode("user", "first message", 1_000);
    let ep2 = make_episode("assistant", "second message", 2_000);

    append_episode(&path, &ep1).expect("first append must succeed");
    append_episode(&path, &ep2).expect("second append must succeed");

    let content = std::fs::read_to_string(&path).expect("read back file");

    // Two trailing-newline-terminated lines: split('\n') yields ["line1", "line2", ""].
    let parts: Vec<&str> = content.split('\n').collect();
    assert_eq!(
        parts.len(),
        3,
        "two newline-terminated lines split into three parts (including trailing empty)"
    );
    assert!(parts[2].is_empty(), "last split part must be empty (trailing newline)");

    let recovered1: Episode =
        serde_json::from_str(parts[0]).expect("first line parses as Episode");
    let recovered2: Episode =
        serde_json::from_str(parts[1]).expect("second line parses as Episode");

    assert_eq!(recovered1, ep1, "first line must equal first episode");
    assert_eq!(recovered2, ep2, "second line must equal second episode");
}
