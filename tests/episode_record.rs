//! Frozen tests for T-02.02 — "Define the episode record".
//!
//! These tests verify:
//!   C1 — `zira_memory::Episode` is a public struct with `role: String`,
//!         `text: String`, and `timestamp: u64` fields, and derives
//!         `Serialize`, `Deserialize`, `Clone`, `PartialEq`, and `Debug`.
//!   C2 — an `Episode` survives a `serde_json::to_string` → `serde_json::from_str`
//!         round-trip unchanged (equality via `PartialEq`).
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_episode_has_required_fields,
//!         c1_episode_derives_clone,
//!         c1_episode_derives_partial_eq,
//!         c1_episode_derives_debug,
//!         c1_empty_string_fields_are_valid
//!   C2 -> c2_episode_serde_json_round_trip,
//!         c2_episode_empty_strings_round_trip

use zira_memory::Episode;

// ---- C1 -------------------------------------------------------------------------------

/// Constructing an `Episode` with the three required fields must compile.
/// This is the primary shape-existence check for C1.
#[test]
fn c1_episode_has_required_fields() {
    let ep = Episode {
        role: "user".to_string(),
        text: "hello".to_string(),
        timestamp: 1_000_000,
    };
    assert_eq!(ep.role, "user");
    assert_eq!(ep.text, "hello");
    assert_eq!(ep.timestamp, 1_000_000u64);
}

/// `Clone` must be derived: a clone equals the original via `PartialEq`.
#[test]
fn c1_episode_derives_clone() {
    let ep = Episode {
        role: "assistant".to_string(),
        text: "world".to_string(),
        timestamp: 42,
    };
    let cloned = ep.clone();
    assert_eq!(ep, cloned);
}

/// `PartialEq` must be derived: two independently-constructed identical episodes
/// must compare equal, and differing episodes must not.
#[test]
fn c1_episode_derives_partial_eq() {
    let a = Episode {
        role: "user".to_string(),
        text: "same".to_string(),
        timestamp: 1,
    };
    let b = Episode {
        role: "user".to_string(),
        text: "same".to_string(),
        timestamp: 1,
    };
    let c = Episode {
        role: "user".to_string(),
        text: "different".to_string(),
        timestamp: 1,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

/// `Debug` must be derived: formatting with `{:?}` must not panic.
#[test]
fn c1_episode_derives_debug() {
    let ep = Episode {
        role: "system".to_string(),
        text: "init".to_string(),
        timestamp: 0,
    };
    let debug_str = format!("{ep:?}");
    assert!(!debug_str.is_empty());
}

/// Empty-string fields are valid episodes (edge case from task notes).
#[test]
fn c1_empty_string_fields_are_valid() {
    let ep = Episode {
        role: String::new(),
        text: String::new(),
        timestamp: 0,
    };
    assert_eq!(ep.role, "");
    assert_eq!(ep.text, "");
    assert_eq!(ep.timestamp, 0u64);
}

// ---- C2 -------------------------------------------------------------------------------

/// Full serde_json round-trip: serialize → deserialize must yield the original value.
#[test]
fn c2_episode_serde_json_round_trip() {
    let original = Episode {
        role: "user".to_string(),
        text: "round-trip me".to_string(),
        timestamp: 1_718_000_000,
    };
    let json = serde_json::to_string(&original).expect("serialize Episode");
    let recovered: Episode = serde_json::from_str(&json).expect("deserialize Episode");
    assert_eq!(original, recovered);
}

/// Round-trip with empty-string fields (the edge case from task notes).
#[test]
fn c2_episode_empty_strings_round_trip() {
    let original = Episode {
        role: String::new(),
        text: String::new(),
        timestamp: 0,
    };
    let json = serde_json::to_string(&original).expect("serialize Episode with empty fields");
    let recovered: Episode = serde_json::from_str(&json).expect("deserialize Episode with empty fields");
    assert_eq!(original, recovered);
}
