//! Frozen tests for T-00.02 — "Declare the shared dependencies".
//!
//! These assert the cargo-observable shape of the workspace's shared dependency surface:
//! the six core deps are declared once at the root `[workspace.dependencies]` table with
//! pinned versions, at least one member inherits one via `{ workspace = true }`, and the
//! whole thing resolves (`cargo build`) and is metadata-clean (`cargo metadata`).
//!
//! Manifest structure is checked by reading the TOML directly (pure std, hermetic). The
//! resolution criteria shell out to the real `cargo` so the inheritance wiring is proven
//! end-to-end, not merely asserted on text. Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_workspace_dependencies_declares_six_core_deps,
//!         c1_each_shared_dep_has_a_pinned_version
//!   C2 -> c2_a_member_inherits_a_shared_dep_via_workspace_true,
//!         c2_cargo_build_resolves_the_workspace
//!   C3 -> c3_cargo_metadata_exits_zero,
//!         c3_each_shared_dep_appears_exactly_once_in_workspace_dependencies

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// The six shared dependencies that MUST be declared once at the workspace root.
const SHARED_DEPS: [&str; 6] = ["tokio", "serde", "serde_json", "thiserror", "anyhow", "tracing"];

/// The ten member crates, any of which may carry the inherited dependency.
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

/// True if `line` carries a pinned version: a quoted token whose first char is a digit
/// (covers both `dep = "1.2"` and `dep = { version = "1.2", .. }`).
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

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_workspace_dependencies_declares_six_core_deps() {
    let manifest = read_root_manifest();
    let body = table_body(&manifest, "workspace.dependencies");
    assert!(
        !body.trim().is_empty(),
        "root Cargo.toml must declare a [workspace.dependencies] table"
    );
    for dep in SHARED_DEPS {
        assert!(
            !decl_lines(&body, dep).is_empty(),
            "[workspace.dependencies] must declare `{dep}`; got body:\n{body}"
        );
    }
}

#[test]
fn c1_each_shared_dep_has_a_pinned_version() {
    let manifest = read_root_manifest();
    let body = table_body(&manifest, "workspace.dependencies");
    for dep in SHARED_DEPS {
        let lines = decl_lines(&body, dep);
        assert!(
            !lines.is_empty(),
            "[workspace.dependencies] must declare `{dep}` before it can be pinned"
        );
        assert!(
            lines.iter().any(|l| has_pinned_version(l)),
            "`{dep}` in [workspace.dependencies] must pin a version; got:\n{}",
            lines.join("\n")
        );
    }
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_a_member_inherits_a_shared_dep_via_workspace_true() {
    let crates_dir = repo_root().join("crates");
    let mut inheriting: Vec<String> = Vec::new();

    for name in MEMBERS {
        let manifest_path = crates_dir.join(name).join("Cargo.toml");
        let Ok(manifest) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        // Look only inside the crate's dependency tables for a `workspace = true` entry
        // that names one of the six shared deps.
        for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let body = table_body(&manifest, table);
            for dep in SHARED_DEPS {
                for line in decl_lines(&body, dep) {
                    let normalized: String = line.chars().filter(|c| !c.is_whitespace()).collect();
                    if normalized.contains("workspace=true") {
                        inheriting.push(format!("{name}:{dep}"));
                    }
                }
            }
        }
    }

    assert!(
        !inheriting.is_empty(),
        "at least one member crate must consume a shared dep via `{{ workspace = true }}`"
    );
}

#[test]
fn c2_cargo_build_resolves_the_workspace() {
    // A separate target dir isolates this nested build from the outer `cargo test` build
    // lock and keeps scratch artifacts inside the gitignored /target tree.
    let target_dir = repo_root().join("target").join("t0002-build");
    let output = Command::new(cargo())
        .args(["build", "--workspace"])
        .current_dir(repo_root())
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("spawn cargo build");

    assert!(
        output.status.success(),
        "`cargo build --workspace` must resolve the workspace-inherited deps; \
         exit={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_cargo_metadata_exits_zero() {
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
fn c3_each_shared_dep_appears_exactly_once_in_workspace_dependencies() {
    let manifest = read_root_manifest();
    let body = table_body(&manifest, "workspace.dependencies");
    assert!(
        !body.trim().is_empty(),
        "root Cargo.toml must declare a [workspace.dependencies] table"
    );
    for dep in SHARED_DEPS {
        let count = decl_lines(&body, dep).len();
        assert_eq!(
            count, 1,
            "`{dep}` must appear exactly once in [workspace.dependencies] (no version \
             drift); found {count} declarations"
        );
    }
}
