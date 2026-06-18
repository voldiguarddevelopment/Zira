//! Frozen tests for T-05.08 — "Write the default config".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_creates_parent_dirs_and_round_trips_default,
//!          c1_file_exists_after_write,
//!          c1_loaded_config_equals_default
//!   C2 -> c2_second_call_succeeds,
//!          c2_second_call_leaves_config_unchanged

use std::path::PathBuf;

use zira_config::{load_from, write_default_config, ZiraConfig};

/// A unique path under the Cargo-managed temp dir for one test case.
/// The intermediate directory is deliberately NOT created — `write_default_config`
/// is responsible for creating all parent directories.
fn temp_config_path(label: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("write_default_config")
        .join(label)
        .join("config.toml")
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_creates_parent_dirs_and_round_trips_default() {
    let path = temp_config_path("c1_round_trip");
    // Precondition: neither the file nor its parent directory exists.
    let parent = path.parent().expect("path has a parent");
    let _ = std::fs::remove_dir_all(parent);
    assert!(
        !parent.exists(),
        "precondition: parent directory must not exist before write_default_config"
    );
    assert!(
        !path.exists(),
        "precondition: config file must not exist before write_default_config"
    );

    write_default_config(&path).expect("write_default_config must succeed over a fresh path");

    // C1 part 1: parent directories must have been created.
    assert!(
        parent.is_dir(),
        "write_default_config must create the parent directory"
    );

    // C1 part 2: the file must exist and load_from must succeed.
    assert!(
        path.exists(),
        "write_default_config must leave the config file on disk"
    );

    let loaded = load_from(&path).expect("load_from must succeed after write_default_config");
    assert_eq!(
        loaded,
        ZiraConfig::default(),
        "loaded config must equal ZiraConfig::default() after write_default_config"
    );
}

#[test]
fn c1_file_exists_after_write() {
    let path = temp_config_path("c1_file_exists");
    let parent = path.parent().expect("path has a parent");
    let _ = std::fs::remove_dir_all(parent);

    write_default_config(&path).expect("write_default_config must succeed");

    assert!(
        path.is_file(),
        "write_default_config must create a regular file at the given path"
    );
}

#[test]
fn c1_loaded_config_equals_default() {
    let path = temp_config_path("c1_loaded_equals_default");
    let parent = path.parent().expect("path has a parent");
    let _ = std::fs::remove_dir_all(parent);

    write_default_config(&path).expect("write_default_config must succeed");

    let loaded = load_from(&path).expect("load_from must succeed after write_default_config");
    let expected = ZiraConfig::default();

    // Field-by-field context to help diagnose a mismatch.
    assert_eq!(loaded.paths, expected.paths, "paths field mismatch");
    assert_eq!(loaded.model, expected.model, "model field mismatch");
    assert_eq!(loaded.wakeword, expected.wakeword, "wakeword field mismatch");
    assert_eq!(loaded.vad, expected.vad, "vad field mismatch");
    assert_eq!(loaded.stt, expected.stt, "stt field mismatch");
    assert_eq!(loaded.tts, expected.tts, "tts field mismatch");
    assert_eq!(loaded.emotion, expected.emotion, "emotion field mismatch");
    assert_eq!(loaded.memory, expected.memory, "memory field mismatch");
    assert_eq!(loaded.avatar, expected.avatar, "avatar field mismatch");
    assert_eq!(
        loaded, expected,
        "full ZiraConfig round-trip mismatch after write_default_config"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_second_call_succeeds() {
    let path = temp_config_path("c2_second_call");
    let parent = path.parent().expect("path has a parent");
    let _ = std::fs::remove_dir_all(parent);

    write_default_config(&path).expect("first write_default_config call must succeed");
    // Idempotency: a second call against an already-existing file must also succeed.
    write_default_config(&path).expect("second write_default_config call must succeed (idempotent)");
}

#[test]
fn c2_second_call_leaves_config_unchanged() {
    let path = temp_config_path("c2_unchanged");
    let parent = path.parent().expect("path has a parent");
    let _ = std::fs::remove_dir_all(parent);

    write_default_config(&path).expect("first write_default_config call must succeed");
    let after_first =
        load_from(&path).expect("load_from must succeed after first write_default_config");

    write_default_config(&path).expect("second write_default_config call must succeed");
    let after_second =
        load_from(&path).expect("load_from must succeed after second write_default_config");

    assert_eq!(
        after_first,
        ZiraConfig::default(),
        "config after first write must equal ZiraConfig::default()"
    );
    assert_eq!(
        after_second,
        ZiraConfig::default(),
        "config after second write must equal ZiraConfig::default()"
    );
    assert_eq!(
        after_first, after_second,
        "config must be unchanged between the first and second write_default_config calls"
    );
}
