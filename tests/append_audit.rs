//! Frozen tests for T-04.13 — "Append an audit entry".
//!
//! Criterion → test mapping:
//!   C1 -> c1_genesis_constant_is_64_lowercase_hex_zeros
//!   C1 -> c1_append_to_nonempty_chain_links_to_last_entry_hash
//!   C2 -> c2_empty_chain_prev_hash_equals_genesis
//!   C3 -> c3_two_entries_second_prev_hash_equals_first_entry_hash

use zira_skills::{append_audit, AuditEntry, GENESIS_HASH};

// ---- C1 -----------------------------------------------------------------------

/// C1: The genesis constant is pinned to 64 lowercase hex zeros — the sentinel
///     used when the chain is empty.
#[test]
fn c1_genesis_constant_is_64_lowercase_hex_zeros() {
    assert_eq!(
        GENESIS_HASH,
        "0000000000000000000000000000000000000000000000000000000000000000",
        "GENESIS_HASH must be exactly 64 lowercase hex zeros"
    );
    assert_eq!(GENESIS_HASH.len(), 64, "GENESIS_HASH must be exactly 64 characters");
}

/// C1: Appending to a non-empty chain produces an entry whose `prev_hash` equals
///     the last entry's `entry_hash` — not the genesis constant.
#[test]
fn c1_append_to_nonempty_chain_links_to_last_entry_hash() {
    let key = b"ratchet-test-key";
    let existing = AuditEntry {
        skill_name: "existing-skill".to_string(),
        action: "install".to_string(),
        prev_hash: GENESIS_HASH.to_string(),
        entry_hash: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
    };
    let chain = [existing.clone()];
    let new_entry = append_audit(key, &chain, "next-skill", "upgrade");
    assert_eq!(
        new_entry.prev_hash, existing.entry_hash,
        "appending to a non-empty chain must set prev_hash to the last entry's entry_hash"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// C2: Appending to an empty chain produces an entry whose `prev_hash` equals
///     the genesis constant.
#[test]
fn c2_empty_chain_prev_hash_equals_genesis() {
    let key = b"ratchet-test-key";
    let entry = append_audit(key, &[], "first-skill", "install");
    assert_eq!(
        entry.prev_hash,
        GENESIS_HASH,
        "first entry in an empty chain must link to the genesis constant"
    );
}

// ---- C3 -----------------------------------------------------------------------

/// C3: Appending two entries in sequence produces a linked chain where the
///     second entry's `prev_hash` equals the first entry's `entry_hash`.
#[test]
fn c3_two_entries_second_prev_hash_equals_first_entry_hash() {
    let key = b"ratchet-test-key";
    let first = append_audit(key, &[], "skill-alpha", "install");
    let chain = [first.clone()];
    let second = append_audit(key, &chain, "skill-beta", "activate");
    assert_eq!(
        second.prev_hash, first.entry_hash,
        "second entry's prev_hash must equal the first entry's entry_hash"
    );
}
