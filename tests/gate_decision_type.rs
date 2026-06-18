use zira_skills::GateDecision;

// C1 + C2: Allow.is_allowed() must be true
#[test]
fn test_allow_is_allowed() {
    let d = GateDecision::Allow;
    assert!(d.is_allowed());
}

// C1 + C2: Deny.is_allowed() must be false
#[test]
fn test_deny_is_not_allowed() {
    let d = GateDecision::Deny {
        capability: "net.fetch".to_string(),
        reason: "not permitted by constitution".to_string(),
    };
    assert!(!d.is_allowed());
}

// C2: Deny carries the exact capability and reason strings
#[test]
fn test_deny_carries_capability_and_reason() {
    let cap = "fs.write";
    let reason = "outside allowed roots";
    let d = GateDecision::Deny {
        capability: cap.to_string(),
        reason: reason.to_string(),
    };
    if let GateDecision::Deny { capability, reason: r } = d {
        assert_eq!(capability, cap);
        assert_eq!(r, reason);
    } else {
        panic!("expected Deny variant");
    }
}

// C1: derives Clone and PartialEq
#[test]
fn test_clone_and_partial_eq() {
    let allow = GateDecision::Allow;
    assert_eq!(allow.clone(), GateDecision::Allow);

    let deny = GateDecision::Deny {
        capability: "exec".to_string(),
        reason: "blocked".to_string(),
    };
    assert_eq!(deny.clone(), deny);
    assert_ne!(allow, deny);
}

// C3: Display output is non-empty for both variants and they are distinct
#[test]
fn test_display_non_empty_and_distinct() {
    let allow = GateDecision::Allow;
    let deny = GateDecision::Deny {
        capability: "net.fetch".to_string(),
        reason: "not permitted".to_string(),
    };

    let allow_str = format!("{allow}");
    let deny_str = format!("{deny}");

    assert!(!allow_str.is_empty(), "Allow Display must be non-empty");
    assert!(!deny_str.is_empty(), "Deny Display must be non-empty");
    assert_ne!(allow_str, deny_str, "Allow and Deny Display must differ");
}

// C3: Display exercises the Deny fields so mutation of format! operands is caught
#[test]
fn test_deny_display_contains_fields() {
    let cap = "capability-x";
    let reason = "reason-y";
    let deny = GateDecision::Deny {
        capability: cap.to_string(),
        reason: reason.to_string(),
    };
    let s = format!("{deny}");
    assert!(s.contains(cap), "Deny Display must include the capability");
    assert!(s.contains(reason), "Deny Display must include the reason");
}
