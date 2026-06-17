//! Frozen scaffold tests for T-00.01 — "Scaffold the Cargo workspace".
//!
//! These assert the cargo-observable structure of the ten-crate workspace by reading the
//! manifests and the on-disk crate layout directly (pure std, no external deps, so the
//! gate stays hermetic and offline-safe). Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_root_is_hybrid_package_and_workspace, c1_workspace_sets_resolver_two
//!   C2 -> c2_root_declares_workspace_members,      c2_all_ten_member_crates_have_named_manifests
//!   C3 -> c3_zira_is_a_binary_target,              c3_other_nine_are_library_targets

use std::fs;
use std::path::PathBuf;

/// The ten member crates the workspace must contain, in plan order.
const MEMBERS: [&str; 10] = [
    "zira",
    "zira-core",
    "zira-bridge",
    "zira-voice",
    "zira-emotion",
    "zira-avatar",
    "zira-memory",
    "zira-skills",
    "zira-config",
    "zira-proto",
];

/// Repo root = the manifest dir of this (root) package.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_root_manifest() -> String {
    let path = repo_root().join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// True if `text` contains a bare top-level table header line `[name]`.
fn has_table_header(text: &str, name: &str) -> bool {
    let header = format!("[{name}]");
    text.lines().any(|l| l.trim() == header)
}

/// Return the lines belonging to the top-level `[name]` table (its key/value body,
/// up to the next table header). Empty string if the table is absent or has no body.
fn table_body(text: &str, name: &str) -> String {
    let header = format!("[{name}]");
    let mut body = String::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        // Any `[...]` line is a table header that ends the current section. This keeps
        // sub-tables like `[workspace.dependencies]` from leaking into `[workspace]`.
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            inside = trimmed == header;
            continue;
        }
        if inside {
            body.push_str(line);
            body.push('\n');
        }
    }
    body
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_root_is_hybrid_package_and_workspace() {
    let manifest = read_root_manifest();
    assert!(
        has_table_header(&manifest, "package"),
        "root Cargo.toml must declare a [package] table"
    );
    assert!(
        has_table_header(&manifest, "workspace"),
        "root Cargo.toml must declare a [workspace] table (hybrid package + workspace root)"
    );
}

#[test]
fn c1_workspace_sets_resolver_two() {
    let manifest = read_root_manifest();
    let workspace = table_body(&manifest, "workspace");
    let normalized: String = workspace.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalized.contains("resolver=\"2\"") || normalized.contains("resolver='2'"),
        "[workspace] must set resolver = \"2\"; got body:\n{workspace}"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_root_declares_workspace_members() {
    let manifest = read_root_manifest();
    let workspace = table_body(&manifest, "workspace");
    assert!(
        workspace.contains("members") && workspace.contains('['),
        "[workspace] must declare a `members` array; got body:\n{workspace}"
    );
}

#[test]
fn c2_all_ten_member_crates_have_named_manifests() {
    let crates_dir = repo_root().join("crates");

    // Exactly the ten member crates — no fewer, no extras.
    let mut found: Vec<String> = fs::read_dir(&crates_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", crates_dir.display()))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    found.sort();
    let mut expected: Vec<String> = MEMBERS.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "crates/ must contain exactly the ten member crates"
    );

    // Each member crate is a real package whose name matches its directory.
    for name in MEMBERS {
        let manifest_path = crates_dir.join(name).join("Cargo.toml");
        let manifest = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
        assert!(
            has_table_header(&manifest, "package"),
            "crates/{name}/Cargo.toml must declare a [package] table"
        );
        let normalized: String = manifest.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            normalized.contains(&format!("name=\"{name}\"")),
            "crates/{name}/Cargo.toml must set package name = \"{name}\""
        );
    }
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_zira_is_a_binary_target() {
    let zira = repo_root().join("crates").join("zira");
    assert!(
        zira.join("src").join("main.rs").is_file(),
        "crates/zira must expose a binary target at src/main.rs"
    );
    assert!(
        !zira.join("src").join("lib.rs").is_file(),
        "crates/zira must be a pure binary crate (no src/lib.rs)"
    );
}

#[test]
fn c3_other_nine_are_library_targets() {
    for name in MEMBERS.iter().filter(|n| **n != "zira") {
        let crate_dir = repo_root().join("crates").join(name);
        assert!(
            crate_dir.join("src").join("lib.rs").is_file(),
            "crates/{name} must expose a library target at src/lib.rs"
        );
        assert!(
            !crate_dir.join("src").join("main.rs").is_file(),
            "crates/{name} must be a pure library crate (no src/main.rs)"
        );
    }
}
