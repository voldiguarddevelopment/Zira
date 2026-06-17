//! zira-config — config schema, TOML load, XDG paths, constitution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathsConfig {
    pub config_dir: Option<String>,
    pub data_dir: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_id: String,
    pub binary_path: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct WakewordConfig {
    pub model_path: String,
    pub threshold: f32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct VadConfig {
    pub threshold: f32,
    pub silence_ms: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SttConfig {
    pub model_path: String,
    pub language: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TtsConfig {
    pub model_path: String,
    pub voice: String,
    pub sample_rate: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmotionConfig {
    pub default_emotion: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_episodes: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvatarConfig {
    pub vrm_path: Option<String>,
}

// ZiraConfig fields are not yet marked #[serde(default)], so toml::from_str("") fails
// with a missing-field error. Adding #[serde(default)] to each field is the GREEN task.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
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
