//! RED-phase frozen tests for T-01.17 — "Transcribe the speech".
//!
//! These reference `zira_voice::WhisperStt` and `zira_voice::SttError`, which do
//! not exist yet; until they are implemented this file fails to compile — that IS
//! the red state (mirrors the T-02.19 candle-embedder precedent).
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> _c1_c2_api_pins (compile-time), c1_c2_c3_real_asr
//!   C2 -> _c1_c2_api_pins (compile-time), c1_c2_c3_real_asr
//!   C3 -> c1_c2_c3_real_asr
//!   C4 -> _c4_api_pins (compile-time),
//!         c4_transcribe_impl_emits_transcript_ready_matching_pcm,
//!         c4_transcribe_impl_maps_failure_to_error_event
//!   C5 -> c5_stt_error_implements_error_and_display,
//!         c5_display_exercises_every_variant
//!
//! The model-bearing tests load the real on-disk whisper-tiny.en from
//! `$ZIRA_STT_MODEL` (default `~/.cache/zira/models/whisper-tiny.en`) and RETURN
//! EARLY when the model is absent, so a model-less CI stays green. The C5 Display
//! tests and the compile-time API pins need no model and always run.

use std::error::Error;
use std::path::{Path, PathBuf};

use zira_core::SttEngine;
use zira_proto::{Event, Transcript};
use zira_voice::{SttError, WhisperStt};

// ---- helpers ------------------------------------------------------------------

/// Resolves the model directory: `$ZIRA_STT_MODEL` if set, else the default
/// `~/.cache/zira/models/whisper-tiny.en`.
fn model_dir() -> PathBuf {
    if let Ok(p) = std::env::var("ZIRA_STT_MODEL") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".cache/zira/models/whisper-tiny.en")
}

/// True only when every required model asset plus the jfk fixture is on disk.
fn model_present(dir: &Path) -> bool {
    dir.join("config.json").is_file()
        && dir.join("tokenizer.json").is_file()
        && dir.join("model.safetensors").is_file()
        && dir.join("melfilters.bytes").is_file()
        && dir.join("jfk.wav").is_file()
}

/// Decodes the 16 kHz `jfk.wav` fixture into a mono f32 PCM buffer in `[-1, 1]`.
fn load_pcm(wav: &Path) -> Vec<f32> {
    let mut reader = hound::WavReader::open(wav).expect("open jfk.wav fixture");
    let spec = reader.spec();
    assert_eq!(spec.sample_rate, 16_000, "jfk fixture must be 16 kHz");
    match spec.sample_format {
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .map(|s| s.expect("read i16 sample") as f32 / 32_768.0)
            .collect(),
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.expect("read f32 sample"))
            .collect(),
    }
}

// ---- C1 + C2: compile-time API pins -------------------------------------------

/// C1: `WhisperStt::load` has signature `fn(&Path) -> Result<WhisperStt, SttError>`
///     and `WhisperStt` implements `zira_core::SttEngine`.
/// C2: `WhisperStt::transcribe_pcm` has signature
///     `fn(&mut WhisperStt, &[f32]) -> Result<String, SttError>`.
/// Never called — the coercions alone pin the API at compile time, so the surface
/// is checked even when the model is absent and the runtime tests skip.
#[allow(dead_code)]
fn _c1_c2_api_pins() {
    fn is_stt_engine<T: SttEngine>() {}
    is_stt_engine::<WhisperStt>();

    let _load: fn(&Path) -> Result<WhisperStt, SttError> = WhisperStt::load;
    let _transcribe_pcm: fn(&mut WhisperStt, &[f32]) -> Result<String, SttError> =
        WhisperStt::transcribe_pcm;
}

// ---- C4: compile-time API pin -------------------------------------------------

