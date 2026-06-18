//! Frozen tests for T-04.10 — "Define the GateDecision type".
//!
//! Criterion → test mapping:
//!   C1 -> c1_is_allowed_true_for_allow
//!   C1 -> c1_is_allowed_false_for_deny
//!   C1 -> c1_derives_debug_clone_partialeq
//!   C2 -> c2_allow_is_allowed_returns_true
//!   C2 -> c2_deny_is_allowed_returns_false
//!   C2 -> c2_deny_carries_capability_and_reason
//!   C3 -> c3_display_allow_is_nonempty
//!   C3 -> c3_display_deny_is_nonempty
//!   C3 -> c3_display_allow_and_deny_are_distinct

use zira_skills::GateDecision;

// ---- C1 -------------------------------------------------------------------------------

/// C1: `is_allowed()` returns `true` for the `Allow` variant.
#[test]
fn c1_is_allowed_true_for_allow() {
    let allow = GateDecision::Allow;
    assert!(
        allow.is_allowed(),
        "Allow.is_allowed() must return true"
    );
}

/// C1: `is_allowed()` returns `false` for the `Deny` variant.
#[test]
fn c1_is_allowed_false_for_deny() {
    let deny = GateDecision::Deny {
        capability: "shell_exec".to_string(),
        reason: "prohibited by the Zira constitution".to_string(),
    };
    assert!(
        !deny.is_allowed(),
        "Deny.is_allowed() must return false"
    );
}

/// C1: `GateDecision` derives `Debug`, `Clone`, and `PartialEq` — all must be usable
///     without a `mut` binding and without any additional impl.
#[test]
fn c1_derives_debug_clone_partialeq() {
    let original = GateDecision::Allow;
    let cloned: GateDecision = original.clone();
    assert_eq!(original, cloned, "cloned Allow must equal the original");

    let deny = GateDecision::Deny {
        capability: "harm".to_string(),
        reason: "constitution forbids harm".to_string(),
    };
    let deny_clone = deny.clone();
    assert_eq!(deny, deny_clone, "cloned Deny must equal the original");

    assert_ne!(original, deny, "Allow and Deny must not be equal");

    // Debug: the format must succeed (non-empty output is checked in C3)
    let _ = format!("{:?}", original);
    let _ = format!("{:?}", deny);
}

// ---- C2 -------------------------------------------------------------------------------

/// C2: `Allow.is_allowed()` is `true` — the affirmative path.
#[test]
fn c2_allow_is_allowed_returns_true() {
    assert!(
        GateDecision::Allow.is_allowed(),
        "Allow must be allowed"
    );
}

/// C2: a `Deny` value's `is_allowed()` is `false` — the denial path.
#[test]
fn c2_deny_is_allowed_returns_false() {
    let decision = GateDecision::Deny {
        capability: "arbitrary_shell".to_string(),
        reason: "violates constitution rule 4".to_string(),
    };
    assert!(!decision.is_allowed(), "Deny must not be allowed");
}

/// C2: a `Deny` value carries the exact offending capability and reason strings
///     passed at construction.
#[test]
fn c2_deny_carries_capability_and_reason() {
    let cap = "network_egress".to_string();
    let rsn = "constitution prohibits unrestricted outbound access".to_string();
    let deny = GateDecision::Deny {
        capability: cap.clone(),
        reason: rsn.clone(),
    };
    match deny {
        GateDecision::Deny { capability, reason } => {
            assert_eq!(capability, cap, "Deny must carry the offending capability");
            assert_eq!(reason, rsn, "Deny must carry the denial reason");
        }
        GateDecision::Allow => panic!("expected Deny, got Allow"),
    }
}

// ---- C3 -------------------------------------------------------------------------------

/// C3: `Display` for `Allow` produces a non-empty string.
#[test]
fn c3_display_allow_is_nonempty() {
    let allow = GateDecision::Allow;
    let s = format!("{}", allow);
    assert!(!s.is_empty(), "Display for Allow must not be empty");
}

/// C3: `Display` for `Deny` produces a non-empty string.
#[test]
fn c3_display_deny_is_nonempty() {
    let deny = GateDecision::Deny {
        capability: "shell_exec".to_string(),
        reason: "prohibited".to_string(),
    };
    let s = format!("{}", deny);
    assert!(!s.is_empty(), "Display for Deny must not be empty");
}

/// C3: the `Display` output of `Allow` and `Deny` are distinct — every variant
///     has its own format, so no single format string can survive mutation to cover
///     both variants simultaneously.
#[test]
fn c3_display_allow_and_deny_are_distinct() {
    let allow_str = format!("{}", GateDecision::Allow);
    let deny_str = format!(
        "{}",
        GateDecision::Deny {
            capability: "shell_exec".to_string(),
            reason: "prohibited".to_string(),
        }
    );
    assert_ne!(
        allow_str, deny_str,
        "Display for Allow and Deny must produce distinct strings"
    );
}
