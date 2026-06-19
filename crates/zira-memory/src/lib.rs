//! zira-memory — on-disk memory: episodic, facts, vector index.

use serde::{Deserialize, Serialize};

/// Seam between retrieval logic and the embedding model.
///
/// Every implementation must guarantee that `embed(text).len() == dim()`
/// for all inputs, including empty strings.
pub trait Embedder {
    /// Returns the fixed dimensionality of all vectors produced by this embedder.
    fn dim(&self) -> usize;

    /// Produces a fixed-length embedding vector for `text`.
    fn embed(&self, text: &str) -> Vec<f32>;
}

/// Deterministic hash-based embedder for use in tests and offline tooling.
///
/// Produces reproducible vectors without external model weights or downloads.
/// Same input always yields the same vector; distinct inputs yield distinct vectors.
pub struct HashEmbedder {
    dim: usize,
}

impl HashEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }
}

impl Embedder for HashEmbedder {
    fn dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        (0..self.dim)
            .map(|i| {
                let h = hash_slot(i as u64, text.as_bytes());
                h as f32
            })
            .collect()
    }
}

/// FNV-1a 64-bit hash with a per-slot seed mixed in first.
fn hash_slot(slot: u64, data: &[u8]) -> u64 {
    const OFFSET: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    let mut h = OFFSET;
    for byte in slot.to_le_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(PRIME);
    }
    for &byte in data {
        h ^= byte as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Typed errors raised while loading or running the candle BERT embedder.
///
/// Each variant carries a context string (the offending path or the underlying
/// error text) so a failure points at its cause.
#[derive(thiserror::Error, Debug)]
pub enum EmbedderError {
    /// A required model asset (`config.json`, `tokenizer.json`, or
    /// `model.safetensors`) was absent from the model directory.
    #[error("missing model file: {0}")]
    MissingModelFile(String),
    /// The tokenizer (`tokenizer.json`) could not be parsed/loaded.
    #[error("tokenizer load failed: {0}")]
    TokenizerLoad(String),
    /// The model weights (`config.json` parse or `model.safetensors`) could not
    /// be loaded into the BERT model.
    #[error("model weights load failed: {0}")]
    ModelLoad(String),
}

/// A CPU BERT sentence-embedding model loaded from disk via candle-transformers.
///
/// Loads `config.json` + `tokenizer.json` + `model.safetensors` from a directory
/// (placed there out-of-band — never downloaded in-code, never committed) and
/// produces mean-pooled last-hidden-state embeddings through the [`Embedder`]
/// trait. CPU-only: no CUDA, no quantization, no conversion.
pub struct CandleEmbedder {
    model: candle_transformers::models::bert::BertModel,
    tokenizer: tokenizers::Tokenizer,
    dim: usize,
}

impl CandleEmbedder {
    /// Loads the BERT sentence-embedding model from `model_dir` on the CPU.
    ///
    /// Expects `config.json`, `tokenizer.json`, and `model.safetensors` to be
    /// present. Returns an [`EmbedderError`] variant naming which stage failed.
    pub fn load(model_dir: &std::path::Path) -> Result<CandleEmbedder, EmbedderError> {
        use candle_transformers::models::bert::{BertModel, Config, DTYPE};

        let config_path = model_dir.join("config.json");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let weights_path = model_dir.join("model.safetensors");

        for path in [&config_path, &tokenizer_path, &weights_path] {
            if !path.is_file() {
                return Err(EmbedderError::MissingModelFile(path.display().to_string()));
            }
        }

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| EmbedderError::TokenizerLoad(e.to_string()))?;

        let config_text = std::fs::read_to_string(&config_path)
            .map_err(|e| EmbedderError::ModelLoad(e.to_string()))?;
        let config: Config = serde_json::from_str(&config_text)
            .map_err(|e| EmbedderError::ModelLoad(e.to_string()))?;
        let dim = config.hidden_size;

        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(
                &[weights_path],
                DTYPE,
                &candle_core::Device::Cpu,
            )
            .map_err(|e| EmbedderError::ModelLoad(e.to_string()))?
        };
        let model = BertModel::load(vb, &config)
            .map_err(|e| EmbedderError::ModelLoad(e.to_string()))?;

        Ok(CandleEmbedder {
            model,
            tokenizer,
            dim,
        })
    }

    /// Runs the model for `text` and returns the mean-pooled embedding.
    ///
    /// Separated from the trait `embed` so candle/tokenizer failures surface as
    /// errors here; `embed` preserves the `len() == dim()` invariant on failure.
    fn embed_inner(&self, text: &str) -> candle_core::Result<Vec<f32>> {
        use candle_core::{Device, Tensor};

        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| candle_core::Error::Msg(e.to_string()))?;

        let ids = encoding.get_ids();
        let seq = ids.len();

        let token_ids = Tensor::new(ids, &Device::Cpu)?.unsqueeze(0)?;
        let type_ids = token_ids.zeros_like()?;
        let attn = Tensor::new(encoding.get_attention_mask(), &Device::Cpu)?.unsqueeze(0)?;

        let out = self.model.forward(&token_ids, &type_ids, Some(&attn))?;
        let pooled = (out.sum(1)? / seq as f64)?;
        pooled.squeeze(0)?.to_vec1::<f32>()
    }
}

