//! Frozen tests for T-00.06 — "Define the State type".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_default_is_idle, c1_all_six_variants_exist, c1_copy_semantics, c1_partial_eq
//!   C2 -> c2_serde_json_round_trip, c2_default_is_idle_serde_context

use zira_proto::State;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_default_is_idle() {
    assert_eq!(State::default(), State::Idle);
}

#[test]
fn c1_all_six_variants_exist() {
    let variants = [
        State::Idle,
        State::Listening,
        State::Transcribing,
        State::Thinking,
        State::PlanReview,
        State::Speaking,
    ];
    assert_eq!(variants.len(), 6, "exactly six State variants must exist");
}

#[test]
fn c1_copy_semantics() {
    // Copy must be derived: assigning to another binding must not move the original.
    let a = State::Idle;
    let b = a;
    // Both bindings are still usable — this only compiles if State: Copy.
    assert_eq!(a, b);
}

#[test]
fn c1_partial_eq() {
    // PartialEq must be derived: same variant compares equal, different variants differ.
    assert_eq!(State::Idle, State::Idle);
    assert_eq!(State::Listening, State::Listening);
    assert_eq!(State::Transcribing, State::Transcribing);
    assert_eq!(State::Thinking, State::Thinking);
    assert_eq!(State::PlanReview, State::PlanReview);
    assert_eq!(State::Speaking, State::Speaking);

    assert_ne!(State::Idle, State::Listening);
    assert_ne!(State::Listening, State::Transcribing);
    assert_ne!(State::Transcribing, State::Thinking);
    assert_ne!(State::Thinking, State::PlanReview);
    assert_ne!(State::PlanReview, State::Speaking);
    assert_ne!(State::Speaking, State::Idle);
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_serde_json_round_trip() {
    let variants = [
        State::Idle,
        State::Listening,
        State::Transcribing,
        State::Thinking,
        State::PlanReview,
        State::Speaking,
    ];
    for variant in variants {
        let json = serde_json::to_string(&variant)
            .unwrap_or_else(|e| panic!("serialize {variant:?}: {e}"));
        let back: State = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("deserialize {json:?}: {e}"));
        assert_eq!(
            variant, back,
            "serde round-trip must preserve variant identity"
        );
    }
}

#[test]
fn c2_default_is_idle_serde_context() {
    // Confirm Default::default() returns Idle, and that the default value
    // round-trips correctly through serde_json.
    let default_state: State = Default::default();
    assert_eq!(default_state, State::Idle);
    let json = serde_json::to_string(&default_state).expect("serialize default State");
    let back: State = serde_json::from_str(&json).expect("deserialize default State");
    assert_eq!(back, State::Idle);
}
