//! Frozen tests for T-04.11 — "Check a path against the sandbox".
//!
//! Criterion → test mapping:
//!   C1 -> c1_path_inside_declared_root_is_allowed
//!   C2 -> c2_path_outside_all_roots_is_denied
//!   C3 -> c3_traversal_escape_is_denied
//!   C3 -> c3_traversal_escape_at_root_boundary_is_denied

use zira_skills::{path_allowed, SkillManifest};

fn manifest(allowed_roots: Vec<&str>) -> SkillManifest {
    SkillManifest {
        name: "test-skill".into(),
        version: "0.1.0".into(),
        entry: "main.rs".into(),
        capabilities: vec![],
        allowed_roots: allowed_roots.into_iter().map(str::to_owned).collect(),
    }
}

// ---- C1 -------------------------------------------------------------------------------

/// C1: a candidate that lies directly inside a declared root returns `true`.
#[test]
fn c1_path_inside_declared_root_is_allowed() {
    let m = manifest(vec!["/tmp/skills/myskill"]);
    let candidate = std::path::Path::new("/tmp/skills/myskill/data/file.txt");
    assert!(
        path_allowed(&m, candidate),
        "a path lexically inside a declared root must be allowed"
    );
}

/// C1: a candidate that IS the declared root itself returns `true` (boundary: root == root).
#[test]
fn c1_path_equal_to_declared_root_is_allowed() {
    let m = manifest(vec!["/tmp/skills/myskill"]);
    let candidate = std::path::Path::new("/tmp/skills/myskill");
    assert!(
        path_allowed(&m, candidate),
        "a path equal to a declared root must be allowed"
    );
}

/// C1: a candidate is allowed if it lies under ANY one of multiple declared roots.
#[test]
fn c1_path_under_second_of_two_roots_is_allowed() {
    let m = manifest(vec!["/tmp/alpha", "/tmp/beta"]);
    let candidate = std::path::Path::new("/tmp/beta/subdir/file.rs");
    assert!(
        path_allowed(&m, candidate),
        "a path under any declared root (not just the first) must be allowed"
    );
}

// ---- C2 -------------------------------------------------------------------------------

/// C2: a candidate outside every declared root returns `false`.
#[test]
fn c2_path_outside_all_roots_is_denied() {
    let m = manifest(vec!["/tmp/skills/myskill"]);
    let candidate = std::path::Path::new("/home/user/secrets/password.txt");
    assert!(
        !path_allowed(&m, candidate),
        "a path outside every declared root must be denied"
    );
}

/// C2: a candidate that shares a prefix with a declared root but is not under it is denied.
///
/// `/tmp/skills/myskill_extra` shares the prefix `/tmp/skills/myskill` but is not a
/// child of it; the check must not be a simple string prefix match.
#[test]
fn c2_path_with_matching_prefix_but_not_child_is_denied() {
    let m = manifest(vec!["/tmp/skills/myskill"]);
    // This is a sibling, not a child — a naive startswith check would wrongly allow it.
    let candidate = std::path::Path::new("/tmp/skills/myskill_extra/file.txt");
    assert!(
        !path_allowed(&m, candidate),
        "a path whose string starts with a root but is not a child of it must be denied"
    );
}

/// C2: an empty allowed_roots list denies every candidate.
#[test]
fn c2_empty_roots_denies_any_candidate() {
    let m = manifest(vec![]);
    let candidate = std::path::Path::new("/tmp/skills/anything/file.txt");
    assert!(
        !path_allowed(&m, candidate),
        "an empty allowed_roots list must deny every path"
    );
}

// ---- C3 -------------------------------------------------------------------------------

/// C3: a candidate formed by joining a declared root with `../` escapes above the root
///     and must be denied — the `..` cannot smuggle a path outside the sandbox.
#[test]
fn c3_traversal_escape_is_denied() {
    let m = manifest(vec!["/tmp/skills/myskill"]);
    // Lexically: /tmp/skills/myskill/../secret → /tmp/skills/secret
    // /tmp/skills/secret is NOT under /tmp/skills/myskill — deny.
    let candidate = std::path::Path::new("/tmp/skills/myskill/../secret.txt");
    assert!(
        !path_allowed(&m, candidate),
        "a `../` traversal escaping above the declared root must be denied"
    );
}

/// C3: a deeper `../` chain that climbs above the root is also denied.
#[test]
fn c3_traversal_escape_at_root_boundary_is_denied() {
    let m = manifest(vec!["/tmp/skills/myskill"]);
    // /tmp/skills/myskill/subdir/../../other → /tmp/skills/other — deny.
    let candidate = std::path::Path::new("/tmp/skills/myskill/subdir/../../other/file.txt");
    assert!(
        !path_allowed(&m, candidate),
        "a multi-hop `../` traversal escaping above the declared root must be denied"
    );
}