/// C4: `WhisperStt::with_audio` attaches a PCM buffer to a loaded engine —
/// signature `fn(WhisperStt, Vec<f32>) -> WhisperStt`. Pinned at compile time so
/// the builder shape is checked even when the runtime tests skip.
#[allow(dead_code)]
fn _c4_api_pins() {
    let _with_audio: fn(WhisperStt, Vec<f32>) -> WhisperStt = WhisperStt::with_audio;
}

// ---- C1 + C2 + C3: real ASR on the jfk fixture --------------------------------

/// C1: loads the real Candle whisper-tiny.en model from `model_dir` on the CPU.
/// C2: `transcribe_pcm` pads/clamps the 16 kHz audio to the 30 s window, computes
///     the log-mel, runs the encoder + greedy decode, and returns the transcript.
/// C3: the lowercased transcript contains both `country` and `americans` and is at
///     least 60 chars — proving real ASR of the JFK quote, not a stub.
#[test]
fn c1_c2_c3_real_asr() {
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!(
            "skipping c1_c2_c3_real_asr: whisper model absent at {}",
            dir.display()
        );
        return;
    }

    let mut stt = WhisperStt::load(&dir).expect("load whisper-tiny.en from model dir");
    let pcm = load_pcm(&dir.join("jfk.wav"));
    let transcript = stt
        .transcribe_pcm(&pcm)
        .expect("transcribe_pcm of the jfk fixture");

    let lower = transcript.to_lowercase();
    assert!(
        lower.contains("country"),
        "transcript must contain 'country', got: {transcript:?}"
    );
    assert!(
        lower.contains("americans"),
        "transcript must contain 'americans', got: {transcript:?}"
    );
    assert!(
        transcript.chars().count() >= 60,
        "transcript must be >= 60 chars (real ASR, not a stub), got {} chars: {transcript:?}",
        transcript.chars().count()
    );
}

// ---- C4: the SttEngine::transcribe impl ---------------------------------------

/// C4: an engine constructed with the fixture audio via `with_audio` yields, from
/// the `SttEngine::transcribe` impl, `Event::TranscriptReady(Transcript { text })`
/// whose `text` equals the direct `transcribe_pcm` result on the same audio.
#[tokio::test]
async fn c4_transcribe_impl_emits_transcript_ready_matching_pcm() {
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!(
            "skipping c4_transcribe_impl_emits_transcript_ready_matching_pcm: model absent at {}",
            dir.display()
        );
        return;
    }

    let pcm = load_pcm(&dir.join("jfk.wav"));

    // The reference text from the direct, synchronous worker.
    let mut direct_engine = WhisperStt::load(&dir).expect("load for direct transcribe_pcm");
    let direct = direct_engine
        .transcribe_pcm(&pcm)
        .expect("direct transcribe_pcm");

    // The same audio driven through the async SttEngine::transcribe impl.
    let mut engine = WhisperStt::load(&dir)
        .expect("load for transcribe impl")
        .with_audio(pcm.clone());
    let event = engine.transcribe().await;

    match event {
        Event::TranscriptReady(Transcript { text }) => assert_eq!(
            text, direct,
            "SttEngine::transcribe text must equal the direct transcribe_pcm result"
        ),
        other => panic!("expected Event::TranscriptReady, got {other:?}"),
    }
}

/// C4: a decode failure (here: no audio buffer was supplied) yields an
/// `Event::Error` from the `SttEngine::transcribe` impl rather than a panic.
#[tokio::test]
async fn c4_transcribe_impl_maps_failure_to_error_event() {
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!(
            "skipping c4_transcribe_impl_maps_failure_to_error_event: model absent at {}",
            dir.display()
        );
        return;
    }

    // Loaded model, but no audio attached: transcribe must surface the failure as
    // an Event::Error, never an unwrap/panic.
    let mut engine = WhisperStt::load(&dir).expect("load without audio");
    let event = engine.transcribe().await;

    assert!(
        matches!(event, Event::Error(_)),
        "an unset/empty audio buffer must map to Event::Error, got {event:?}"
    );
}

// ---- C5: the SttError type ----------------------------------------------------

