//! Frozen tests for T-00.07 — "Define the payload types".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_all_six_structs_derive_clone,
//!          c1_all_six_structs_derive_serialize_deserialize
//!   C2 -> c2_segment_carries_emotion_and_text
//!   C3 -> c3_round_trip_transcript, c3_round_trip_audio_chunk,
//!          c3_round_trip_segment, c3_round_trip_viseme_frame,
//!          c3_round_trip_plan_summary, c3_round_trip_usage

use zira_proto::{AudioChunk, Emotion, PlanSummary, Segment, Transcript, Usage, VisemeFrame};

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_all_six_structs_derive_clone() {
    // Clone must be derived on every payload struct. We construct a populated instance
    // of each, call .clone(), and assert equality where PartialEq is available; the
    // important thing is that the call compiles.
    let transcript = Transcript {
        text: "hello world".to_string(),
    };
    let _t2 = transcript.clone();

    let chunk = AudioChunk {
        samples: vec![0.0_f32, 0.5, -0.5],
        sample_rate: 16_000,
        channels: 1,
    };
    let _c2 = chunk.clone();

    let segment = Segment {
        emotion: Emotion::Happy,
        text: "That works!".to_string(),
    };
    let _s2 = segment.clone();

    let frame = VisemeFrame {
        viseme: "aa".to_string(),
        weight: 0.8,
    };
    let _f2 = frame.clone();

    let plan = PlanSummary {
        description: "Refactor the parser".to_string(),
        steps: vec!["step 1".to_string(), "step 2".to_string()],
    };
    let _p2 = plan.clone();

    let usage = Usage {
        input_tokens: 42,
        output_tokens: 17,
    };
    let _u2 = usage.clone();
}

#[test]
fn c1_all_six_structs_derive_serialize_deserialize() {
    // Serialize/Deserialize must be derived. A successful to_string + from_str without
    // panic is sufficient to confirm both impls are present.
    let t = Transcript {
        text: "hi".to_string(),
    };
    let _: Transcript = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap();

    let c = AudioChunk {
        samples: vec![0.1],
        sample_rate: 44_100,
        channels: 2,
    };
    let _: AudioChunk = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();

    let s = Segment {
        emotion: Emotion::Calm,
        text: "steady".to_string(),
    };
    let _: Segment = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();

    let f = VisemeFrame {
        viseme: "ih".to_string(),
        weight: 0.4,
    };
    let _: VisemeFrame = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();

    let p = PlanSummary {
        description: "plan".to_string(),
        steps: vec![],
    };
    let _: PlanSummary = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();

    let u = Usage {
        input_tokens: 1,
        output_tokens: 2,
    };
    let _: Usage = serde_json::from_str(&serde_json::to_string(&u).unwrap()).unwrap();
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_segment_carries_emotion_and_text() {
    // Segment must have an `emotion: Emotion` field and a `text: String` field so
    // an emotion-tagged segment is directly representable.
    let seg = Segment {
        emotion: Emotion::Excited,
        text: "It compiles!".to_string(),
    };
    assert_eq!(seg.emotion, Emotion::Excited);
    assert_eq!(seg.text, "It compiles!");

    // Verify Neutral is also representable (the default emotion).
    let neutral_seg = Segment {
        emotion: Emotion::Neutral,
        text: String::new(),
    };
    assert_eq!(neutral_seg.emotion, Emotion::Neutral);
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_round_trip_transcript() {
    let original = Transcript {
        text: "cargo test passes".to_string(),
    };
    let json = serde_json::to_string(&original).expect("serialize Transcript");
    let restored: Transcript = serde_json::from_str(&json).expect("deserialize Transcript");
    assert_eq!(restored.text, original.text);
}

#[test]
fn c3_round_trip_audio_chunk() {
    let original = AudioChunk {
        samples: vec![0.0, 0.25, -0.25, 0.5, -0.5, 1.0, -1.0],
        sample_rate: 16_000,
        channels: 1,
    };
    let json = serde_json::to_string(&original).expect("serialize AudioChunk");
    let restored: AudioChunk = serde_json::from_str(&json).expect("deserialize AudioChunk");
    assert_eq!(restored.sample_rate, original.sample_rate);
    assert_eq!(restored.channels, original.channels);
    assert_eq!(restored.samples.len(), original.samples.len());
    for (a, b) in restored.samples.iter().zip(original.samples.iter()) {
        assert!(
            (a - b).abs() < f32::EPSILON,
            "sample mismatch: {a} vs {b}"
        );
    }
}

#[test]
fn c3_round_trip_segment() {
    let original = Segment {
        emotion: Emotion::Curious,
        text: "Should this be a struct or an enum?".to_string(),
    };
    let json = serde_json::to_string(&original).expect("serialize Segment");
    let restored: Segment = serde_json::from_str(&json).expect("deserialize Segment");
    assert_eq!(restored.emotion, original.emotion);
    assert_eq!(restored.text, original.text);
}

#[test]
fn c3_round_trip_viseme_frame() {
    let original = VisemeFrame {
        viseme: "ow".to_string(),
        weight: 0.65,
    };
    let json = serde_json::to_string(&original).expect("serialize VisemeFrame");
    let restored: VisemeFrame = serde_json::from_str(&json).expect("deserialize VisemeFrame");
    assert_eq!(restored.viseme, original.viseme);
    assert!(
        (restored.weight - original.weight).abs() < f32::EPSILON,
        "weight mismatch: {} vs {}",
        restored.weight,
        original.weight
    );
}

#[test]
fn c3_round_trip_plan_summary() {
    let original = PlanSummary {
        description: "Add serde to all payload types".to_string(),
        steps: vec![
            "Define struct".to_string(),
            "Derive Serialize/Deserialize".to_string(),
            "Write tests".to_string(),
        ],
    };
    let json = serde_json::to_string(&original).expect("serialize PlanSummary");
    let restored: PlanSummary = serde_json::from_str(&json).expect("deserialize PlanSummary");
    assert_eq!(restored.description, original.description);
    assert_eq!(restored.steps, original.steps);
}

#[test]
fn c3_round_trip_usage() {
    let original = Usage {
        input_tokens: 1_234,
        output_tokens: 567,
    };
    let json = serde_json::to_string(&original).expect("serialize Usage");
    let restored: Usage = serde_json::from_str(&json).expect("deserialize Usage");
    assert_eq!(restored.input_tokens, original.input_tokens);
    assert_eq!(restored.output_tokens, original.output_tokens);
}
