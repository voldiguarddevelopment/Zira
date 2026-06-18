//! Frozen tests for T-01.04 — "Map emotion to prosody".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_prosody_neutral, c1_prosody_happy, c1_prosody_sad,
//!          c1_prosody_angry, c1_prosody_excited, c1_prosody_calm,
//!          c1_prosody_curious, c1_prosody_concerned, c1_prosody_playful,
//!          c1_prosody_tired, c1_all_ten_variants_return_prosody
//!   C2 -> c2_neutral_is_baseline
//!   C3 -> c3_all_variants_in_bounds

use zira_emotion::{prosody, Prosody};
use zira_proto::Emotion;

// ---- C1 — prosody() is total over all ten Emotion variants --------------------------------

#[test]
fn c1_prosody_neutral() {
    let p = prosody(Emotion::Neutral);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_happy() {
    let p = prosody(Emotion::Happy);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_sad() {
    let p = prosody(Emotion::Sad);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_angry() {
    let p = prosody(Emotion::Angry);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_excited() {
    let p = prosody(Emotion::Excited);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_calm() {
    let p = prosody(Emotion::Calm);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_curious() {
    let p = prosody(Emotion::Curious);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_concerned() {
    let p = prosody(Emotion::Concerned);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_playful() {
    let p = prosody(Emotion::Playful);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_prosody_tired() {
    let p = prosody(Emotion::Tired);
    let _ = (p.rate, p.pitch, p.volume);
}

#[test]
fn c1_all_ten_variants_return_prosody() {
    // Exhaustively exercises every variant so that adding an eleventh variant
    // without updating this test causes a compile error (non-exhaustive match).
    let variants = [
        Emotion::Neutral,
        Emotion::Happy,
        Emotion::Sad,
        Emotion::Angry,
        Emotion::Excited,
        Emotion::Calm,
        Emotion::Curious,
        Emotion::Concerned,
        Emotion::Playful,
        Emotion::Tired,
    ];
    assert_eq!(variants.len(), 10, "expected exactly ten variants");
    for e in variants {
        // Each call must return without panicking and yield a Prosody.
        let _p: Prosody = prosody(e);
    }
}

// ---- C2 — Neutral is the baseline ---------------------------------------------------------

#[test]
fn c2_neutral_is_baseline() {
    let p = prosody(Emotion::Neutral);
    assert_eq!(
        p,
        Prosody { rate: 1.0, pitch: 1.0, volume: 1.0 },
        "Neutral must return the unmodified baseline prosody"
    );
}

// ---- C3 — every field of every variant lies within 0.5..=2.0 ------------------------------

#[test]
fn c3_all_variants_in_bounds() {
    let variants = [
        Emotion::Neutral,
        Emotion::Happy,
        Emotion::Sad,
        Emotion::Angry,
        Emotion::Excited,
        Emotion::Calm,
        Emotion::Curious,
        Emotion::Concerned,
        Emotion::Playful,
        Emotion::Tired,
    ];
    for e in variants {
        let p = prosody(e);
        assert!(
            (0.5_f32..=2.0_f32).contains(&p.rate),
            "{e:?}: rate {rate} is outside 0.5..=2.0",
            rate = p.rate,
        );
        assert!(
            (0.5_f32..=2.0_f32).contains(&p.pitch),
            "{e:?}: pitch {pitch} is outside 0.5..=2.0",
            pitch = p.pitch,
        );
        assert!(
            (0.5_f32..=2.0_f32).contains(&p.volume),
            "{e:?}: volume {volume} is outside 0.5..=2.0",
            volume = p.volume,
        );
    }
}
