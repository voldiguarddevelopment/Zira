//! Frozen tests for T-04.09 — Gate capabilities against the constitution.
//!
//! Criterion → test mapping:
//!   C1 -> test_allow_when_all_capabilities_sanctioned
//!   C2 -> test_deny_names_forbidden_capability
//!   C3 -> test_deny_unknown_capability_default_deny
//!
//! The embedded constitution (crates/zira-config/src/constitution.txt) provides
//! the rule set used in every test:
//!
//!   Sanctioned (non-prohibitive rule mentions it):
//!     "coding" — appears in two non-prohibitive rules.
//!
//!   Forbidden (prohibitive rule mentions it):
//!     "impersonate" — appears in "Zira must never impersonate a human …".
//!
//!   Unknown (no rule mentions it at all):
//!     "quantum-teleportation" — absent from all rules → default-deny.

use zira_config::Constitution;
use zira_skills::{gate_capabilities, GateDecision, SkillManifest};

fn manifest_with_caps(caps: Vec<&str>) -> SkillManifest {
    SkillManifest {
        name: "test-skill".to_string(),
        version: "0.1.0".to_string(),
        entry: "skill".to_string(),
        capabilities: caps.into_iter().map(|c| c.to_string()).collect(),
        allowed_roots: vec![],
    }
}

// ---- C1 -------------------------------------------------------------------------------

/// C1: Every capability is named by a non-prohibitive constitution rule → Allow.
///
/// "coding" appears in the embedded constitution as "coding helper" (rule 1) and
/// "coding and general assistant" (rule 7).  Neither rule is prohibitive, so the
/// capability is sanctioned and the gate must return Allow.
#[test]
fn test_allow_when_all_capabilities_sanctioned() {
    let constitution = Constitution::load_default();
    let manifest = manifest_with_caps(vec!["coding"]);
    let decision = gate_capabilities(&constitution, &manifest);
    assert!(
        decision.is_allowed(),
        "expected Allow for sanctioned capability 'coding'; got: {decision:?}",
    );
}

// ---- C2 -------------------------------------------------------------------------------

/// C2: A capability that a prohibitive rule names is denied, and Deny carries the
///     offending capability string.
///
/// "impersonate" appears in "Zira must never impersonate a human …".  The word "never"
/// makes the rule prohibitive, so gate_capabilities must return
/// `GateDecision::Deny { capability: "impersonate", .. }`.
#[test]
fn test_deny_names_forbidden_capability() {
    let constitution = Constitution::load_default();
    let manifest = manifest_with_caps(vec!["impersonate"]);
    let decision = gate_capabilities(&constitution, &manifest);
    match decision {
        GateDecision::Deny { capability, .. } => {
            assert_eq!(
                capability, "impersonate",
                "Deny must name the offending capability",
            );
        }
        GateDecision::Allow => {
            panic!("expected Deny for forbidden capability 'impersonate'; got Allow");
        }
    }
}

// ---- C3 -------------------------------------------------------------------------------

/// C3: A capability that no constitution rule mentions is denied (default-deny).
///
/// "quantum-teleportation" appears in no rule, so the gate must not allow it even
/// though no rule explicitly forbids it.  Default-deny: unlisted means denied.
#[test]
fn test_deny_unknown_capability_default_deny() {
    let constitution = Constitution::load_default();
    let manifest = manifest_with_caps(vec!["quantum-teleportation"]);
    let decision = gate_capabilities(&constitution, &manifest);
    assert!(
        !decision.is_allowed(),
        "unknown capability must be denied by default; got: {decision:?}",
    );
}
