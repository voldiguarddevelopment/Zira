//! Frozen tests for T-05.03 — "Verify the plan-review transition".
//!
//! Criterion coverage:
//!
//!   C1 -> test_accept_lands_in_thinking
//!   C2 -> test_reject_lands_in_idle

use zira_core::{plan_review_next_state, PlanDecision};
use zira_proto::{PlanSummary, State};

fn make_plan() -> PlanSummary {
    PlanSummary {
        description: "test plan".into(),
        steps: vec!["step one".into()],
    }
}

/// C1: next_state(State::PlanReview, &review_plan(&plan, PlanDecision::Accept)) == Some(State::Thinking)
#[test]
fn test_accept_lands_in_thinking() {
    let plan = make_plan();
    assert_eq!(
        plan_review_next_state(&plan, PlanDecision::Accept),
        Some(State::Thinking),
    );
}

/// C2: next_state(State::PlanReview, &review_plan(&plan, PlanDecision::Reject)) == Some(State::Idle)
#[test]
fn test_reject_lands_in_idle() {
    let plan = make_plan();
    assert_eq!(
        plan_review_next_state(&plan, PlanDecision::Reject),
        Some(State::Idle),
    );
}
