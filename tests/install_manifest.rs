//! Frozen tests for T-05.12 — `zira_config::install_manifest`.
//!
//! C1: the function returns a string that contains the `build_version()` value.
//! C2: the returned string contains the application directory name `zira`.

/// C1 — manifest embeds the build version returned by `build_version()`.
#[test]
fn install_manifest_contains_build_version() {
    let manifest = zira_config::install_manifest();
    let version = zira_config::build_version();
    assert!(
        manifest.contains(version),
        "install_manifest() must contain build_version() ({version:?}); got:\n{manifest}"
    );
}

/// C1 — the manifest is a non-empty string (a degenerate empty string could never
/// contain the version, but this makes the invariant explicit).
#[test]
fn install_manifest_is_non_empty() {
    let manifest = zira_config::install_manifest();
    assert!(
        !manifest.is_empty(),
        "install_manifest() must return a non-empty string"
    );
}

/// C2 — manifest names the `zira` application directory.
#[test]
fn install_manifest_contains_app_dir_name() {
    let manifest = zira_config::install_manifest();
    assert!(
        manifest.contains("zira"),
        "install_manifest() must contain the application directory name \"zira\"; got:\n{manifest}"
    );
}
