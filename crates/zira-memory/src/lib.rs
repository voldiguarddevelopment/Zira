//! zira-memory — on-disk memory: episodic, facts, vector index.

use serde::{Deserialize, Serialize};

/// A single conversational episode stored in episodic memory.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Episode {
    pub role: String,
    pub text: String,
    pub timestamp: u64,
}

pub fn load_episodes(path: &std::path::Path) -> std::io::Result<Vec<Episode>> {
    use std::io::BufRead;
    match std::fs::File::open(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e),
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            reader
                .lines()
                .filter(|line| line.as_ref().map(|l| !l.is_empty()).unwrap_or(true))
                .map(|line| {
                    let line = line?;
                    serde_json::from_str(&line)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })
                .collect()
        }
    }
}

pub fn cap_episodes(path: &std::path::Path, max_episodes: usize) -> std::io::Result<()> {
    let episodes = load_episodes(path)?;
    let start = episodes.len().saturating_sub(max_episodes);
    let retained = &episodes[start..];
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    use std::io::Write;
    for ep in retained {
        let line = serde_json::to_string(ep)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(file, "{}", line)?;
    }
    Ok(())
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
