//! zira-memory — on-disk memory: episodic, facts, vector index.

use serde::{Deserialize, Serialize};

/// Typed errors for the fact store.
#[derive(thiserror::Error, Debug)]
pub enum FactStoreError {
    #[error("database open failed: {0}")]
    OpenFailed(String),
    #[error("transaction failed: {0}")]
    TransactionFailed(String),
    #[error("serialization failed: {0}")]
    SerializeFailed(String),
}

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

const FACTS_TABLE: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("facts");

/// A handle to the redb-backed fact store.
pub struct FactStore {
    db: redb::Database,
}

impl FactStore {
    /// Opens (creating if absent) a redb database at `path`.
    pub fn open(path: &std::path::Path) -> Result<Self, FactStoreError> {
        let db = redb::Database::create(path)
            .map_err(|e| FactStoreError::OpenFailed(e.to_string()))?;
        Ok(Self { db })
    }

    /// Commits a `key -> value` entry to the redb store durably.
    /// Putting an existing key overwrites the prior value.
    pub fn put(&self, key: &str, value: &str) -> Result<(), FactStoreError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(FACTS_TABLE)
                .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
            table
                .insert(key, value)
                .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        Ok(())
    }

    /// Removes the entry for `key`; deleting an absent key is `Ok(())` (idempotent).
    pub fn delete(&self, key: &str) -> Result<(), FactStoreError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(FACTS_TABLE)
                .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
            table
                .remove(key)
                .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        Ok(())
    }

    /// Returns the value for `key`, or `Ok(None)` if the key is absent.
    /// A missing key is never an error variant.
    pub fn get(&self, key: &str) -> Result<Option<String>, FactStoreError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| FactStoreError::TransactionFailed(e.to_string()))?;
        let table = match read_txn.open_table(FACTS_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(FactStoreError::TransactionFailed(e.to_string())),
        };
        match table.get(key).map_err(|e| FactStoreError::TransactionFailed(e.to_string()))? {
            Some(guard) => Ok(Some(guard.value().to_owned())),
            None => Ok(None),
        }
    }
}
