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
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("io error: {0}")]
    Io(String),
}

/// Parse a TOML-encoded manifest string into a [`SkillManifest`].
pub fn parse_manifest_toml(text: &str) -> Result<SkillManifest, ManifestError> {
    toml::from_str(text).map_err(|e| ManifestError::Parse(e.to_string()))
}

/// Parse a JSON-encoded manifest string into a [`SkillManifest`].
pub fn parse_manifest_json(text: &str) -> Result<SkillManifest, ManifestError> {
    serde_json::from_str(text).map_err(|e| ManifestError::Parse(e.to_string()))
}

/// The serialized form of an HMAC tag — a carrier for raw bytes with hex I/O.
#[derive(Debug, Clone, PartialEq)]
pub struct Signature(Vec<u8>);

impl Signature {
    /// Construct a [`Signature`] from raw HMAC bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Encode the raw bytes as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        todo!("implement hex encoding")
    }

    /// Decode a lowercase hex string into a [`Signature`].
    pub fn from_hex(s: &str) -> Result<Self, ManifestError> {
        todo!("implement hex decoding")
    }
}
