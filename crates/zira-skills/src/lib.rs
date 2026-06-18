//! zira-skills — skill/MCP staging, signing, audit log.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The data record every skill safety check reads.
///
/// Declares a skill's name, version, entry point, requested capabilities,
/// and allowed filesystem roots. Empty vecs are legal here; default-deny
/// enforcement is applied downstream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub capabilities: Vec<String>,
    pub allowed_roots: Vec<String>,
}

/// Typed errors for manifest parsing and validation.
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("parse error: {0}")]
    Parse(String),
}

/// Parse a TOML-encoded manifest string into a [`SkillManifest`].
pub fn parse_manifest_toml(text: &str) -> Result<SkillManifest, ManifestError> {
    toml::from_str(text).map_err(|e| ManifestError::Parse(e.to_string()))
}

/// Parse a JSON-encoded manifest string into a [`SkillManifest`].
pub fn parse_manifest_json(text: &str) -> Result<SkillManifest, ManifestError> {
    serde_json::from_str(text).map_err(|e| ManifestError::Parse(e.to_string()))
}
