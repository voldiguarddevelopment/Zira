//! Frozen tests for T-00.03 — "Configure the lint policy".
//!
//! Asserts the config-file presence and content required by the lint policy: a
//! rust-toolchain.toml pinning stable with rustfmt and clippy components, a rustfmt.toml
//! that is present and loadable, and a [workspace.lints.clippy] table in the root
//! Cargo.toml.  All checks are pure-std and offline-safe.
//!
//!   C1 -> c1_rust_toolchain_file_exists, c1_toolchain_pins_stable_channel,
//!          c1_toolchain_includes_rustfmt_component, c1_toolchain_includes_clippy_component
//!   C2 -> c2_rustfmt_file_exists, c2_rustfmt_file_is_loadable
//!   C3 -> c3_cargo_toml_has_workspace_lints_clippy_table

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_root_manifest() -> String {
    let path = repo_root().join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_toolchain() -> String {
    let path = repo_root().join("rust-toolchain.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_rustfmt() -> String {
    let path = repo_root().join("rustfmt.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_rust_toolchain_file_exists() {
    let path = repo_root().join("rust-toolchain.toml");
    assert!(
        path.is_file(),
        "rust-toolchain.toml must exist at the workspace root"
    );
}

#[test]
fn c1_toolchain_pins_stable_channel() {
    let content = read_toolchain();
    let normalized: String = content.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalized.contains("channel=\"stable\"") || normalized.contains("channel='stable'"),
        "rust-toolchain.toml must set channel = \"stable\"; got:\n{content}"
    );
}

#[test]
fn c1_toolchain_includes_rustfmt_component() {
    let content = read_toolchain();
    assert!(
        content.contains("rustfmt"),
        "rust-toolchain.toml must list rustfmt in the components array; got:\n{content}"
    );
}

#[test]
fn c1_toolchain_includes_clippy_component() {
    let content = read_toolchain();
    assert!(
        content.contains("clippy"),
        "rust-toolchain.toml must list clippy in the components array; got:\n{content}"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_rustfmt_file_exists() {
    let path = repo_root().join("rustfmt.toml");
    assert!(
        path.is_file(),
        "rustfmt.toml must exist at the workspace root"
    );
}

#[test]
fn c2_rustfmt_file_is_loadable() {
    let content = read_rustfmt();
    assert!(
        !content.trim().is_empty(),
        "rustfmt.toml must be non-empty; the file is present but contains no content"
    );
    // Minimal TOML structure: at least one key = value line or table header.
    let has_toml_content = content.lines().any(|l| {
        let t = l.trim();
        !t.starts_with('#') && !t.is_empty() && (t.contains('=') || t.starts_with('['))
    });
    assert!(
        has_toml_content,
        "rustfmt.toml must contain parseable TOML (key = value entries); got:\n{content}"
    );
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_cargo_toml_has_workspace_lints_clippy_table() {
    let manifest = read_root_manifest();
    let has_clippy_table = manifest
        .lines()
        .any(|l| l.trim() == "[workspace.lints.clippy]");
    assert!(
        has_clippy_table,
        "root Cargo.toml must declare a [workspace.lints.clippy] table; \
         searched manifest:\n{manifest}"
    );
}
