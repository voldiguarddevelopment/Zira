//! Frozen tests for T-04.15 — "Register a skill".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_register_then_lookup_returns_manifest,
//!          c1_register_then_list_includes_manifest
//!   C2 -> c2_remove_present_returns_true_and_clears_lookup,
//!          c2_remove_absent_returns_false
//!   C3 -> c3_reregister_same_name_replaces_not_duplicates

use zira_skills::{SkillManifest, SkillRegistry};

fn make_manifest(name: &str) -> SkillManifest {
    SkillManifest {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        entry: "skill.wasm".to_string(),
        capabilities: vec![],
        allowed_roots: vec![],
    }
}

// ---- C1 — register + lookup + list --------------------------------------------------------

/// Registering a manifest makes `lookup` return it by name.
#[test]
fn c1_register_then_lookup_returns_manifest() {
    let mut registry = SkillRegistry::new();
    let m = make_manifest("my-skill");
    registry.register(m.clone());
    let found = registry.lookup("my-skill").expect("manifest should be present after register");
    assert_eq!(found, &m);
}

/// Registering a manifest makes `list` include it.
#[test]
fn c1_register_then_list_includes_manifest() {
    let mut registry = SkillRegistry::new();
    let m = make_manifest("listed-skill");
    registry.register(m.clone());
    let all = registry.list();
    assert!(
        all.contains(&&m),
        "list() should include the registered manifest"
    );
}

// ---- C2 — remove semantics ----------------------------------------------------------------

/// `remove` of a registered name returns `true` and a subsequent `lookup` returns `None`.
#[test]
fn c2_remove_present_returns_true_and_clears_lookup() {
    let mut registry = SkillRegistry::new();
    registry.register(make_manifest("removable"));
    assert!(registry.remove("removable"), "remove of a known name should return true");
    assert!(
        registry.lookup("removable").is_none(),
        "lookup after remove should return None"
    );
}

/// `remove` of an absent name returns `false` (benign no-op).
#[test]
fn c2_remove_absent_returns_false() {
    let mut registry = SkillRegistry::new();
    assert!(!registry.remove("never-registered"), "remove of absent name should return false");
}

// ---- C3 — replace on duplicate name -------------------------------------------------------

/// Re-registering a manifest under an existing name replaces the entry; `list` length
/// stays at one for that name (no duplicates).
#[test]
fn c3_reregister_same_name_replaces_not_duplicates() {
    let mut registry = SkillRegistry::new();
    let original = SkillManifest {
        name: "dup-skill".to_string(),
        version: "1.0.0".to_string(),
        entry: "v1.wasm".to_string(),
        capabilities: vec![],
        allowed_roots: vec![],
    };
    let replacement = SkillManifest {
        name: "dup-skill".to_string(),
        version: "2.0.0".to_string(),
        entry: "v2.wasm".to_string(),
        capabilities: vec![],
        allowed_roots: vec![],
    };
    registry.register(original);
    registry.register(replacement.clone());

    // lookup must return the replacement, not the original
    let found = registry.lookup("dup-skill").expect("manifest must be present after re-register");
    assert_eq!(found, &replacement, "lookup should return the replacement manifest");
    assert_eq!(found.version, "2.0.0");

    // list must not contain duplicates for that name
    let all = registry.list();
    let count = all.iter().filter(|m| m.name == "dup-skill").count();
    assert_eq!(count, 1, "list should contain exactly one entry for the registered name");
}
