//! zira-voice — wakeword/VAD/STT/TTS interfaces.
//!
//! The real STT engine (T-01.17): a CPU Candle whisper-tiny.en model that
//! transcribes a supplied 16 kHz PCM buffer into text. Mic capture, GPU, and
//! streaming are out of scope — the engine transcribes one supplied utterance at
//! a time on the CPU.

use std::path::{Path, PathBuf};

use byteorder::{LittleEndian, ReadBytesExt};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::whisper::{self, model::Whisper, Config};
use tokenizers::Tokenizer;
use zira_core::SttEngine;
use zira_proto::{Event, Transcript};

/// Whisper special tokens.
const SOT_TOKEN: u32 = 50257;
const EOT_TOKEN: u32 = 50256;
const NO_TIMESTAMPS_TOKEN: u32 = 50362;

/// Whisper consumes a fixed 30-second window at 16 kHz.
const SAMPLE_RATE: usize = 16_000;
const CHUNK_SECONDS: usize = 30;
const N_SAMPLES: usize = SAMPLE_RATE * CHUNK_SECONDS;
/// Encoder frame budget for one 30 s window.
const MAX_FRAMES: usize = 3000;
/// Greedy-decode token budget for one utterance.
const MAX_DECODE_STEPS: usize = 224;

/// Typed errors raised while loading or running the whisper STT engine. Each
/// variant names a distinct failure surface so callers (and the `Display` tests)
/// can tell a missing asset from a load failure from a decode failure.
#[derive(Debug, thiserror::Error)]
pub enum SttError {
    /// A required model asset (config/tokenizer/weights/mel filters) was absent
    /// from the model directory.
    #[error("missing model file: {0}")]
    MissingModelFile(PathBuf),

    /// The model assets were present but could not be parsed or loaded onto the
    /// CPU (config parse, safetensors mmap, tokenizer build, mel-filter read).
    #[error("model load failed: {0}")]
    ModelLoad(String),

    /// The encoder/decoder run over the supplied audio failed, or no audio was
    /// available to decode.
    #[error("audio decode failed: {0}")]
    Decode(String),
}

/// A CPU Candle whisper model loaded from disk, plus an optional attached PCM
/// buffer that the async [`SttEngine::transcribe`] impl decodes.
///
/// Built from a directory holding `config.json`, `tokenizer.json`,
/// `model.safetensors`, and `melfilters.bytes`. The model carries a kv-cache, so
/// transcription needs `&mut self`.
pub struct WhisperStt {
    model: Whisper,
    config: Config,
    tokenizer: Tokenizer,
    mel_filters: Vec<f32>,
    audio: Option<Vec<f32>>,
}

impl WhisperStt {
    /// Load a whisper model from `model_dir` onto the CPU.
    ///
    /// Expects `config.json`, `tokenizer.json`, `model.safetensors`, and
    /// `melfilters.bytes` in the directory. A missing asset yields
    /// [`SttError::MissingModelFile`]; a parse/load failure yields
    /// [`SttError::ModelLoad`].
    pub fn load(model_dir: &Path) -> Result<WhisperStt, SttError> {
        let config_path = model_dir.join("config.json");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let weights_path = model_dir.join("model.safetensors");
        let mel_path = model_dir.join("melfilters.bytes");

        for path in [&config_path, &tokenizer_path, &weights_path, &mel_path] {
            if !path.is_file() {
                return Err(SttError::MissingModelFile(path.clone()));
            }
        }

        let config_bytes = std::fs::read(&config_path)
            .map_err(|e| SttError::ModelLoad(format!("read config.json: {e}")))?;
        let config: Config = serde_json::from_slice(&config_bytes)
            .map_err(|e| SttError::ModelLoad(format!("parse config.json: {e}")))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| SttError::ModelLoad(format!("load tokenizer.json: {e}")))?;

        let mel_filters = read_mel_filters(&mel_path)?;

