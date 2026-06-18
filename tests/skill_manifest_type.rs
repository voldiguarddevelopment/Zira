//! Frozen tests for T-04.01 — "Define the SkillManifest type".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_skill_manifest_fields_and_derives
//!   C2 -> c2_skill_manifest_field_readback,
//!          c2_skill_manifest_empty_vecs_are_legal
//!   C3 -> c3_skill_manifest_serde_json_round_trip

use zira_skills::SkillManifest;

// Compile-time proof that SkillManifest: Debug + Clone + PartialEq.
fn assert_debug_clone_partialeq<T: std::fmt::Debug + Clone + PartialEq>(_: &T) {}

// Compile-time proof that SkillManifest: serde::Serialize + serde::de::DeserializeOwned.
fn assert_serde<T: serde::Serialize + serde::de::DeserializeOwned>() {}

// ---- C1 — struct exists with all five public fields and required derives ----------------

/// Constructs SkillManifest using named-field syntax, confirming every field exists with
/// the declared type, and exercises the Debug/Clone/PartialEq derive bounds.
#[test]
fn c1_skill_manifest_fields_and_derives() {
    let m = SkillManifest {
        name: "example-skill".to_string(),
        version: "1.0.0".to_string(),
        entry: "skill.wasm".to_string(),
        capabilities: vec!["fs.read".to_string()],
        allowed_roots: vec!["/home/user/projects".to_string()],
    };
    assert_debug_clone_partialeq(&m);
    // Clone produces an equal value.
    assert_eq!(m.clone(), m);
    // Debug output is non-empty.
    assert!(!format!("{m:?}").is_empty());
    // Confirm Serialize + DeserializeOwned bounds are satisfied (compile-time only call).
    assert_serde::<SkillManifest>();
}

// ---- C2 — field readback after construction --------------------------------------------

/// Constructs a fully-populated SkillManifest and asserts that each field stores and
/// returns exactly the value that was passed in.
#[test]
fn c2_skill_manifest_field_readback() {
    let m = SkillManifest {
        name: "my-skill".to_string(),
        version: "0.2.1".to_string(),
        entry: "bin/run".to_string(),
        capabilities: vec!["net.connect".to_string(), "fs.write".to_string()],
        allowed_roots: vec!["/tmp".to_string(), "/var/data".to_string()],
    };
    assert_eq!(m.name, "my-skill");
    assert_eq!(m.version, "0.2.1");
    assert_eq!(m.entry, "bin/run");
    assert_eq!(m.capabilities, vec!["net.connect", "fs.write"]);
    assert_eq!(m.allowed_roots, vec!["/tmp", "/var/data"]);
}

/// An empty capabilities or allowed_roots vec is legal at construction time.
/// (Default-deny enforcement is downstream, not in the type itself.)
#[test]
fn c2_skill_manifest_empty_vecs_are_legal() {
    let m = SkillManifest {
        name: "minimal".to_string(),
        version: "0.0.1".to_string(),
        entry: "run".to_string(),
        capabilities: vec![],
        allowed_roots: vec![],
    };
    assert!(m.capabilities.is_empty());
    assert!(m.allowed_roots.is_empty());
}

// ---- C3 — serde_json round-trip --------------------------------------------------------

/// Serializes a SkillManifest to JSON and deserializes it back; the result must equal
/// the original. This pins the serde stability of the type.
#[test]
fn c3_skill_manifest_serde_json_round_trip() {
    let original = SkillManifest {
        name: "round-trip-skill".to_string(),
        version: "3.1.4".to_string(),
        entry: "dist/main.wasm".to_string(),
        capabilities: vec!["fs.read".to_string(), "env.read".to_string()],
        allowed_roots: vec!["/projects/myapp".to_string()],
    };
    let json = serde_json::to_string(&original).expect("serialize failed");
    let recovered: SkillManifest = serde_json::from_str(&json).expect("deserialize failed");
    assert_eq!(original, recovered);
}
