//! zira-skills — skill/MCP staging, signing, audit log.

use serde::{Deserialize, Serialize};

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
