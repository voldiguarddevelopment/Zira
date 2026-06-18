use zira_core::{review_plan, PlanDecision};
use zira_proto::{Event, PlanSummary};

fn plan_a() -> PlanSummary {
    PlanSummary {
        description: "Alpha plan".into(),
        steps: vec!["step one".into()],
    }
}

fn plan_b() -> PlanSummary {
    PlanSummary {
        description: "Beta plan with different content".into(),
        steps: vec!["step one".into(), "step two".into()],
    }
}

#[test]
fn accept_returns_turn_started() {
    let event = review_plan(&plan_a(), PlanDecision::Accept);
    assert!(
        matches!(event, Event::TurnStarted),
        "expected Event::TurnStarted, got {:?}",
        event
    );
}

#[test]
fn reject_returns_error() {
    let event = review_plan(&plan_a(), PlanDecision::Reject);
    assert!(
        matches!(event, Event::Error(_)),
        "expected Event::Error(_), got {:?}",
        event
    );
}

#[test]
fn decision_independent_of_plan_content() {
    let accept_a = review_plan(&plan_a(), PlanDecision::Accept);
    let accept_b = review_plan(&plan_b(), PlanDecision::Accept);
    assert!(
        matches!(accept_a, Event::TurnStarted) && matches!(accept_b, Event::TurnStarted),
        "Accept over two different plans must both yield TurnStarted"
    );

    let reject_a = review_plan(&plan_a(), PlanDecision::Reject);
    let reject_b = review_plan(&plan_b(), PlanDecision::Reject);
    assert!(
        matches!(reject_a, Event::Error(_)) && matches!(reject_b, Event::Error(_)),
        "Reject over two different plans must both yield Error(_)"
    );
}
