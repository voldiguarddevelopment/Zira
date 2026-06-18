//! Frozen tests for T-03.10 — "Drive the avatar sink".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_new_driver_starts_resting, c1_apply_emotion_sets_expression,
//!          c1_apply_viseme_sets_mouth, c1_state_returns_ref
//!   C2 -> c2_on_emotion_returns_expression_change,
//!          c2_on_emotion_updates_expression
//!   C3 -> c3_emotion_then_viseme_sequence

use zira_avatar::{expression_for, AvatarDriver, AvatarState, Viseme};
use zira_proto::{Emotion, Event};

// ---- C1 — AvatarDriver shape, initial state, and mutating accessors ----------------

#[test]
fn c1_new_driver_starts_resting() {
    let driver = AvatarDriver::new();
    assert_eq!(
        driver.state(),
        &AvatarState::resting(),
        "a freshly constructed AvatarDriver must hold AvatarState::resting()"
    );
}

#[test]
fn c1_apply_emotion_sets_expression() {
    let mut driver = AvatarDriver::new();
    driver.apply_emotion(Emotion::Happy);
    assert_eq!(
        driver.state().expression,
        expression_for(Emotion::Happy),
        "apply_emotion(Happy) must set state().expression to expression_for(Happy)"
    );
}

#[test]
fn c1_apply_viseme_sets_mouth() {
    let mut driver = AvatarDriver::new();
    driver.apply_viseme(Viseme::A);
    assert_eq!(
        driver.state().mouth,
        Viseme::A,
        "apply_viseme(A) must set state().mouth to Viseme::A"
    );
}

#[test]
fn c1_state_returns_ref() {
    let driver = AvatarDriver::new();
    // state() must return a shared reference — the borrow must compile and alias
    // the internal field (verifiable because two calls return identical values).
    let s1 = driver.state();
    let s2 = driver.state();
    assert_eq!(s1, s2, "state() must return a reference to the internal AvatarState");
}

// ---- C2 — on_emotion returns Event::ExpressionChange and updates expression ---------

#[test]
fn c2_on_emotion_returns_expression_change() {
    let mut driver = AvatarDriver::new();
    let event = driver.on_emotion(Emotion::Sad);
    assert!(
        matches!(event, Event::ExpressionChange),
        "on_emotion must return Event::ExpressionChange; got {event:?}"
    );
}

#[test]
fn c2_on_emotion_updates_expression() {
    let mut driver = AvatarDriver::new();
    driver.on_emotion(Emotion::Excited);
    assert_eq!(
        driver.state().expression,
        expression_for(Emotion::Excited),
        "after on_emotion(Excited), state().expression must equal expression_for(Excited)"
    );
}

// ---- C3 — integration: emotion then viseme sequence tracks latest state in order ----

#[test]
fn c3_emotion_then_viseme_sequence() {
    let mut driver = AvatarDriver::new();

    // Apply an emotion first; expression should reflect it.
    driver.apply_emotion(Emotion::Playful);
    assert_eq!(
        driver.state().expression,
        expression_for(Emotion::Playful),
        "after apply_emotion(Playful), expression must equal expression_for(Playful)"
    );

    // Feed a sequence of visemes; each call must update mouth to the latest shape.
    let visemes = [Viseme::A, Viseme::I, Viseme::U, Viseme::E, Viseme::O, Viseme::Sil];
    for v in visemes {
        driver.apply_viseme(v);
        assert_eq!(
            driver.state().mouth,
            v,
            "after apply_viseme({v:?}), state().mouth must equal {v:?}"
        );
        // Expression must be unchanged by a viseme update.
        assert_eq!(
            driver.state().expression,
            expression_for(Emotion::Playful),
            "apply_viseme must not disturb state().expression"
        );
    }

    // Apply a second emotion; expression updates, mouth stays at last viseme.
    driver.apply_emotion(Emotion::Calm);
    assert_eq!(
        driver.state().expression,
        expression_for(Emotion::Calm),
        "after second apply_emotion(Calm), expression must equal expression_for(Calm)"
    );
    assert_eq!(
        driver.state().mouth,
        Viseme::Sil,
        "apply_emotion must not disturb state().mouth (last viseme was Sil)"
    );
}
