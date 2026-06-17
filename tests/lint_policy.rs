//! Frozen lint-policy tests for T-00.03 — "Configure the lint policy".
//!
//! These assert the tool-observable style + lint floor: the pinned toolchain, a rustfmt
//! config that makes `cargo fmt --all --check` clean, and a workspace clippy gate that
//! denies warnings while `cargo clippy --workspace` still exits 0 on the scaffold. The
//! structural checks read the config files directly (pure std, hermetic); the two command
//! checks invoke the real tools — the criteria are explicitly defined by their exit codes,
//! so a file-only check would be a weaker detector. Each criterion maps to ≥1 test:
//!
//!   C1 -> c1_toolchain_pins_stable_channel, c1_toolchain_includes_rustfmt_and_clippy
//!   C2 -> c2_rustfmt_config_exists,         c2_cargo_fmt_all_check_exits_zero
//!   C3 -> c3_workspace_denies_clippy_warnings, c3_cargo_clippy_workspace_exits_zero

use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

/// Repo root = the manifest dir of this (root) package.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The `cargo` to drive nested tool invocations with (respects the harness's CARGO).
fn cargo() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn read(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Whitespace-stripped view of `text` for order/spacing-insensitive substring matching.
fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Lines belonging to the `[toolchain]` table body (up to the next table header).
fn table_body(text: &str, name: &str) -> String {
    let header = format!("[{name}]");
    let mut body = String::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
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

/// Concatenated body of every `[workspace.lints...]` table (the table itself and any
/// `[workspace.lints.clippy]` / `[workspace.lints.rust]` sub-tables).
fn lints_region(text: &str) -> String {
    let mut region = String::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            inside = trimmed.starts_with("[workspace.lints");
            continue;
        }
        if inside {
            region.push_str(line);
            region.push('\n');
        }
    }
    region
}

// ---- C1: pinned toolchain ------------------------------------------------------------

#[test]
fn c1_toolchain_pins_stable_channel() {
    let manifest = read(&repo_root().join("rust-toolchain.toml"));
    let toolchain = table_body(&manifest, "toolchain");
    let normalized = squeeze(&toolchain);
    assert!(
        normalized.contains("channel=\"stable\"") || normalized.contains("channel='stable'"),
        "rust-toolchain.toml [toolchain] must pin channel = \"stable\"; got body:\n{toolchain}"
    );
}

#[test]
fn c1_toolchain_includes_rustfmt_and_clippy() {
    let manifest = read(&repo_root().join("rust-toolchain.toml"));
    let toolchain = table_body(&manifest, "toolchain");
    let normalized = squeeze(&toolchain);
    assert!(
        normalized.contains("components=["),
        "rust-toolchain.toml [toolchain] must declare a `components` array; got body:\n{toolchain}"
    );
    assert!(
        toolchain.contains("rustfmt"),
        "rust-toolchain.toml components must include `rustfmt`; got body:\n{toolchain}"
    );
    assert!(
        toolchain.contains("clippy"),
        "rust-toolchain.toml components must include `clippy`; got body:\n{toolchain}"
    );
}

// ---- C2: rustfmt config + clean tree -------------------------------------------------

#[test]
fn c2_rustfmt_config_exists() {
    let path = repo_root().join("rustfmt.toml");
    assert!(
        path.is_file(),
        "a rustfmt.toml must exist at the repo root: {}",
        path.display()
    );
}

#[test]
fn c2_cargo_fmt_all_check_exits_zero() {
    let output = Command::new(cargo())
        .args(["fmt", "--all", "--check"])
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|e| panic!("spawn `cargo fmt --all --check`: {e}"));
    assert!(
        output.status.success(),
        "`cargo fmt --all --check` must exit 0 on the scaffolded tree; status {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

// ---- C3: workspace clippy gate -------------------------------------------------------

#[test]
fn c3_workspace_denies_clippy_warnings() {
    let manifest = read(&repo_root().join("Cargo.toml"));
    assert!(
        manifest
            .lines()
            .any(|l| l.trim().starts_with("[workspace.lints")),
        "root Cargo.toml must declare a [workspace.lints] table (or sub-table) configuring the clippy gate"
    );
    let region = squeeze(&lints_region(&manifest));
    assert!(
        region.contains("=\"deny\"") || region.contains("='deny'"),
        "the [workspace.lints] config must deny warnings (a `= \"deny\"` level); got region:\n{}",
        lints_region(&manifest)
    );
}

#[test]
fn c3_cargo_clippy_workspace_exits_zero() {
    // A dedicated target dir so the nested clippy build does not contend with the outer
    // `cargo test` target lock (same-dir would deadlock).
    let target = repo_root().join("target").join("lint-gate");
    let output = Command::new(cargo())
        .args(["clippy", "--workspace"])
        .current_dir(repo_root())
        .env("CARGO_TARGET_DIR", &target)
        .output()
        .unwrap_or_else(|e| panic!("spawn `cargo clippy --workspace`: {e}"));
    assert!(
        output.status.success(),
        "`cargo clippy --workspace` must exit 0 on the scaffolded tree; status {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
