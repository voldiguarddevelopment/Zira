//! Frozen tests for T-05.07 — "Detect the first run".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_absent_path_returns_true,
//!          c1_present_file_returns_false,
//!          c1_present_but_empty_file_returns_false
//!   C2 -> c2_detection_does_not_create_missing_path,
//!          c2_detection_does_not_modify_existing_file,
//!          c2_detection_does_not_delete_existing_file

use std::path::PathBuf;

use zira_config::is_first_run;

/// Return a path inside the Cargo-managed temp dir that is guaranteed not to exist.
fn absent_path(label: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("first_run_detect")
        .join(label)
        .join("config.toml")
}

/// Return a path to a newly-created file inside the Cargo-managed temp dir.
fn present_path(label: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("first_run_detect")
        .join(label);
    std::fs::create_dir_all(&base).expect("create temp subdir");
    let path = base.join("config.toml");
    std::fs::write(&path, b"").expect("write temp file");
    path
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_absent_path_returns_true() {
    let path = absent_path("c1_absent");
    assert!(
        !path.exists(),
        "precondition: path must not exist before the call"
    );
    assert!(
        is_first_run(&path),
        "is_first_run must return true when the config file does not exist"
    );
}

#[test]
fn c1_present_file_returns_false() {
    let path = present_path("c1_present");
    assert!(
        path.exists(),
        "precondition: file must exist before the call"
    );
    assert!(
        !is_first_run(&path),
        "is_first_run must return false when the config file exists"
    );
}

#[test]
fn c1_present_but_empty_file_returns_false() {
    // A present-but-empty file still counts as existing — not a first run.
    let path = present_path("c1_empty");
    let metadata = std::fs::metadata(&path).expect("read metadata");
    assert_eq!(
        metadata.len(),
        0,
        "precondition: file must be empty for this test"
    );
    assert!(
        !is_first_run(&path),
        "is_first_run must return false for an empty-but-present config file"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_detection_does_not_create_missing_path() {
    let path = absent_path("c2_no_create");
    assert!(
        !path.exists(),
        "precondition: path must not exist before the call"
    );

    let _ = is_first_run(&path);

    assert!(
        !path.exists(),
        "is_first_run must not create the config file when it is absent"
    );
    // Also assert neither the parent directory was created, since detection is read-only.
    let parent = path.parent().expect("path has a parent");
    assert!(
        !parent.exists(),
        "is_first_run must not create any parent directories either"
    );
}

#[test]
fn c2_detection_does_not_modify_existing_file() {
    let path = present_path("c2_no_modify");

    let meta_before = std::fs::metadata(&path).expect("read metadata before");
    let mtime_before = meta_before
        .modified()
        .expect("platform supports mtime");
    let size_before = meta_before.len();

    let _ = is_first_run(&path);

    let meta_after = std::fs::metadata(&path).expect("read metadata after");
    let mtime_after = meta_after
        .modified()
        .expect("platform supports mtime");
    let size_after = meta_after.len();

    assert_eq!(
        mtime_before, mtime_after,
        "is_first_run must not modify the file's mtime"
    );
    assert_eq!(
        size_before, size_after,
        "is_first_run must not change the file's size"
    );
}

#[test]
fn c2_detection_does_not_delete_existing_file() {
    let path = present_path("c2_no_delete");
    assert!(path.exists(), "precondition: file must exist before the call");

    let _ = is_first_run(&path);

    assert!(
        path.exists(),
        "is_first_run must not delete the config file"
    );
}
