//! zira-config — config schema, TOML load, XDG paths, constitution.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// Errors that `load_from` can return (missing file is NOT an error — it yields the default).
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("malformed TOML in config file: {0}")]
    Parse(String),
    #[error("I/O error reading config file: {0}")]
    Io(String),
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
