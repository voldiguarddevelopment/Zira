//! T-01.16 — earshot VAD over committed 16 kHz fixtures (no model, CI-portable).
//!   C1 -> c1_implements_vad_gate
//!   C2 -> c2_scan_emits_speech_boundaries
//!   C3 -> c3_speech_starts_silence_does_not
//!   C4 -> c4_voiced_ratio_separates_speech_from_silence

use zira_core::VadGate;
use zira_proto::Event;
use zira_voice::EarshotVad;

fn load(name: &str) -> Vec<i16> {
    let path = format!("{}/tests/fixtures/vad/{name}", env!("CARGO_MANIFEST_DIR"));
    let mut r = hound::WavReader::open(path).expect("fixture wav");
    r.samples::<i16>().map(|s| s.unwrap()).collect()
}

#[test]
fn c2_scan_emits_speech_boundaries() {
    let mut vad = EarshotVad::new();
    let events = vad.scan_16khz(&load("speech16.wav"));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::SpeechStarted)),
        "speech must yield at least one SpeechStarted; got {events:?}"
    );
}

#[test]
fn c3_speech_starts_silence_does_not() {
    let started = |name: &str| {
        EarshotVad::new()
            .scan_16khz(&load(name))
            .iter()
            .filter(|e| matches!(e, Event::SpeechStarted))
            .count()
    };
    assert!(started("speech16.wav") >= 1, "speech must start");
    assert_eq!(started("silence.wav"), 0, "silence must never start speech");
}

#[test]
fn c4_voiced_ratio_separates_speech_from_silence() {
    let speech = EarshotVad::new().voiced_ratio(&load("speech16.wav"));
    let silence = EarshotVad::new().voiced_ratio(&load("silence.wav"));
    assert!(speech >= 0.6, "speech voiced ratio {speech} should be >= 0.6");
    assert_eq!(silence, 0.0, "silence voiced ratio must be 0");
}

#[tokio::test]
async fn c1_implements_vad_gate() {
    let mut gate = EarshotVad::new().with_audio(load("speech16.wav"));
    // Pull boundaries until the first SpeechStarted appears within a bounded number of pulls.
    let mut started = false;
    for _ in 0..400 {
        if matches!(gate.next_activity().await, Event::SpeechStarted) {
            started = true;
            break;
        }
    }
    assert!(started, "the VadGate must surface SpeechStarted from speech audio");
}
