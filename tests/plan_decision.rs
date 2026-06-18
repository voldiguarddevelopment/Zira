use zira_core::PlanDecision;

/// C1: PlanDecision has exactly Accept and Reject, with Debug+Clone+Copy+PartialEq derives.
#[test]
fn test_plan_decision_derives() {
    let accept = PlanDecision::Accept;
    let reject = PlanDecision::Reject;

    // Copy: can use after assignment without moving
    let accept2 = accept;
    let reject2 = reject;
    let _ = accept;
    let _ = reject;

    // Clone: explicit clone round-trips
    assert_eq!(accept2.clone(), PlanDecision::Accept);
    assert_eq!(reject2.clone(), PlanDecision::Reject);

    // PartialEq: each variant equals itself
    assert_eq!(accept2, PlanDecision::Accept);
    assert_eq!(reject2, PlanDecision::Reject);

    // Debug: both variants produce non-empty strings
    assert!(!format!("{:?}", accept2).is_empty());
    assert!(!format!("{:?}", reject2).is_empty());
}

/// C2: Accept and Reject are distinguishable (compare unequal).
#[test]
fn test_plan_decision_distinguishable() {
    assert_ne!(PlanDecision::Accept, PlanDecision::Reject);
}
