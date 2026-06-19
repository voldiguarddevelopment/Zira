//! RED-phase frozen tests for T-01.18 — "Synthesize the speech".
//!
//! These reference `zira_voice::PiperTts` and `zira_voice::TtsError`, which do
//! not exist yet; until they are implemented this file fails to compile — that IS
//! the red state (mirrors the T-01.17 whisper-stt / T-02.19 candle-embedder
//! precedent in this repo).
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> _c1_c2_api_pins (compile-time), c1_c2_c3_real_synthesis
//!   C2 -> _c1_c2_api_pins (compile-time), c1_c2_c3_real_synthesis
//!   C3 -> c1_c2_c3_real_synthesis
//!   C4 -> _c4_api_pin (compile-time),
//!         c4_speak_emits_viseme_frames_within_weight_bounds,
//!         c4_speak_frame_count_tracks_phoneme_count
//!   C5 -> c5_tts_error_implements_error_and_display,
//!         c5_display_exercises_every_variant
//!
//! The model-bearing tests load the real on-disk Piper VITS voice from
//! `$ZIRA_TTS_MODEL` (default `~/.cache/zira/models/piper/en_US-lessac-medium`)
//! and RETURN EARLY when the voice directory is absent, so a model-less CI stays
//! green. The C5 Display tests and the compile-time API pins need no model and
//! always run.

use std::error::Error;
use std::path::{Path, PathBuf};

use zira_core::TtsEngine;
use zira_proto::{Event, VisemeFrame};
use zira_voice::{PiperTts, TtsError};

// ---- helpers ------------------------------------------------------------------

/// Resolves the voice directory: `$ZIRA_TTS_MODEL` if set, else the default
/// `~/.cache/zira/models/piper/en_US-lessac-medium`.
fn voice_dir() -> PathBuf {
    if let Ok(p) = std::env::var("ZIRA_TTS_MODEL") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".cache/zira/models/piper/en_US-lessac-medium")
}

/// True only when the voice directory exists and holds both a `*.onnx` Piper VITS
/// model and its `*.onnx.json` companion config — the two assets `load` needs.
fn voice_present(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let mut has_onnx = false;
    let mut has_json = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".onnx.json") {
            has_json = true;
        } else if name.ends_with(".onnx") {
            has_onnx = true;
        }
    }
    has_onnx && has_json
}

/// The peak absolute amplitude of a PCM buffer (0.0 for an empty buffer).
fn peak_amplitude(pcm: &[f32]) -> f32 {
    pcm.iter().copied().map(f32::abs).fold(0.0_f32, f32::max)
}

// ---- C1 + C2: compile-time API pins -------------------------------------------

/// C1: `PiperTts::load` has signature `fn(&Path) -> Result<PiperTts, TtsError>`
///     and `PiperTts` implements `zira_core::TtsEngine`.
/// C2: `PiperTts::synth` has signature
///     `fn(&mut PiperTts, &str) -> Result<Vec<f32>, TtsError>`.
/// Never called — the coercions alone pin the API at compile time, so the surface
/// is checked even when the voice is absent and the runtime tests skip.
#[allow(dead_code)]
fn _c1_c2_api_pins() {
    fn is_tts_engine<T: TtsEngine>() {}
    is_tts_engine::<PiperTts>();

    let _load: fn(&Path) -> Result<PiperTts, TtsError> = PiperTts::load;
    let _synth: fn(&mut PiperTts, &str) -> Result<Vec<f32>, TtsError> = PiperTts::synth;
}

// ---- C4: compile-time API pin -------------------------------------------------

/// C4: `PiperTts::with_text` attaches the phrase to be spoken to a loaded engine —
/// signature `fn(PiperTts, String) -> PiperTts`. Pinned at compile time so the
/// "constructed with a phrase" builder shape is checked even when the runtime
/// tests skip.
#[allow(dead_code)]
fn _c4_api_pin() {
    let _with_text: fn(PiperTts, String) -> PiperTts = PiperTts::with_text;
}

