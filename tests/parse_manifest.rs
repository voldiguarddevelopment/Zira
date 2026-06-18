//! Frozen tests for T-04.02 — "Parse the manifest".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_parse_manifest_toml_valid_fixture
//!   C2 -> c2_parse_manifest_json_valid_fixture,
//!          c2_toml_and_json_parse_to_equal_values
//!   C3 -> c3_malformed_toml_returns_parse_error,
//!          c3_malformed_json_returns_parse_error

use zira_skills::{parse_manifest_json, parse_manifest_toml, ManifestError, SkillManifest};

// A valid TOML manifest fixture used across C1 and C2 tests.
const VALID_TOML: &str = r#"
name = "test-skill"
version = "1.0.0"
entry = "skill.wasm"
capabilities = ["fs.read", "net.connect"]
allowed_roots = ["/home/user/projects"]
"#;

// The same manifest encoded as JSON — must parse to an equal SkillManifest.
const VALID_JSON: &str = r#"{
    "name": "test-skill",
    "version": "1.0.0",
    "entry": "skill.wasm",
    "capabilities": ["fs.read", "net.connect"],
    "allowed_roots": ["/home/user/projects"]
}"#;

fn expected_manifest() -> SkillManifest {
    SkillManifest {
        name: "test-skill".to_string(),
        version: "1.0.0".to_string(),
        entry: "skill.wasm".to_string(),
        capabilities: vec!["fs.read".to_string(), "net.connect".to_string()],
        allowed_roots: vec!["/home/user/projects".to_string()],
    }
}

// ---- C1 — parse_manifest_toml deserializes a well-formed TOML fixture ---------------

/// parse_manifest_toml must return Ok(SkillManifest) for a complete, well-formed TOML
/// document, with every field matching the fixture.
#[test]
fn c1_parse_manifest_toml_valid_fixture() {
    let result = parse_manifest_toml(VALID_TOML);
    assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
    assert_eq!(result.unwrap(), expected_manifest());
}

// ---- C2 — parse_manifest_json + format equality -------------------------------------

/// parse_manifest_json must return Ok(SkillManifest) for a complete, well-formed JSON
/// document, with every field matching the fixture.
#[test]
fn c2_parse_manifest_json_valid_fixture() {
    let result = parse_manifest_json(VALID_JSON);
    assert!(result.is_ok(), "expected Ok but got: {:?}", result.err());
    assert_eq!(result.unwrap(), expected_manifest());
}

/// TOML and JSON forms of the same manifest must parse to equal SkillManifest values.
/// This pins that both parsers decode fields identically and share no format-specific bias.
#[test]
fn c2_toml_and_json_parse_to_equal_values() {
    let from_toml = parse_manifest_toml(VALID_TOML).expect("TOML parse failed");
    let from_json = parse_manifest_json(VALID_JSON).expect("JSON parse failed");
    assert_eq!(
        from_toml, from_json,
        "TOML and JSON forms of the same manifest must parse to equal SkillManifest values"
    );
}

// ---- C3 — malformed input returns Err(ManifestError::Parse(..)), never panics ------

/// Invalid TOML must return Err(ManifestError::Parse(_)), not a panic.
#[test]
fn c3_malformed_toml_returns_parse_error() {
    let result = parse_manifest_toml("not valid toml [[[");
    match result {
        Err(ManifestError::Parse(_)) => {}
        other => panic!(
            "expected Err(ManifestError::Parse(_)) for malformed TOML, got {:?}",
            other
        ),
    }
}

/// Invalid JSON must return Err(ManifestError::Parse(_)), not a panic.
#[test]
fn c3_malformed_json_returns_parse_error() {
    let result = parse_manifest_json("{not valid json");
    match result {
        Err(ManifestError::Parse(_)) => {}
        other => panic!(
            "expected Err(ManifestError::Parse(_)) for malformed JSON, got {:?}",
            other
        ),
    }
}
