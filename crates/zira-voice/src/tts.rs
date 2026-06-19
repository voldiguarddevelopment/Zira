//! Real Phase-1 TTS (T-01.18) — a Piper VITS voice run through the ONNX runtime.
//!
//! [`PiperTts::load`] opens a Piper voice (`<voice>.onnx` + `<voice>.onnx.json`)
//! on the CPU. [`PiperTts::synth`] phonemizes the text with the system
//! `espeak-ng` binary, maps the phonemes to the voice's phoneme ids, runs one
//! forward pass of the model, and returns the f32 PCM at the voice's sample rate.
//! The async [`TtsEngine::speak`] impl turns the attached phrase into one
//! [`Event::VisemeFrame`] per phoneme for lip-sync.
//!
//! Live audio playback, emotion/prosody modulation, and streaming are out of
//! scope — the engine returns one PCM buffer for one phrase at a time.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ort::session::Session;
use ort::value::Tensor;
use zira_core::TtsEngine;
use zira_proto::{Event, VisemeFrame};

/// Piper's beginning-of-sentence, padding, and end-of-sentence phoneme symbols.
const BOS: &str = "^";
const PAD: &str = "_";
const EOS: &str = "$";

/// Default VITS inference scales, used when the voice config omits `inference`.
const DEFAULT_NOISE_SCALE: f32 = 0.667;
const DEFAULT_LENGTH_SCALE: f32 = 1.0;
const DEFAULT_NOISE_W: f32 = 0.8;

/// Viseme weight for an open-mouth vowel shape.
const VOWEL_WEIGHT: f32 = 1.0;
/// Viseme weight for a partially-closed consonant shape.
const CONSONANT_WEIGHT: f32 = 0.5;

/// Typed errors raised while loading or running the Piper TTS engine. Each
/// variant names a distinct failure surface so callers (and the `Display` tests)
/// can tell a missing model from a phonemizer failure from an inference failure.
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    /// The Piper `<voice>.onnx` model (or its `.onnx.json` companion) was absent
    /// from the voice directory.
    #[error("missing model file: {0}")]
    MissingModelFile(PathBuf),

    /// The phonemizer (the `espeak-ng` binary) could not be run, exited non-zero,
    /// or produced no phonemes for the text.
    #[error("phonemizer (espeak-ng) failed: {0}")]
    Phonemize(String),

    /// Building/parsing the voice config, building the ONNX session, or running a
    /// forward pass failed.
    #[error("inference failed: {0}")]
    Inference(String),
}

/// A Piper VITS voice loaded onto the CPU through the ONNX runtime, plus an
/// optional attached phrase that the async [`TtsEngine::speak`] impl renders into
/// viseme frames.
///
/// Built from a directory holding `<voice>.onnx` and `<voice>.onnx.json`. The
/// ONNX session is mutated by a run, so [`PiperTts::synth`] takes `&mut self`.
pub struct PiperTts {
    session: Session,
    phoneme_id_map: HashMap<String, i64>,
    sample_rate: u32,
    noise_scale: f32,
    length_scale: f32,
    noise_w: f32,
    text: Option<String>,
}