impl Embedder for CandleEmbedder {
    fn dim(&self) -> usize {
        self.dim
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        match self.embed_inner(text) {
            Ok(v) if v.len() == self.dim => v,
            // Preserve the trait invariant `embed().len() == dim()` even on the
            // unexpected path; honest models on real weights take the Ok arm.
            _ => vec![0.0; self.dim],
        }
    }
}

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
        Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => Ok(vec![]),
        Err(e) => Err(e),
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

/// Computes the cosine similarity of two equal-length `f32` vectors.
///
/// **Precondition:** `a.len() == b.len()`. Callers must pass equal-length slices.
///
/// Returns a value in `[-1.0, 1.0]`. A zero-magnitude input yields `0.0`
/// (divide-by-zero guard — never NaN).
///
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    let denom = mag_a.mul_add(mag_b, 0.0);
    if denom == 0.0 {
        0.0
    } else {
        dot.mul_add(denom.recip(), 0.0)
    }
}

/// In-memory vector index: stores (id, vector) pairs and reports their count.
///
/// Insertion-only; the index is rebuilt from the episode/fact store on each run.
pub struct VectorIndex {
    entries: Vec<(usize, Vec<f32>)>,
}

impl Default for VectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorIndex {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, id: usize, vector: Vec<f32>) {
        self.entries.push((id, vector));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns up to `k` `(id, score)` pairs sorted by descending cosine similarity
    /// to `query`. Returns an empty vec when `k == 0`; saturates at `len()` when
    /// `k` exceeds the number of stored vectors.
    pub fn search(&self, query: &[f32], k: usize) -> Vec<(usize, f32)> {
        if k == 0 {
            return Vec::new();
        }
        let mut scored: Vec<(usize, f32)> = self
            .entries
            .iter()
            .map(|(id, vec)| (*id, cosine_similarity(query, vec)))
            .collect();
        scored.sort_unstable_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }
}

/// Loads the episodes at `path`, embeds each one plus the `query` via `embedder`,
/// and returns the top-`k` episodes sorted by descending cosine similarity to the query.
///
/// A missing or empty file returns `Ok(vec![])`.  The index is ephemeral — computed
/// per call; nothing is persisted.
pub fn retrieve(
    path: &std::path::Path,
    embedder: &impl Embedder,
    query: &str,
    k: usize,
) -> std::io::Result<Vec<Episode>> {
    let episodes = load_episodes(path)?;
    if episodes.is_empty() {
        return Ok(vec![]);
    }

    let query_vec = embedder.embed(query);

    let mut index = VectorIndex::new();
    for (i, ep) in episodes.iter().enumerate() {
        index.add(i, embedder.embed(&ep.text));
    }

    let hits = index.search(&query_vec, k);
    Ok(hits.into_iter().map(|(id, _score)| episodes[id].clone()).collect())
}

/// Renders a slice of retrieved episodes into a single prompt-preamble string.
///
/// An empty slice returns an empty string so no noise is injected when there is
/// nothing to recall.
pub fn format_preamble(episodes: &[Episode]) -> String {
    if episodes.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for ep in episodes {
        out.push_str(&format!("[{}] {}\n", ep.role, ep.text));
    }
    out
}

/// Stateless consolidation pass: derives deduplicated facts from the episodic log
/// and writes each into `store` using the episode text as the key.
///
/// Returns the count of unique facts written. A missing or empty episode file
/// writes nothing and returns `Ok(0)`.
pub fn consolidate(
    episode_path: &std::path::Path,
    store: &FactStore,
) -> Result<usize, FactStoreError> {
    let episodes = load_episodes(episode_path)
        .map_err(|e| FactStoreError::OpenFailed(e.to_string()))?;

    let mut seen = std::collections::HashSet::new();
    let mut count = 0usize;

    for ep in &episodes {
        if seen.insert(ep.text.clone()) {
            store.put(&ep.text, &ep.text)?;
            count += 1;
        }
    }

    Ok(count)
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
