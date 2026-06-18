//! Frozen tests for T-04.16 — "Scaffold the MCP config".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_mcpservers_key_and_command_from_manifest
//!   C2 -> c2_factory_json_parses_and_keys_match_manifest
//!   C3 -> c3_generated_config_round_trips

use serde_json::Value;
use zira_skills::{mcp_config_from_manifest, SkillManifest};

fn sample_manifest() -> SkillManifest {
    SkillManifest {
        name: "my-skill".to_string(),
        version: "1.0.0".to_string(),
        entry: "bin/my-skill-server".to_string(),
        capabilities: vec!["fs.read".to_string()],
        allowed_roots: vec!["/home/user/projects".to_string()],
    }
}

// ---- C1 — mcpServers table contains an entry keyed by manifest name with command from entry ----

/// Calls the factory and asserts that the returned Value:
/// - has an `mcpServers` object at the top level, and
/// - that object contains a key matching the manifest `name`, and
/// - that entry's `command` field equals the manifest `entry`.
#[test]
fn c1_mcpservers_key_and_command_from_manifest() {
    let m = sample_manifest();
    let config: Value = mcp_config_from_manifest(&m);

    // Top-level key must be "mcpServers"
    let servers = config
        .get("mcpServers")
        .expect("config must have a `mcpServers` key");
    assert!(
        servers.is_object(),
        "`mcpServers` must be a JSON object, got: {servers:?}"
    );

    // Entry keyed by manifest name must exist
    let entry = servers
        .get(&m.name)
        .expect("mcpServers must contain an entry keyed by the manifest name");

    // That entry's `command` must equal the manifest `entry` field
    let command = entry
        .get("command")
        .expect("server entry must have a `command` field");
    assert_eq!(
        command.as_str().expect("`command` must be a string"),
        m.entry.as_str(),
        "`command` must match the manifest `entry`"
    );
}

// ---- C2 — integration: factory returns parseable JSON with matching keys -----------------

/// Calls the factory on a SkillManifest and asserts that:
/// - the returned Value is a non-null object,
/// - the `mcpServers` key is present,
/// - the server entry's name key matches the manifest `name`,
/// - the server entry's `command` matches the manifest `entry`.
#[test]
fn c2_factory_json_parses_and_keys_match_manifest() {
    let m = SkillManifest {
        name: "audio-tool".to_string(),
        version: "0.3.1".to_string(),
        entry: "/usr/local/bin/audio-tool-mcp".to_string(),
        capabilities: vec!["audio.play".to_string()],
        allowed_roots: vec![],
    };

    let config: Value = mcp_config_from_manifest(&m);

    // Must be a JSON object
    assert!(config.is_object(), "config must be a JSON object");

    // Must contain `mcpServers`
    assert!(
        config.get("mcpServers").is_some(),
        "config must contain the `mcpServers` key"
    );

    let servers = &config["mcpServers"];

    // Server entry keyed by manifest name
    assert!(
        servers.get(&m.name).is_some(),
        "mcpServers must have an entry keyed by \"{}\"",
        m.name
    );

    let server_entry = &servers[&m.name];

    // command matches manifest entry
    assert_eq!(
        server_entry["command"]
            .as_str()
            .expect("`command` must be a JSON string"),
        m.entry,
        "`command` in the server entry must equal the manifest `entry`"
    );
}

// ---- C3 — generated config serializes to a string and re-parses to an equal Value ------

/// Asserts that serializing the generated config to a JSON string and then parsing it
/// again yields a Value equal to the original — confirming the config is a stable,
/// valid MCP skeleton.
#[test]
fn c3_generated_config_round_trips() {
    let m = sample_manifest();
    let config: Value = mcp_config_from_manifest(&m);

    // Serialize to string
    let serialized = serde_json::to_string(&config).expect("config must serialize to a string");
    assert!(
        !serialized.is_empty(),
        "serialized config must be a non-empty string"
    );

    // Re-parse from string
    let reparsed: Value =
        serde_json::from_str(&serialized).expect("serialized config must re-parse as valid JSON");

    // Must equal original
    assert_eq!(
        config, reparsed,
        "re-parsed config must equal the original Value"
    );
}