impl PiperTts {
    /// Load a Piper VITS voice from `voice_dir` onto the CPU.
    ///
    /// Expects a `<voice>.onnx` model and its `<voice>.onnx.json` config in the
    /// directory. A missing asset yields [`TtsError::MissingModelFile`]; a config
    /// parse or session-build failure yields [`TtsError::Inference`].
    pub fn load(voice_dir: &Path) -> Result<PiperTts, TtsError> {
        let onnx_path = find_onnx(voice_dir)?;
        let json_path = {
            let mut p = onnx_path.clone().into_os_string();
            p.push(".json");
            PathBuf::from(p)
        };
        if !json_path.is_file() {
            return Err(TtsError::MissingModelFile(json_path));
        }

        let config_bytes = std::fs::read(&json_path)
            .map_err(|e| TtsError::Inference(format!("read voice config {json_path:?}: {e}")))?;
        let config: serde_json::Value = serde_json::from_slice(&config_bytes)
            .map_err(|e| TtsError::Inference(format!("parse voice config: {e}")))?;

        let phoneme_id_map = parse_phoneme_id_map(&config)?;

        let sample_rate = config["audio"]["sample_rate"]
            .as_u64()
            .ok_or_else(|| TtsError::Inference("voice config missing audio.sample_rate".into()))?
            as u32;

        let inference = &config["inference"];
        let scale = |key: &str, default: f32| {
            inference[key].as_f64().map(|v| v as f32).unwrap_or(default)
        };
        let noise_scale = scale("noise_scale", DEFAULT_NOISE_SCALE);
        let length_scale = scale("length_scale", DEFAULT_LENGTH_SCALE);
        let noise_w = scale("noise_w", DEFAULT_NOISE_W);

        let session = Session::builder()
            .map_err(|e| TtsError::Inference(format!("build ONNX session builder: {e}")))?
            .commit_from_file(&onnx_path)
            .map_err(|e| TtsError::Inference(format!("load ONNX model {onnx_path:?}: {e}")))?;

        Ok(PiperTts {
            session,
            phoneme_id_map,
            sample_rate,
            noise_scale,
            length_scale,
            noise_w,
            text: None,
        })
    }

    /// Attach the phrase the async [`TtsEngine::speak`] impl will render into
    /// viseme frames. Builder-style: consumes and returns the engine.
    #[must_use]
    pub fn with_text(mut self, text: String) -> PiperTts {
        self.text = Some(text);
        self
    }

    /// The voice's audio sample rate, in Hz (e.g. 22050 for `lessac-medium`).
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Synthesize `text` into f32 PCM audio at the voice's sample rate.
    ///
    /// Phonemizes with `espeak-ng`, maps the phonemes to the voice's phoneme ids,
    /// runs one forward pass of the VITS model, and flattens the `[1,1,1,N]`
    /// output to the `N` PCM samples.
    pub fn synth(&mut self, text: &str) -> Result<Vec<f32>, TtsError> {
        let phonemes = phonemize(text)?;
        let ids = self.phoneme_ids(&phonemes)?;
        let len = ids.len();

        let input = Tensor::from_array(([1_i64, len as i64], ids))
            .map_err(|e| TtsError::Inference(format!("build input tensor: {e}")))?;
        let input_lengths = Tensor::from_array(([1_i64], vec![len as i64]))
            .map_err(|e| TtsError::Inference(format!("build input_lengths tensor: {e}")))?;
        let scales = Tensor::from_array((
            [3_i64],
            vec![self.noise_scale, self.length_scale, self.noise_w],
        ))
        .map_err(|e| TtsError::Inference(format!("build scales tensor: {e}")))?;

        let outputs = self
            .session
            .run(ort::inputs! {
                "input" => input,
                "input_lengths" => input_lengths,
                "scales" => scales,
            })
            .map_err(|e| TtsError::Inference(format!("ONNX session run: {e}")))?;

        let (_, pcm) = outputs["output"]
            .try_extract_tensor::<f32>()
            .map_err(|e| TtsError::Inference(format!("extract output PCM: {e}")))?;

        Ok(pcm.to_vec())
    }

    /// Map a phoneme string to the model's id sequence: `^` (BOS), `_` (pad), then
    /// each mappable phoneme followed by a pad, then `$` (EOS).
    fn phoneme_ids(&self, phonemes: &str) -> Result<Vec<i64>, TtsError> {
        let id = |k: &str| {
            self.phoneme_id_map
                .get(k)
                .copied()
                .ok_or_else(|| TtsError::Inference(format!("voice has no id for phoneme {k:?}")))
        };

        let bos = id(BOS)?;
        let pad = id(PAD)?;
        let eos = id(EOS)?;

        let mut ids = vec![bos, pad];
        for ch in phonemes.chars() {
            if let Some(&phoneme_id) = self.phoneme_id_map.get(&ch.to_string()) {
                ids.push(phoneme_id);
                ids.push(pad);
            }
        }
        ids.push(eos);
        Ok(ids)
    }
}

