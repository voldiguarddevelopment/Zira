//! T-00.16 — transition table integration tests.
//!
//! C1: every valid (state, event) pair from PLAN.md §5 returns Some(expected).
//! C2: undefined pairs return None, never panic or produce a wrong state.
//! C3: this file is the repo-root integration test that exercises the full table.

use zira_core::next_state;
use zira_proto::{Event, State, Transcript, Usage};

// ── C1: valid transitions ────────────────────────────────────────────────────

/// Idle + WakeDetected → Listening  (example cited in C1)
#[test]
fn test_idle_wake_detected_to_listening() {
    assert_eq!(
        next_state(State::Idle, &Event::WakeDetected),
        Some(State::Listening),
    );
}

/// Listening + SpeechEnded → Transcribing
#[test]
fn test_listening_speech_ended_to_transcribing() {
    assert_eq!(
        next_state(State::Listening, &Event::SpeechEnded),
        Some(State::Transcribing),
    );
}

/// Transcribing + TranscriptReady → Thinking
#[test]
fn test_transcribing_transcript_ready_to_thinking() {
    let ev = Event::TranscriptReady(Transcript { text: "hello".into() });
    assert_eq!(next_state(State::Transcribing, &ev), Some(State::Thinking));
}

/// Thinking + SpeakRequest → Speaking  (response tokens arrive)
#[test]
fn test_thinking_speak_request_to_speaking() {
    assert_eq!(
        next_state(State::Thinking, &Event::SpeakRequest),
        Some(State::Speaking),
    );
}

/// Thinking + PlanReady → PlanReview  (example cited in C1)
#[test]
fn test_thinking_plan_ready_to_plan_review() {
    assert_eq!(
        next_state(State::Thinking, &Event::PlanReady),
        Some(State::PlanReview),
    );
}

/// PlanReview + TurnStarted → Thinking  (user approves; bridge switches to AcceptEdits)
#[test]
fn test_plan_review_turn_started_to_thinking() {
    assert_eq!(
        next_state(State::PlanReview, &Event::TurnStarted),
        Some(State::Thinking),
    );
}

/// PlanReview + Error → Idle  (user rejects; abort back to Idle)
#[test]
fn test_plan_review_error_to_idle() {
    let ev = Event::Error("user rejected plan".into());
    assert_eq!(next_state(State::PlanReview, &ev), Some(State::Idle));
}

/// Speaking + TurnComplete → Idle  (utterance complete)
#[test]
fn test_speaking_turn_complete_to_idle() {
    let ev = Event::TurnComplete(Usage { input_tokens: 10, output_tokens: 20 });
    assert_eq!(next_state(State::Speaking, &ev), Some(State::Idle));
}

/// Speaking + BargeIn → Listening  (example cited in C1)
#[test]
fn test_speaking_barge_in_to_listening() {
    assert_eq!(
        next_state(State::Speaking, &Event::BargeIn),
        Some(State::Listening),
    );
}

/// Thinking + BargeIn → Listening  (interrupt mid-turn)
#[test]
fn test_thinking_barge_in_to_listening() {
    assert_eq!(
        next_state(State::Thinking, &Event::BargeIn),
        Some(State::Listening),
    );
}

// ── C2: undefined pairs return None ─────────────────────────────────────────

/// Idle has no transition on SpeechEnded.
#[test]
fn test_undefined_idle_speech_ended_is_none() {
    assert_eq!(next_state(State::Idle, &Event::SpeechEnded), None);
}

/// Idle has no transition on BargeIn.
#[test]
fn test_undefined_idle_barge_in_is_none() {
    assert_eq!(next_state(State::Idle, &Event::BargeIn), None);
}

/// Idle has no transition on TurnComplete.
#[test]
fn test_undefined_idle_turn_complete_is_none() {
    let ev = Event::TurnComplete(Usage { input_tokens: 0, output_tokens: 0 });
    assert_eq!(next_state(State::Idle, &ev), None);
}

/// Listening has no transition on WakeDetected.
#[test]
fn test_undefined_listening_wake_detected_is_none() {
    assert_eq!(next_state(State::Listening, &Event::WakeDetected), None);
}

/// Transcribing has no transition on BargeIn.
#[test]
fn test_undefined_transcribing_barge_in_is_none() {
    assert_eq!(next_state(State::Transcribing, &Event::BargeIn), None);
}

/// PlanReview has no transition on SpeechEnded.
#[test]
fn test_undefined_plan_review_speech_ended_is_none() {
    assert_eq!(next_state(State::PlanReview, &Event::SpeechEnded), None);
}

/// Speaking has no transition on PlanReady.
#[test]
fn test_undefined_speaking_plan_ready_is_none() {
    assert_eq!(next_state(State::Speaking, &Event::PlanReady), None);
}
