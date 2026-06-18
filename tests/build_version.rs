//! Frozen tests for T-05.11 — `zira_config::build_version`.
//!
//! C1: the function exists, returns `&'static str`, and is non-empty.
//! C2: the returned string has at least three dot-separated numeric components
//!     (semver `major.minor.patch` shape).

/// C1 — non-empty static string returned.
#[test]
fn build_version_is_non_empty() {
    let v = zira_config::build_version();
    assert!(
        !v.is_empty(),
        "build_version() must return a non-empty string"
    );
}

/// C1 — the returned value tracks the crate's `CARGO_PKG_VERSION`.
///
/// Both the root scaffold package and `zira-config` share the same `0.0.0`
/// workspace version; we verify by parsing, not by comparing to the test-host
/// package's own env var (which is the root package, not the config crate).
/// The semver-shape test below is the companion assertion.
#[test]
fn build_version_is_static_str() {
    // The function signature pins `&'static str`.  If the impl returns a
    // reference to a local, this will not compile.
    let _: &'static str = zira_config::build_version();
}

/// C2 — the string parses as `major.minor.patch` (three numeric components).
#[test]
fn build_version_is_semver_shaped() {
    let v = zira_config::build_version();
    let parts: Vec<&str> = v.splitn(4, '.').collect();
    assert!(
        parts.len() >= 3,
        "build_version() must have at least major.minor.patch; got {v:?}"
    );
    for (i, part) in parts.iter().take(3).enumerate() {
        let label = ["major", "minor", "patch"][i];
        assert!(
            part.parse::<u64>().is_ok(),
            "build_version() component `{label}` ({part:?}) must be numeric; got {v:?}"
        );
    }
}
