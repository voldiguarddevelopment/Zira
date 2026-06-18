//! Frozen tests for T-04.12 — "Define the audit entry".
//!
//! Criterion → test mapping:
//!   C1 -> c1_audit_entry_struct_fields
//!   C1 -> c1_audit_entry_derives_debug_clone_partialeq
//!   C1 -> c1_audit_entry_derives_serialize_deserialize
//!   C2 -> c2_construct_and_read_fields
//!   C2 -> c2_serde_json_round_trip
//!   C3 -> c3_hash_deterministic
//!   C3 -> c3_hash_is_nonempty_hex
//!   C3 -> c3_hash_sensitive_to_skill_name
//!   C3 -> c3_hash_sensitive_to_action
//!   C3 -> c3_hash_sensitive_to_prev_hash
//!   C3 -> c3_hash_sensitive_to_key

use zira_skills::{compute_entry_hash, AuditEntry};

// ---- C1 -----------------------------------------------------------------------

/// C1: `AuditEntry` has four public string fields and they read back correctly.
#[test]
fn c1_audit_entry_struct_fields() {
    let entry = AuditEntry {
        skill_name: "demo".to_string(),
        action: "install".to_string(),
        prev_hash: "abc".to_string(),
        entry_hash: "def".to_string(),
    };
    assert_eq!(entry.skill_name, "demo");
    assert_eq!(entry.action, "install");
    assert_eq!(entry.prev_hash, "abc");
    assert_eq!(entry.entry_hash, "def");
}

/// C1: `AuditEntry` derives `Debug`, `Clone`, and `PartialEq`.
#[test]
fn c1_audit_entry_derives_debug_clone_partialeq() {
    let e1 = AuditEntry {
        skill_name: "s".to_string(),
        action: "a".to_string(),
        prev_hash: "p".to_string(),
        entry_hash: "h".to_string(),
    };
    let e2 = e1.clone();
    assert_eq!(e1, e2, "cloned entry must equal the original");

    let e3 = AuditEntry {
        skill_name: "different".to_string(),
        ..e1.clone()
    };
    assert_ne!(e1, e3, "entries with different skill_name must not be equal");

    let _ = format!("{:?}", e1);
}

/// C1: `AuditEntry` derives `Serialize` and `Deserialize` — verified via round-trip.
#[test]
fn c1_audit_entry_derives_serialize_deserialize() {
    let e = AuditEntry {
        skill_name: "ser-skill".to_string(),
        action: "load".to_string(),
        prev_hash: "0000".to_string(),
        entry_hash: "ffff".to_string(),
    };
    let json = serde_json::to_string(&e).expect("Serialize derive must allow to_string");
    let back: AuditEntry =
        serde_json::from_str(&json).expect("Deserialize derive must parse the JSON back");
    assert_eq!(e, back, "entry must survive a serde_json encode/decode cycle");
}

// ---- C2 -----------------------------------------------------------------------

/// C2: construct an `AuditEntry` with all four fields and assert each reads back.
#[test]
fn c2_construct_and_read_fields() {
    let skill = "my-skill".to_string();
    let action = "deploy".to_string();
    let prev =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    let hash = "cafebabe0000000000000000000000000000000000000000000000000000cafe".to_string();

    let entry = AuditEntry {
        skill_name: skill.clone(),
        action: action.clone(),
        prev_hash: prev.clone(),
        entry_hash: hash.clone(),
    };

    assert_eq!(entry.skill_name, skill, "skill_name must round-trip through the struct");
    assert_eq!(entry.action, action, "action must round-trip through the struct");
    assert_eq!(entry.prev_hash, prev, "prev_hash must round-trip through the struct");
    assert_eq!(entry.entry_hash, hash, "entry_hash must round-trip through the struct");
}

/// C2: `AuditEntry` round-trips through `serde_json` to an equal value.
#[test]
fn c2_serde_json_round_trip() {
    let original = AuditEntry {
        skill_name: "serde-skill".to_string(),
        action: "activate".to_string(),
        prev_hash: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        entry_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
    };
    let json =
        serde_json::to_string(&original).expect("serialization must not fail");
    let restored: AuditEntry =
        serde_json::from_str(&json).expect("deserialization must not fail");
    assert_eq!(
        original, restored,
        "serde round-trip must yield an equal AuditEntry"
    );
}

// ---- C3 -----------------------------------------------------------------------

/// C3: `compute_entry_hash` is deterministic — same inputs produce the same hash.
#[test]
fn c3_hash_deterministic() {
    let key = b"ratchet-test-key";
    let h1 = compute_entry_hash(key, "my-skill", "install", "0000");
    let h2 = compute_entry_hash(key, "my-skill", "install", "0000");
    assert_eq!(h1, h2, "same inputs must yield the same hash on repeated calls");
}

/// C3: `compute_entry_hash` returns a non-empty 64-character lowercase hex string,
///     pinning the format to HMAC-SHA256.
#[test]
fn c3_hash_is_nonempty_hex() {
    let key = b"ratchet-test-key";
    let h = compute_entry_hash(key, "my-skill", "install", "0000");
    assert!(!h.is_empty(), "compute_entry_hash must return a non-empty string");
    assert_eq!(
        h.len(),
        64,
        "HMAC-SHA256 hex output must be exactly 64 characters; got {h:?}"
    );
    assert!(
        h.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
        "output must consist of lowercase hex digits; got {h:?}"
    );
}

/// C3: changing `skill_name` changes the hash.
#[test]
fn c3_hash_sensitive_to_skill_name() {
    let key = b"ratchet-test-key";
    let h1 = compute_entry_hash(key, "skill-alpha", "install", "0000");
    let h2 = compute_entry_hash(key, "skill-beta", "install", "0000");
    assert_ne!(h1, h2, "changing skill_name must change the hash");
}

/// C3: changing `action` changes the hash.
#[test]
fn c3_hash_sensitive_to_action() {
    let key = b"ratchet-test-key";
    let h1 = compute_entry_hash(key, "my-skill", "install", "0000");
    let h2 = compute_entry_hash(key, "my-skill", "upgrade", "0000");
    assert_ne!(h1, h2, "changing action must change the hash");
}

/// C3: changing `prev_hash` changes the hash.
#[test]
fn c3_hash_sensitive_to_prev_hash() {
    let key = b"ratchet-test-key";
    let h1 = compute_entry_hash(key, "my-skill", "install", "prev-a");
    let h2 = compute_entry_hash(key, "my-skill", "install", "prev-b");
    assert_ne!(h1, h2, "changing prev_hash must change the hash");
}

/// C3: changing the HMAC key changes the hash.
#[test]
fn c3_hash_sensitive_to_key() {
    let h1 = compute_entry_hash(b"key-alpha", "my-skill", "install", "0000");
    let h2 = compute_entry_hash(b"key-beta", "my-skill", "install", "0000");
    assert_ne!(h1, h2, "changing the HMAC key must change the hash");
}