// ---- C1 + C2 + C3: real synthesis on the on-disk voice ------------------------

/// C1: loads the real Piper VITS voice (`<voice>.onnx` + `<voice>.onnx.json`) from
///     `voice_dir` via the ONNX runtime.
/// C2: `synth` phonemizes the text with espeak-ng, maps phonemes to the voice's
///     phoneme ids, runs the model, and returns f32 PCM at the voice sample rate.
/// C3: the synthesized non-trivial phrase yields at least 6000 samples (~0.27 s at
///     22 kHz) and a peak amplitude strictly within `(0.05, 2.0)` — proving real
///     speech, not silence (peak ~0) or garbage/clipping (peak >= 2.0).
#[test]
fn c1_c2_c3_real_synthesis() {
    let dir = voice_dir();
    if !voice_present(&dir) {
        eprintln!(
            "skipping c1_c2_c3_real_synthesis: piper voice absent at {}",
            dir.display()
        );
        return;
    }

    let mut tts = PiperTts::load(&dir).expect("load piper voice from voice dir");
    let pcm = tts
        .synth("Hello there, this is a test of the speech synthesizer.")
        .expect("synth a non-trivial phrase");

    assert!(
        pcm.len() >= 6000,
        "synth must yield >= 6000 samples (~0.27s of real speech), got {} samples",
        pcm.len()
    );

    let peak = peak_amplitude(&pcm);
    assert!(
        peak > 0.05,
        "peak amplitude must exceed 0.05 (real speech, not silence), got {peak}"
    );
    assert!(
        peak < 2.0,
        "peak amplitude must stay below 2.0 (real speech, not garbage/clipping), got {peak}"
    );
}

// ---- C4: the TtsEngine::speak impl emits one viseme frame per phoneme ----------

/// C4: an engine constructed with a phrase via `with_text` yields, from the
/// `TtsEngine::speak` impl, a non-empty `Vec<Event>` whose every element is an
/// `Event::VisemeFrame(VisemeFrame { .. })` with `weight` within `0.0..=1.0`.
#[tokio::test]
async fn c4_speak_emits_viseme_frames_within_weight_bounds() {
    let dir = voice_dir();
    if !voice_present(&dir) {
        eprintln!(
            "skipping c4_speak_emits_viseme_frames_within_weight_bounds: voice absent at {}",
            dir.display()
        );
        return;
    }

    let mut tts = PiperTts::load(&dir)
        .expect("load piper voice")
        .with_text("Hello there, friend.".to_string());
    let events = tts.speak().await;

    assert!(
        !events.is_empty(),
        "speak must emit at least one viseme frame for a non-trivial phrase"
    );

    for event in &events {
        match event {
            Event::VisemeFrame(frame @ VisemeFrame { .. }) => {
                // Field access auto-derefs, so `weight` is `f32` whether `frame`
                // is owned or borrowed — the bounds check is binding-mode robust.
                let weight = frame.weight;
                assert!(
                    (0.0..=1.0).contains(&weight),
                    "every viseme frame weight must be within 0.0..=1.0, got {weight}"
                );
            }
            other => panic!("speak must emit only Event::VisemeFrame, got {other:?}"),
        }
    }
}

/// C4: "one frame per phoneme" — the frame count tracks the phoneme content of the
/// phrase. A clearly longer phrase (more phonemes) must yield strictly more viseme
/// frames than a very short one, so a constant-count or fixed-stub `speak` cannot
/// pass.
#[tokio::test]
async fn c4_speak_frame_count_tracks_phoneme_count() {
    let dir = voice_dir();
    if !voice_present(&dir) {
        eprintln!(
            "skipping c4_speak_frame_count_tracks_phoneme_count: voice absent at {}",
            dir.display()
        );
        return;
    }

    let mut short = PiperTts::load(&dir)
        .expect("load piper voice (short)")
        .with_text("Hi.".to_string());
    let short_frames = short.speak().await;

    let mut long = PiperTts::load(&dir)
        .expect("load piper voice (long)")
        .with_text("Hello there, this is a considerably longer test sentence.".to_string());
    let long_frames = long.speak().await;

    assert!(
        long_frames.len() > short_frames.len(),
        "a longer phrase must yield more viseme frames (one per phoneme): \
         long={} vs short={}",
        long_frames.len(),
        short_frames.len()
    );
}

