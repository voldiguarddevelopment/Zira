//! Frozen tests for T-02.01 — "Declare the memory dependencies".
//!
//! These assert the cargo-observable shape of the `zira-memory` dependency surface:
//! `redb` is declared exactly once in `[workspace.dependencies]` with a pinned version,
//! and `zira-memory/Cargo.toml` inherits `redb`, `serde`, `serde_json`, and `zira-proto`
//! via `{ workspace = true }` (or a path dep). Manifest structure is checked by reading
//! TOML directly (hermetic). Resolution is proven by shelling out to the real `cargo`.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_redb_declared_in_workspace_dependencies,
//!         c1_redb_has_pinned_version_in_workspace,
//!         c1_zira_memory_inherits_memory_deps,
//!         c1_cargo_build_zira_memory_exits_zero
//!   C2 -> c2_cargo_metadata_exits_zero,
//!         c2_redb_appears_exactly_once_in_workspace_dependencies

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Repo root = the manifest dir of this (root) package.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_root_manifest() -> String {
    let path = repo_root().join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read_memory_manifest() -> String {
    let path = repo_root()
        .join("crates")
        .join("zira-memory")
        .join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The `cargo` binary that invoked this test (falls back to PATH `cargo`).
fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

/// Return the body lines of the top-level `[name]` table (dotted names like
/// `workspace.dependencies` are matched verbatim), up to the next table header.
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

/// Lines in `body` that *declare* dependency key `dep` (i.e. start, ignoring leading
/// whitespace, with `dep` followed by optional spaces and `=`). Continuation lines of a
/// multi-line inline table do not start with the key, so they are not counted.
fn decl_lines<'a>(body: &'a str, dep: &str) -> Vec<&'a str> {
    body.lines()
        .filter(|line| {
            let t = line.trim_start();
            match t.strip_prefix(dep) {
                Some(rest) => rest.trim_start().starts_with('='),
                None => false,
            }
        })
        .collect()
}

/// True if `line` carries a pinned version: a quoted token whose first char is a digit.
fn has_pinned_version(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            if let Some(&next) = bytes.get(i + 1) {
                if next.is_ascii_digit() {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}

/// True if `line` inherits from the workspace (`{ workspace = true }`) or is a path dep.
fn is_workspace_or_path_dep(line: &str) -> bool {
    let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    normalized.contains("workspace=true") || normalized.contains("path=")
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_redb_declared_in_workspace_dependencies() {
    let manifest = read_root_manifest();
    let body = table_body(&manifest, "workspace.dependencies");
    assert!(
        !body.trim().is_empty(),
        "root Cargo.toml must declare a [workspace.dependencies] table"
    );
    let lines = decl_lines(&body, "redb");
    assert!(
        !lines.is_empty(),
        "[workspace.dependencies] must declare `redb`; got body:\n{body}"
    );
}

#[test]
fn c1_redb_has_pinned_version_in_workspace() {
    let manifest = read_root_manifest();
    let body = table_body(&manifest, "workspace.dependencies");
    let lines = decl_lines(&body, "redb");
    assert!(
        !lines.is_empty(),
        "[workspace.dependencies] must declare `redb` before it can be pinned"
    );
    assert!(
        lines.iter().any(|l| has_pinned_version(l)),
        "`redb` in [workspace.dependencies] must pin a version; got:\n{}",
        lines.join("\n")
    );
}

#[test]
fn c1_zira_memory_inherits_memory_deps() {
    let manifest = read_memory_manifest();

    // All four deps must appear in either [dependencies] or [dev-dependencies].
    let memory_deps = ["redb", "serde", "serde_json", "zira-proto"];

    for dep in memory_deps {
        let mut found = false;
        for table in ["dependencies", "dev-dependencies"] {
            let body = table_body(&manifest, table);
            for line in decl_lines(&body, dep) {
                if is_workspace_or_path_dep(line) {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        assert!(
            found,
            "crates/zira-memory/Cargo.toml must inherit `{dep}` via \
             `{{ workspace = true }}` or a path dep; manifest:\n{manifest}"
        );
    }
}

#[test]
fn c1_cargo_build_zira_memory_exits_zero() {
    let target_dir = repo_root().join("target").join("t0201-build");
    let output = Command::new(cargo())
        .args(["build", "-p", "zira-memory"])
        .current_dir(repo_root())
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("spawn cargo build");

    assert!(
        output.status.success(),
        "`cargo build -p zira-memory` must exit 0; exit={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_cargo_metadata_exits_zero() {
    let output = Command::new(cargo())
        .args(["metadata", "--format-version", "1"])
        .current_dir(repo_root())
        .output()
        .expect("spawn cargo metadata");

    assert!(
        output.status.success(),
        "`cargo metadata` must exit 0; exit={:?}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn c2_redb_appears_exactly_once_in_workspace_dependencies() {
    let manifest = read_root_manifest();
    let body = table_body(&manifest, "workspace.dependencies");
    assert!(
        !body.trim().is_empty(),
        "root Cargo.toml must declare a [workspace.dependencies] table"
    );
    let count = decl_lines(&body, "redb").len();
    assert_eq!(
        count, 1,
        "`redb` must appear exactly once in [workspace.dependencies] (no version drift); \
         found {count} declarations"
    );
}