        // SAFETY: mmap of a trusted, on-disk model file we just verified exists.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], whisper::DTYPE, &Device::Cpu)
                .map_err(|e| SttError::ModelLoad(format!("mmap model.safetensors: {e}")))?
        };
        let model = Whisper::load(&vb, config.clone())
            .map_err(|e| SttError::ModelLoad(format!("build whisper model: {e}")))?;

        Ok(WhisperStt {
            model,
            config,
            tokenizer,
            mel_filters,
            audio: None,
        })
    }

    /// Attach a 16 kHz mono PCM buffer for the async [`SttEngine::transcribe`]
    /// impl to decode. Builder-style: consumes and returns the engine.
    pub fn with_audio(mut self, pcm: Vec<f32>) -> WhisperStt {
        self.audio = Some(pcm);
        self
    }

    /// Transcribe a 16 kHz mono PCM buffer.
    ///
    /// Pads/clamps the audio to whisper's 30-second window, computes the log-mel
    /// spectrogram, runs the encoder, then greedily decodes the text tokens.
    pub fn transcribe_pcm(&mut self, pcm: &[f32]) -> Result<String, SttError> {
        self.model.reset_kv_cache();

        // Pad (or clamp) to exactly one 30 s window.
        let mut samples = pcm.to_vec();
        samples.resize(N_SAMPLES, 0.0);

        let mel = whisper::audio::pcm_to_mel(&self.config, &samples, &self.mel_filters);
        let n_mel = self.config.num_mel_bins;
        let frames = mel.len() / n_mel;

        let mel = Tensor::from_vec(mel, (1, n_mel, frames), &Device::Cpu)
            .map_err(|e| SttError::Decode(format!("build mel tensor: {e}")))?;
        let mel = if frames > MAX_FRAMES {
            mel.narrow(2, 0, MAX_FRAMES)
                .map_err(|e| SttError::Decode(format!("narrow mel frames: {e}")))?
        } else {
            mel
        };

        let features = self
            .model
            .encoder
            .forward(&mel, true)
            .map_err(|e| SttError::Decode(format!("encoder forward: {e}")))?;

        let mut tokens: Vec<u32> = vec![SOT_TOKEN, NO_TIMESTAMPS_TOKEN];
        for step in 0..MAX_DECODE_STEPS {
            let input = Tensor::new(tokens.as_slice(), &Device::Cpu)
                .and_then(|t| t.unsqueeze(0))
                .map_err(|e| SttError::Decode(format!("build decoder input: {e}")))?;
            // Flush the kv-cache only on the first step so the cache persists;
            // flushing every step is O(n^2) and far too slow.
            let ys = self
                .model
                .decoder
                .forward(&input, &features, step == 0)
                .map_err(|e| SttError::Decode(format!("decoder forward: {e}")))?;
            let seq_len = ys
                .dim(1)
                .map_err(|e| SttError::Decode(format!("decoder output dim: {e}")))?;
            let logits = ys
                .narrow(1, seq_len - 1, 1)
                .and_then(|last| self.model.decoder.final_linear(&last))
                .and_then(|l| l.squeeze(0))
                .and_then(|l| l.squeeze(0))
                .map_err(|e| SttError::Decode(format!("final linear: {e}")))?;
            let next = logits
                .argmax(0)
                .and_then(|t| t.to_scalar::<u32>())
                .map_err(|e| SttError::Decode(format!("argmax token: {e}")))?;
            if next == EOT_TOKEN {
                break;
            }
            tokens.push(next);
        }

        self.tokenizer
            .decode(&tokens[2..], true)
            .map_err(|e| SttError::Decode(format!("detokenize transcript: {e}")))
    }
}

impl SttEngine for WhisperStt {
    /// Decode the attached audio buffer into a [`Event::TranscriptReady`]. A
    /// missing buffer or a decode failure maps to [`Event::Error`] — never a panic.
    async fn transcribe(&mut self) -> Event {
        let pcm = match self.audio.take() {
            Some(pcm) => pcm,
            None => {
                return Event::Error(
                    SttError::Decode("no audio buffer supplied".into()).to_string(),
                )
            }
        };
        match self.transcribe_pcm(&pcm) {
            Ok(text) => Event::TranscriptReady(Transcript { text }),
            Err(e) => Event::Error(e.to_string()),
        }
    }
}

/// Read `melfilters.bytes` as a little-endian `f32` array (`num_mel_bins * 201`).
fn read_mel_filters(path: &Path) -> Result<Vec<f32>, SttError> {
    let bytes = std::fs::read(path)
        .map_err(|e| SttError::ModelLoad(format!("read melfilters.bytes: {e}")))?;
    if bytes.len() % 4 != 0 {
        return Err(SttError::ModelLoad(format!(
            "melfilters.bytes length {} is not a multiple of 4",
            bytes.len()
        )));
    }
    let mut filters = vec![0f32; bytes.len() / 4];
    let mut cursor = &bytes[..];
    cursor
        .read_f32_into::<LittleEndian>(&mut filters)
        .map_err(|e| SttError::ModelLoad(format!("decode melfilters.bytes: {e}")))?;
    Ok(filters)
}
