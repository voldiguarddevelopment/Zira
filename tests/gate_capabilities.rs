//! Frozen tests for T-04.09 — "Gate capabilities against the constitution".
//!
//! Criterion → test mapping:
//!   C1 -> c1_allow_when_all_capabilities_sanctioned
//!   C2 -> c2_deny_forbidden_capability
//!   C3 -> c3_deny_unknown_capability

use zira_config::Constitution;
use zira_skills::{gate_capabilities, GateDecision, SkillManifest};

fn manifest(capabilities: Vec<&str>) -> SkillManifest {
    SkillManifest {
        name: "test-skill".to_string(),
        version: "0.1.0".to_string(),
        entry: "main".to_string(),
        capabilities: capabilities.into_iter().map(str::to_string).collect(),
        allowed_roots: vec![],
    }
}

// ---- C1 -------------------------------------------------------------------------------

/// C1: a manifest whose capabilities are all constitution-sanctioned returns Allow.
///
/// "coding" appears in the descriptive (allow-type) rules of the default constitution
/// ("voice-driven coding helper", "coding and general assistant") and is not mentioned
/// only in prohibitive rules — it is affirmatively sanctioned.
#[test]
fn c1_allow_when_all_capabilities_sanctioned() {
    let c = Constitution::load_default();
    let m = manifest(vec!["coding"]);
    assert_eq!(
        gate_capabilities(&c, &m),
        GateDecision::Allow,
        "a manifest with only sanctioned capabilities must return Allow"
    );
}

// ---- C2 -------------------------------------------------------------------------------

/// C2: a manifest declaring a capability the constitution forbids returns Deny naming
///     the offending capability.
///
/// "harm" appears in rule 4: "must refuse requests that would cause harm …" — a
/// prohibitive rule.  A capability that appears only in prohibitive rules is forbidden.
#[test]
fn c2_deny_forbidden_capability() {
    let c = Constitution::load_default();
    let m = manifest(vec!["harm"]);
    let result = gate_capabilities(&c, &m);
    match result {
        GateDecision::Deny { capability, .. } => {
            assert_eq!(
                capability, "harm",
                "Deny must name the offending capability"
            );
        }
        GateDecision::Allow => panic!("expected Deny for forbidden capability, got Allow"),
    }
}

// ---- C3 -------------------------------------------------------------------------------

/// C3: a manifest declaring an unknown capability (matched by no constitution rule)
///     returns Deny — never Allow — enforcing the default-deny invariant.
#[test]
fn c3_deny_unknown_capability() {
    let c = Constitution::load_default();
    let m = manifest(vec!["launch_missiles"]);
    let result = gate_capabilities(&c, &m);
    assert_ne!(
        result,
        GateDecision::Allow,
        "an unknown capability must never return Allow"
    );
    match result {
        GateDecision::Deny { capability, .. } => {
            assert_eq!(
                capability, "launch_missiles",
                "Deny must name the unknown capability"
            );
        }
        GateDecision::Allow => unreachable!(),
    }
}
