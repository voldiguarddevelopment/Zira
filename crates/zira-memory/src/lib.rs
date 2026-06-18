//! zira-memory — on-disk memory: episodic, facts, vector index.

use serde::{Deserialize, Serialize};

/// A single conversational episode stored in episodic memory.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Episode {
    pub role: String,
    pub text: String,
    pub timestamp: u64,
}

pub fn append_episode(path: &std::path::Path, episode: &Episode) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(episode)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    writeln!(file, "{}", line)
}
