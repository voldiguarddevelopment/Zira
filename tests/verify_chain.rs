//! Frozen tests for T-04.14 — "Verify the chain".
//!
//! Criterion → test mapping:
//!   C1 -> c1_intact_chain_built_by_append_audit_returns_true
//!   C2 -> c2_tampered_action_field_returns_false
//!   C3 -> c3_removed_entry_breaks_link_returns_false
//!   C3 -> c3_reordered_entries_break_link_returns_false

use zira_skills::{append_audit, verify_chain};

// ---- C1 -----------------------------------------------------------------------

/// C1: A chain built entirely by `append_audit` must be accepted as intact.
///     Verifies the happy path: every entry's recomputed hash matches and every
///     `prev_hash` equals its predecessor's `entry_hash`.
#[test]
fn c1_intact_chain_built_by_append_audit_returns_true() {
    let key = b"ratchet-test-key";
    let e1 = append_audit(key, &[], "skill-alpha", "install");
    let chain1 = vec![e1];
    let e2 = append_audit(key, &chain1, "skill-beta", "activate");
    let mut chain = chain1;
    chain.push(e2);
    let e3 = append_audit(key, &chain, "skill-gamma", "run");
    chain.push(e3);

    assert!(
        verify_chain(key, &chain),
        "an intact chain built by append_audit must verify as true"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// C2: Mutating one entry's `action` field after the chain is built must cause
///     `verify_chain` to return `false` — the tampered-content REJECT path.
#[test]
fn c2_tampered_action_field_returns_false() {
    let key = b"ratchet-test-key";
    let e1 = append_audit(key, &[], "skill-alpha", "install");
    let chain1 = vec![e1];
    let e2 = append_audit(key, &chain1, "skill-beta", "activate");
    let mut chain = chain1;
    chain.push(e2);

    // Tamper: change the action of the first entry after the chain is built.
    chain[0].action = "tampered-action".to_string();

    assert!(
        !verify_chain(key, &chain),
        "a chain with a mutated action field must not verify"
    );
}

// ---- C3 -----------------------------------------------------------------------

/// C3a: Removing an entry from a chain breaks the `prev_hash` link and must cause
///      `verify_chain` to return `false`.
#[test]
fn c3_removed_entry_breaks_link_returns_false() {
    let key = b"ratchet-test-key";
    let e1 = append_audit(key, &[], "skill-alpha", "install");
    let chain1 = vec![e1];
    let e2 = append_audit(key, &chain1, "skill-beta", "activate");
    let mut chain = chain1;
    chain.push(e2);
    let e3 = append_audit(key, &chain, "skill-gamma", "run");
    chain.push(e3);

    // Remove the middle entry — entry[1]'s prev_hash now mismatches entry[0].
    chain.remove(1);

    assert!(
        !verify_chain(key, &chain),
        "removing an entry must break the prev_hash link and fail verification"
    );
}

/// C3b: Reordering entries swaps their `prev_hash` links and must cause
///      `verify_chain` to return `false`.
#[test]
fn c3_reordered_entries_break_link_returns_false() {
    let key = b"ratchet-test-key";
    let e1 = append_audit(key, &[], "skill-alpha", "install");
    let chain1 = vec![e1];
    let e2 = append_audit(key, &chain1, "skill-beta", "activate");
    let mut chain = chain1;
    chain.push(e2);

    // Swap the two entries — now entry[0].prev_hash points to the genesis but
    // entry[0] is actually the second entry whose prev_hash references e1's hash.
    chain.swap(0, 1);

    assert!(
        !verify_chain(key, &chain),
        "reordering entries must break the prev_hash chain and fail verification"
    );
}
