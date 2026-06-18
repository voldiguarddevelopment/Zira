//! zira-memory — on-disk memory: episodic, facts, vector index.

use serde::{Deserialize, Serialize};

/// A single conversational episode stored in episodic memory.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Episode {
    pub role: String,
    pub text: String,
    pub timestamp: u64,
}
