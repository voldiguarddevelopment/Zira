//! zira-skills — skill/MCP staging, signing, audit log.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

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

/// Compute an HMAC-SHA256 over a deterministic serialization of `m` keyed by `key`.
pub fn sign_manifest(key: &[u8], m: &SkillManifest) -> Signature {
    let payload = serde_json::to_vec(m).expect("SkillManifest is always serializable");
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(&payload);
    Signature::new(mac.finalize().into_bytes().to_vec())
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
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Decode a lowercase hex string into a [`Signature`].
    pub fn from_hex(s: &str) -> Result<Self, ManifestError> {
        if s.len() % 2 != 0 {
            return Err(ManifestError::Parse(format!(
                "odd-length hex string: {} chars",
                s.len()
            )));
        }
        let bytes = (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| {
                    ManifestError::Parse(format!("invalid hex at position {i}: {:?}", &s[i..i + 2]))
                })
            })
            .collect::<Result<Vec<u8>, ManifestError>>()?;
        Ok(Self(bytes))
    }
}