// ---- C5: the TtsError type ----------------------------------------------------

/// C5: `TtsError` implements `std::error::Error` and `Display`, with distinct
/// constructible variants for a missing model file, a phonemizer (espeak-ng)
/// failure, and an inference failure.
#[test]
fn c5_tts_error_implements_error_and_display() {
    fn assert_is_error<E: Error>(_: &E) {}

    let missing = TtsError::MissingModelFile(PathBuf::from("some/voice/en_US-lessac-medium.onnx"));
    let phonemize = TtsError::Phonemize("espeak-ng exited non-zero".to_string());
    let inference = TtsError::Inference("ort session run failed".to_string());

    // Error + Display bounds verified at compile time.
    assert_is_error(&missing);
    assert_is_error(&phonemize);
    assert_is_error(&inference);
    let _ = format!("{missing}");
    let _ = format!("{phonemize}");
    let _ = format!("{inference}");
}

/// C5: every required variant's `Display` is exercised — non-empty, names its
/// failure kind, includes its payload context, and all three are mutually
/// distinct, so no two variants can collapse onto one message.
#[test]
fn c5_display_exercises_every_variant() {
    // Neutral context strings carrying none of the failure-kind keywords, so any
    // keyword match comes from the Display format string, not the injected context.
    let missing = TtsError::MissingModelFile(PathBuf::from("ctx-alpha"));
    let phonemize = TtsError::Phonemize("ctx-beta".to_string());
    let inference = TtsError::Inference("ctx-gamma".to_string());

    let missing_msg = format!("{missing}");
    let phonemize_msg = format!("{phonemize}");
    let inference_msg = format!("{inference}");

    // Non-empty.
    assert!(!missing_msg.is_empty(), "MissingModelFile Display must not be empty");
    assert!(!phonemize_msg.is_empty(), "Phonemize Display must not be empty");
    assert!(!inference_msg.is_empty(), "Inference Display must not be empty");

    // Names its failure kind (the injected context contains none of these words).
    let missing_lower = missing_msg.to_lowercase();
    assert!(
        missing_lower.contains("missing") || missing_lower.contains("not found"),
        "MissingModelFile Display must name the missing-file failure, got: {missing_msg:?}"
    );
    let phonemize_lower = phonemize_msg.to_lowercase();
    assert!(
        phonemize_lower.contains("phonem") || phonemize_lower.contains("espeak"),
        "Phonemize Display must name the phonemizer (espeak-ng) failure, got: {phonemize_msg:?}"
    );
    let inference_lower = inference_msg.to_lowercase();
    assert!(
        inference_lower.contains("inference") || inference_lower.contains("infer"),
        "Inference Display must name the inference failure, got: {inference_msg:?}"
    );

    // Includes its payload context — Display must not discard the inner data.
    assert!(
        missing_msg.contains("ctx-alpha"),
        "MissingModelFile Display must include its path, got: {missing_msg:?}"
    );
    assert!(
        phonemize_msg.contains("ctx-beta"),
        "Phonemize Display must include its context, got: {phonemize_msg:?}"
    );
    assert!(
        inference_msg.contains("ctx-gamma"),
        "Inference Display must include its context, got: {inference_msg:?}"
    );

    // Mutually distinct.
    assert_ne!(missing_msg, phonemize_msg, "MissingModelFile and Phonemize must differ");
    assert_ne!(missing_msg, inference_msg, "MissingModelFile and Inference must differ");
    assert_ne!(phonemize_msg, inference_msg, "Phonemize and Inference must differ");
}
