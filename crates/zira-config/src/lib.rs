//! zira-config — config schema, TOML load, XDG paths, constitution.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The application subdirectory Zira owns under each XDG base directory.
const APP_DIR: &str = "zira";

/// Errors resolving or creating Zira's on-disk directories.
#[derive(Debug, Error)]
pub enum PathError {
    /// An XDG base directory could not be resolved: the environment variable was unset
    /// (or empty) and no fallback home directory was available.
    #[error("cannot resolve base directory (env var {var} unset and no $HOME fallback)")]
    Unresolved {
        /// The XDG environment variable that was consulted.
        var: &'static str,
    },
    /// A directory could not be created.
    #[error("failed to create directory {path}: {source}")]
    Create {
        /// The path whose creation failed.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Resolve an XDG base directory: the value of `var` when it is set and non-empty,
/// otherwise `$HOME/{fallback}`.
fn xdg_base(var: &'static str, fallback: &str) -> Result<PathBuf, PathError> {
    if let Some(value) = std::env::var_os(var) {
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
    }
    let home = std::env::var_os("HOME").filter(|h| !h.is_empty());
    match home {
        Some(home) => Ok(PathBuf::from(home).join(fallback)),
        None => Err(PathError::Unresolved { var }),
    }
}

/// Zira's configuration directory, under `XDG_CONFIG_HOME` (default `$HOME/.config`).
pub fn config_dir() -> Result<PathBuf, PathError> {
    Ok(xdg_base("XDG_CONFIG_HOME", ".config")?.join(APP_DIR))
}

/// Zira's data directory, under `XDG_DATA_HOME` (default `$HOME/.local/share`).
pub fn data_dir() -> Result<PathBuf, PathError> {
    Ok(xdg_base("XDG_DATA_HOME", ".local/share")?.join(APP_DIR))
}

/// Zira's on-disk memory directory, under the data directory.
pub fn memory_dir() -> Result<PathBuf, PathError> {
    Ok(data_dir()?.join("memory"))
}

/// Zira's staged/live skills directory, under the data directory.
pub fn skills_dir() -> Result<PathBuf, PathError> {
    Ok(data_dir()?.join("skills"))
}

/// Create every Zira directory that does not yet exist.
///
/// Creates the config, data, memory, and skills directories (parents included). An
/// already-existing directory is not an error, so a second call succeeds — the helper
/// is idempotent. A path that cannot be created yields [`PathError::Create`].
pub fn ensure_dirs() -> Result<(), PathError> {
    for dir in [config_dir()?, data_dir()?, memory_dir()?, skills_dir()?] {
        std::fs::create_dir_all(&dir).map_err(|source| PathError::Create {
            path: dir.display().to_string(),
            source,
        })?;
    }
    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    pub config_dir: Option<String>,
    pub data_dir: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    pub model_id: String,
    pub binary_path: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WakewordConfig {
    pub model_path: String,
    pub threshold: f32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VadConfig {
    pub threshold: f32,
    pub silence_ms: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SttConfig {
    pub model_path: String,
    pub language: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TtsConfig {
    pub model_path: String,
    pub voice: String,
    pub sample_rate: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmotionConfig {
    pub default_emotion: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    pub max_episodes: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AvatarConfig {
    pub vrm_path: Option<String>,
}

/// Errors surfaced by config handling: loading (a missing file is NOT an error — it
/// yields the default) and validation (each invalid field maps to a distinct variant
/// naming the offending field and the reason it was rejected).
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("malformed TOML in config file: {0}")]
    Parse(String),
    #[error("I/O error reading config file: {0}")]
    Io(String),
    /// A sample rate that must be a positive number of Hz was non-positive (zero).
    #[error("invalid sample rate for `{field}`: {value} Hz (must be a positive number of Hz)")]
    InvalidSampleRate {
        /// The dotted name of the offending field, e.g. `tts.sample_rate`.
        field: &'static str,
        /// The rejected value.
        value: u32,
    },
    /// A path that is required in the current configuration was empty.
    #[error("required path `{field}` is empty (a non-empty value is required here)")]
    EmptyPath {
        /// The dotted name of the offending field, e.g. `model.binary_path`.
        field: &'static str,
    },
    /// A threshold fell outside the permitted `[0.0, 1.0]` range.
    #[error("threshold `{field}` = {value} is out of range (must be within [0.0, 1.0])")]
    ThresholdOutOfRange {
        /// The dotted name of the offending field, e.g. `wakeword.threshold`.
        field: &'static str,
        /// The rejected value.
        value: f32,
    },
}

/// Errors surfaced by emotion-vocabulary validation.
#[derive(Debug, Error)]
pub enum VocabError {
    /// A tag matched no known `Emotion` variant.
    #[error("unknown emotion tag: {tag}")]
    UnknownTag { tag: String },
}

const DEFAULT_CONSTITUTION: &str = include_str!("constitution.txt");

/// The immutable baseline policy compiled into the Zira binary.
///
/// Loaded from an embedded default via [`Constitution::load_default`]; the text is
/// compiled in with `include_str!` so it is always present without a file on disk.
/// Once loaded, the rule set cannot be mutated — all accessors take `&self`.
#[derive(Debug, Clone)]
pub struct Constitution {
    rules: Vec<String>,
}

impl Constitution {
    /// Return the embedded default constitution.
    ///
    /// This never fails at runtime: the embedded text is compiled in and is guaranteed
    /// non-empty. A malformed embedded default would cause a compile-time panic (or a
    /// panic at first call during testing).
    pub fn load_default() -> Constitution {
        let rules: Vec<String> = DEFAULT_CONSTITUTION
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        assert!(!rules.is_empty(), "embedded constitution must not be empty");
        Constitution { rules }
    }

    /// Return the loaded rule set as an immutable shared slice.
    pub fn rules(&self) -> &[String] {
        &self.rules
    }
}

/// Load a `ZiraConfig` from `path`.
///
/// * If `path` does not exist, returns `Ok(ZiraConfig::default())`.
/// * If `path` exists but is malformed TOML, returns `Err(ConfigError::Parse(...))`.
/// * Absent keys in a partial file fall back to their serde defaults.
pub fn load_from(path: &std::path::Path) -> Result<ZiraConfig, ConfigError> {
    if !path.exists() {
        return Ok(ZiraConfig::default());
    }
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
    toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZiraConfig {
    pub paths: PathsConfig,
    pub model: ModelConfig,
    pub wakeword: WakewordConfig,
    pub vad: VadConfig,
    pub stt: SttConfig,
    pub tts: TtsConfig,
    pub emotion: EmotionConfig,
    pub memory: MemoryConfig,
    pub avatar: AvatarConfig,
}

impl ZiraConfig {
    /// Validate the config, reporting (never repairing) the first invalid field.
    ///
    /// The shipped default is the *unconfigured* state — a fully-empty TOML document
    /// deserializes to it — and must validate `Ok`. Requirements that depend on a
    /// subsystem being in use are therefore gated on that subsystem being configured:
    ///
    /// * Thresholds are always range-checked against `[0.0, 1.0]`; the default `0.0`
    ///   is in range, so an unconfigured config still passes.
    /// * Once a model is named (`model.model_id` is non-empty), the brain is in use, so
    ///   `model.binary_path` must be a non-empty path and the TTS `sample_rate` must be
    ///   a positive number of Hz.
    ///
    /// Each rejection maps to a distinct [`ConfigError`] variant naming the offending
    /// field and the reason.
    pub fn validate(&self) -> Result<(), ConfigError> {
        check_threshold("wakeword.threshold", self.wakeword.threshold)?;
        check_threshold("vad.threshold", self.vad.threshold)?;

        if !self.model.model_id.is_empty() {
            if self.model.binary_path.is_empty() {
                return Err(ConfigError::EmptyPath {
                    field: "model.binary_path",
                });
            }
            if self.tts.sample_rate == 0 {
                return Err(ConfigError::InvalidSampleRate {
                    field: "tts.sample_rate",
                    value: self.tts.sample_rate,
                });
            }
        }

        Ok(())
    }
}

/// Resolve the configured default emotion string to a typed `Emotion`.
pub fn resolve_default_emotion(config: &EmotionConfig) -> zira_proto::Emotion {
    zira_proto::Emotion::from_tag(&config.default_emotion)
}

/// Validate a slice of emotion-tag strings against the ten known `Emotion` variants.
///
/// Each tag is matched case-insensitively. Returns the resolved `Vec<Emotion>` on
/// success, or `Err(VocabError::UnknownTag)` naming the first tag that matches no
/// variant. An empty slice is valid and yields an empty `Vec`.
pub fn validate_vocab(_tags: &[String]) -> Result<Vec<zira_proto::Emotion>, VocabError> {
    todo!("T-05.06: implement validate_vocab")
}

/// Range-check a probability threshold, which must lie within `[0.0, 1.0]`.
fn check_threshold(field: &'static str, value: f32) -> Result<(), ConfigError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ConfigError::ThresholdOutOfRange { field, value });
    }
    Ok(())
}