/// C5: `SttError` implements `std::error::Error` and `Display`, with distinct
/// constructible variants for a missing model file, a model-load failure, and an
/// audio/decode failure.
#[test]
fn c5_stt_error_implements_error_and_display() {
    fn assert_is_error<E: Error>(_: &E) {}

    let missing = SttError::MissingModelFile(PathBuf::from("some/model/config.json"));
    let load = SttError::ModelLoad("safetensors mmap failed".to_string());
    let decode = SttError::Decode("greedy decode produced no tokens".to_string());

    // Error + Display bounds verified at compile time.
    assert_is_error(&missing);
    assert_is_error(&load);
    assert_is_error(&decode);
    let _ = format!("{missing}");
    let _ = format!("{load}");
    let _ = format!("{decode}");
}

/// C5: every variant's `Display` is exercised — non-empty, names its failure kind,
/// includes its payload context, and all three are mutually distinct. The
/// exhaustive `match` (no wildcard) freezes the variant set to exactly these three,
/// so a fourth, unexercised variant cannot slip past this test.
#[test]
fn c5_display_exercises_every_variant() {
    // Neutral context strings carrying none of the failure-kind keywords, so any
    // keyword match comes from the Display format string, not the injected context.
    let missing = SttError::MissingModelFile(PathBuf::from("ctx-alpha"));
    let load = SttError::ModelLoad("ctx-beta".to_string());
    let decode = SttError::Decode("ctx-gamma".to_string());

    // Exhaustive match pins the variant set; if GREEN adds a variant this stops
    // compiling, forcing the set to stay exactly { MissingModelFile, ModelLoad, Decode }.
    fn kind(e: &SttError) -> &'static str {
        match e {
            SttError::MissingModelFile(_) => "missing",
            SttError::ModelLoad(_) => "model-load",
            SttError::Decode(_) => "decode",
        }
    }
    assert_eq!(kind(&missing), "missing");
    assert_eq!(kind(&load), "model-load");
    assert_eq!(kind(&decode), "decode");

    let missing_msg = format!("{missing}");
    let load_msg = format!("{load}");
    let decode_msg = format!("{decode}");

    // Non-empty.
    assert!(!missing_msg.is_empty(), "MissingModelFile Display must not be empty");
    assert!(!load_msg.is_empty(), "ModelLoad Display must not be empty");
    assert!(!decode_msg.is_empty(), "Decode Display must not be empty");

    // Names its failure kind (the injected context contains none of these words).
    let missing_lower = missing_msg.to_lowercase();
    assert!(
        missing_lower.contains("missing") || missing_lower.contains("not found"),
        "MissingModelFile Display must name the missing-file failure, got: {missing_msg:?}"
    );
    let load_lower = load_msg.to_lowercase();
    assert!(
        load_lower.contains("load"),
        "ModelLoad Display must name the model-load failure, got: {load_msg:?}"
    );
    let decode_lower = decode_msg.to_lowercase();
    assert!(
        decode_lower.contains("decode") || decode_lower.contains("audio"),
        "Decode Display must name the audio/decode failure, got: {decode_msg:?}"
    );

    // Includes its payload context — Display must not discard the inner data.
    assert!(
        missing_msg.contains("ctx-alpha"),
        "MissingModelFile Display must include its path, got: {missing_msg:?}"
    );
    assert!(
        load_msg.contains("ctx-beta"),
        "ModelLoad Display must include its context, got: {load_msg:?}"
    );
    assert!(
        decode_msg.contains("ctx-gamma"),
        "Decode Display must include its context, got: {decode_msg:?}"
    );

    // Mutually distinct.
    assert_ne!(missing_msg, load_msg, "MissingModelFile and ModelLoad must differ");
    assert_ne!(missing_msg, decode_msg, "MissingModelFile and Decode must differ");
    assert_ne!(load_msg, decode_msg, "ModelLoad and Decode must differ");
}
