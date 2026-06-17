//! Frozen tests for T-00.08 — "Define the Event type".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_all_fifteen_variants_exist
//!   C2 -> c2_payload_bearing_variants_carry_typed_payloads,
//!          c2_event_derives_clone,
//!          c2_event_derives_serialize_deserialize
//!   C3 -> c3_round_trip_payload_bearing_variant,
//!          c3_round_trip_unit_variant

use zira_proto::{Event, Segment, Transcript, Usage, VisemeFrame};
use zira_proto::Emotion;

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_all_fifteen_variants_exist() {
    // Construct every variant. Fails to compile if any variant is absent or misnamed.
    let _variants: &[Event] = &[
        Event::WakeDetected,
        Event::SpeechStarted,
        Event::SpeechEnded,
        Event::AudioChunk,
        Event::TranscriptReady(Transcript { text: String::new() }),
        Event::TurnStarted,
        Event::TextDelta,
        Event::EmotionSegment(Segment { emotion: Emotion::Neutral, text: String::new() }),
        Event::PlanReady,
        Event::SpeakRequest,
        Event::VisemeFrame(VisemeFrame { viseme: String::new(), weight: 0.0 }),
        Event::ExpressionChange,
        Event::BargeIn,
        Event::TurnComplete(Usage { input_tokens: 0, output_tokens: 0 }),
        Event::Error(String::new()),
    ];
    assert_eq!(_variants.len(), 15, "exactly fifteen Event variants must exist");
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_payload_bearing_variants_carry_typed_payloads() {
    // Each payload-bearing variant must accept exactly the specified payload type.
    // The match extracts the inner value to confirm the type at compile time.
    let tr = Event::TranscriptReady(Transcript { text: "hello".to_string() });
    if let Event::TranscriptReady(t) = &tr {
        assert_eq!(t.text, "hello");
    } else {
        panic!("TranscriptReady did not match");
    }

    let es = Event::EmotionSegment(Segment {
        emotion: Emotion::Happy,
        text: "joyful".to_string(),
    });
    if let Event::EmotionSegment(s) = &es {
        assert_eq!(s.emotion, Emotion::Happy);
        assert_eq!(s.text, "joyful");
    } else {
        panic!("EmotionSegment did not match");
    }

    let vf = Event::VisemeFrame(VisemeFrame {
        viseme: "aa".to_string(),
        weight: 0.9,
    });
    if let Event::VisemeFrame(f) = &vf {
        assert_eq!(f.viseme, "aa");
        assert!((f.weight - 0.9_f32).abs() < f32::EPSILON);
    } else {
        panic!("VisemeFrame did not match");
    }

    let tc = Event::TurnComplete(Usage {
        input_tokens: 10,
        output_tokens: 20,
    });
    if let Event::TurnComplete(u) = &tc {
        assert_eq!(u.input_tokens, 10);
        assert_eq!(u.output_tokens, 20);
    } else {
        panic!("TurnComplete did not match");
    }

    let err = Event::Error("something went wrong".to_string());
    if let Event::Error(msg) = &err {
        assert_eq!(msg, "something went wrong");
    } else {
        panic!("Error did not match");
    }
}

#[test]
fn c2_event_derives_clone() {
    // Clone must be derived on Event; cloning a payload-bearing variant preserves data.
    let original = Event::TranscriptReady(Transcript {
        text: "clone me".to_string(),
    });
    let cloned = original.clone();
    if let (Event::TranscriptReady(o), Event::TranscriptReady(c)) = (&original, &cloned) {
        assert_eq!(o.text, c.text);
    } else {
        panic!("clone changed the variant discriminant");
    }

    // Unit variant clone.
    let wake = Event::WakeDetected;
    let _wake2 = wake.clone();
}

#[test]
fn c2_event_derives_serialize_deserialize() {
    // Serialize/Deserialize must be derived. Round-trip each payload-bearing variant.
    let variants: Vec<Event> = vec![
        Event::TranscriptReady(Transcript { text: "serialize me".to_string() }),
        Event::EmotionSegment(Segment { emotion: Emotion::Sad, text: "sorrow".to_string() }),
        Event::VisemeFrame(VisemeFrame { viseme: "ow".to_string(), weight: 0.5 }),
        Event::TurnComplete(Usage { input_tokens: 7, output_tokens: 3 }),
        Event::Error("oops".to_string()),
        Event::WakeDetected,
        Event::SpeechStarted,
        Event::SpeechEnded,
        Event::AudioChunk,
        Event::TurnStarted,
        Event::TextDelta,
        Event::PlanReady,
        Event::SpeakRequest,
        Event::ExpressionChange,
        Event::BargeIn,
    ];
    for event in &variants {
        let json = serde_json::to_string(event)
            .unwrap_or_else(|e| panic!("serialize {event:?}: {e}"));
        let _back: Event = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("deserialize {json:?}: {e}"));
    }
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_round_trip_payload_bearing_variant() {
    // A payload-bearing variant must survive a serde JSON round-trip with payload intact.
    let original = Event::TranscriptReady(Transcript {
        text: "round trip complete".to_string(),
    });
    let json = serde_json::to_string(&original).expect("serialize Event::TranscriptReady");
    let restored: Event = serde_json::from_str(&json).expect("deserialize Event::TranscriptReady");

    if let Event::TranscriptReady(t) = restored {
        assert_eq!(t.text, "round trip complete");
    } else {
        panic!("serde round-trip changed the variant to something other than TranscriptReady");
    }
}

#[test]
fn c3_round_trip_unit_variant() {
    // A unit variant must survive a serde JSON round-trip unchanged.
    let original = Event::WakeDetected;
    let json = serde_json::to_string(&original).expect("serialize Event::WakeDetected");
    let restored: Event = serde_json::from_str(&json).expect("deserialize Event::WakeDetected");

    // We can only verify the discriminant matches by matching on the variant.
    assert!(
        matches!(restored, Event::WakeDetected),
        "serde round-trip changed WakeDetected to a different variant"
    );
}