impl TtsEngine for PiperTts {
    /// Render the attached phrase into one [`Event::VisemeFrame`] per phoneme.
    ///
    /// Phonemizes the text with `espeak-ng`, then maps each spoken phoneme to a
    /// viseme shape (a vowel to an open mouth, a consonant to a partial mouth),
    /// in order. Returns an empty stream when no phrase is attached or the
    /// phonemizer fails — never a panic.
    async fn speak(&mut self) -> Vec<Event> {
        let Some(text) = self.text.clone() else {
            return Vec::new();
        };
        let Ok(phonemes) = phonemize(&text) else {
            return Vec::new();
        };

        phonemes
            .chars()
            .filter_map(viseme_for_phoneme)
            .map(|(viseme, weight)| Event::VisemeFrame(VisemeFrame { viseme, weight }))
            .collect()
    }
}

/// Find the `<voice>.onnx` model in `voice_dir` (the entry ending in `.onnx` but
/// not `.onnx.json`). Absence yields [`TtsError::MissingModelFile`] naming the dir.
fn find_onnx(voice_dir: &Path) -> Result<PathBuf, TtsError> {
    let entries = std::fs::read_dir(voice_dir)
        .map_err(|_| TtsError::MissingModelFile(voice_dir.to_path_buf()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.ends_with(".onnx") && !name.ends_with(".onnx.json") {
            return Ok(path);
        }
    }
    Err(TtsError::MissingModelFile(voice_dir.to_path_buf()))
}

/// Parse the `phoneme_id_map` (each value an array of ids) into a flat
/// `phoneme -> first id` map.
fn parse_phoneme_id_map(config: &serde_json::Value) -> Result<HashMap<String, i64>, TtsError> {
    let raw = config["phoneme_id_map"]
        .as_object()
        .ok_or_else(|| TtsError::Inference("voice config missing phoneme_id_map".into()))?;

    let mut map = HashMap::with_capacity(raw.len());
    for (phoneme, ids) in raw {
        let id = ids
            .as_array()
            .and_then(|a| a.first())
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                TtsError::Inference(format!("phoneme_id_map entry {phoneme:?} has no id"))
            })?;
        map.insert(phoneme.clone(), id);
    }
    Ok(map)
}

/// Phonemize `text` into an IPA phoneme string via the system `espeak-ng` binary
/// (`espeak-ng -q --ipa -v en-us <text>`). A spawn failure, a non-zero exit, or
/// empty output yields [`TtsError::Phonemize`].
fn phonemize(text: &str) -> Result<String, TtsError> {
    let output = Command::new("espeak-ng")
        .args(["-q", "--ipa", "-v", "en-us", text])
        .output()
        .map_err(|e| TtsError::Phonemize(format!("spawn espeak-ng: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TtsError::Phonemize(format!(
            "espeak-ng exited with {}: {}",
            output.status,
            stderr.trim()
        )));
    }

    let phonemes = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if phonemes.is_empty() {
        return Err(TtsError::Phonemize(format!(
            "espeak-ng produced no phonemes for {text:?}"
        )));
    }
    Ok(phonemes)
}

/// Map a single IPA phoneme character to a `(viseme label, weight)` lip-sync
/// shape, or `None` for whitespace, stress, and length diacritics (which carry no
/// mouth shape). Vowels open the mouth (weight 1.0); consonants partially close
/// it (weight 0.5).
fn viseme_for_phoneme(c: char) -> Option<(String, f32)> {
    // Stress (ˈ ˌ), length (ː ˑ), the tie bar, and whitespace carry no mouth shape.
    if c.is_whitespace() || matches!(c, 'ˈ' | 'ˌ' | 'ː' | 'ˑ' | '\u{0361}') {
        return None;
    }
    let is_vowel = matches!(
        c,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | 'æ'
            | 'ɑ'
            | 'ɒ'
            | 'ɔ'
            | 'ə'
            | 'ɚ'
            | 'ɛ'
            | 'ɜ'
            | 'ɝ'
            | 'ɪ'
            | 'ʊ'
            | 'ʌ'
            | 'ø'
            | 'œ'
            | 'ɐ'
            | 'ɨ'
            | 'ʉ'
            | 'ɯ'
    );
    let (label, weight) = if is_vowel {
        ("open", VOWEL_WEIGHT)
    } else {
        ("partial", CONSONANT_WEIGHT)
    };
    Some((label.to_string(), weight))
}
