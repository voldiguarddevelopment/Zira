//! RED-phase frozen tests for T-02.19 — "Load the embedding model".
//!
//! These reference `zira_memory::CandleEmbedder` and `zira_memory::EmbedderError`,
//! which do not exist yet; until they are implemented this file fails to compile —
//! that IS the red state.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> _c1_assert_api (compile-time), c1_c2_c3_c5_real_model_embeds
//!   C2 -> c1_c2_c3_c5_real_model_embeds
//!   C3 -> c1_c2_c3_c5_real_model_embeds
//!   C4 -> c4_embedder_error_display_variants
//!   C5 -> c1_c2_c3_c5_real_model_embeds
//!
//! The integration test loads the real on-disk model from `$ZIRA_EMBED_MODEL`
//! (default `~/.cache/zira/models/all-MiniLM-L6-v2`) and RETURNS EARLY when that
//! model is absent, so a model-less CI stays green. The C4 Display test needs no
//! model and always runs.

use std::path::{Path, PathBuf};

use zira_memory::{CandleEmbedder, Embedder, EmbedderError};

// ---- helpers ------------------------------------------------------------------

/// Resolves the model directory: `$ZIRA_EMBED_MODEL` if set, else the default
/// `~/.cache/zira/models/all-MiniLM-L6-v2`.
fn model_dir() -> PathBuf {
    if let Ok(p) = std::env::var("ZIRA_EMBED_MODEL") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Path::new(&home).join(".cache/zira/models/all-MiniLM-L6-v2")
}

/// True only when all three required model assets are present on disk.
fn model_present(dir: &Path) -> bool {
    dir.join("config.json").is_file()
        && dir.join("tokenizer.json").is_file()
        && dir.join("model.safetensors").is_file()
}

/// Euclidean (L2) norm of a vector.
fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

// ---- C1 -----------------------------------------------------------------------

/// C1 (compile-time): `CandleEmbedder::load` has the exact signature
/// `fn(&Path) -> Result<CandleEmbedder, EmbedderError>`, and `CandleEmbedder`
/// implements `Embedder`. Never called — the coercions alone pin the API.
#[allow(dead_code)]
fn _c1_assert_api() {
    fn is_embedder<T: Embedder>() {}
    is_embedder::<CandleEmbedder>();

    let _load: fn(&Path) -> Result<CandleEmbedder, EmbedderError> = CandleEmbedder::load;
}

// ---- C1 + C2 + C3 + C5 --------------------------------------------------------

/// C1: loads a real BERT sentence-embedding model from `model_dir` on the CPU.
/// C2: `dim()` is the model hidden size; `embed(text)` returns a `Vec<f32>` of
///     length `dim()`.
/// C3: two distinct sentences each embed to a `dim()`-length, non-zero vector,
///     and the two vectors differ — proving real weights, not the hash stand-in.
/// C5: each embedding's L2 norm lies within `0.5..=30.0` — a raw mean-pool gives
///     ~5–8, but a div→mul magnitude error scales it to >30, so this bound pins
///     the pooling arithmetic.
///
/// Returns early (passing) when the model is absent, keeping a model-less CI green.
#[test]
fn c1_c2_c3_c5_real_model_embeds() {
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!(
            "skipping c1_c2_c3_c5_real_model_embeds: model not present at {}",
            dir.display()
        );
        return;
    }

    let embedder = CandleEmbedder::load(&dir).expect("loading the on-disk model must succeed");

    // C2: dim() is the positive model hidden size.
    let dim = embedder.dim();
    assert!(dim > 0, "dim() must return the positive model hidden size, got {dim}");

    let s1 = "The cat dozed on the warm windowsill all afternoon.";
    let s2 = "Quantum entanglement correlates two distant particles.";
    let v1 = embedder.embed(s1);
    let v2 = embedder.embed(s2);

    // C2 + C3: embed() length equals dim() (ties dim() to the real tensor width).
    assert_eq!(v1.len(), dim, "embed(s1) length must equal dim() = {dim}");
    assert_eq!(v2.len(), dim, "embed(s2) length must equal dim() = {dim}");

    // C3: real weights produce non-zero vectors (the hash stand-in is excluded by
    // the cross-sentence difference and norm bounds below).
    assert!(
        v1.iter().any(|&x| x != 0.0),
        "embed(s1) must be non-zero — real model weights, not a zero stub"
    );
    assert!(
        v2.iter().any(|&x| x != 0.0),
        "embed(s2) must be non-zero — real model weights, not a zero stub"
    );

    // C3: distinct sentences must yield distinct vectors.
    assert_ne!(v1, v2, "two distinct sentences must embed to different vectors");

    // C5: un-normalized mean-pooled norm sits in 0.5..=30.0; a seq-scaled (div→mul)
    // pooling error blows the norm past 30 and fails here.
    let n1 = l2_norm(&v1);
    let n2 = l2_norm(&v2);
    assert!(
        (0.5..=30.0).contains(&n1),
        "embed(s1) L2 norm {n1} must lie in 0.5..=30.0 (raw mean-pool, not seq-scaled)"
    );
    assert!(
        (0.5..=30.0).contains(&n2),
        "embed(s2) L2 norm {n2} must lie in 0.5..=30.0 (raw mean-pool, not seq-scaled)"
    );
}

// ---- C4 -----------------------------------------------------------------------

/// C4: `EmbedderError` implements `std::error::Error` + `Display` with distinct
/// variants for a missing model file, a tokenizer-load failure, and a
/// model-weights load failure; every variant's `Display` is exercised, names its
/// failure kind, and the three messages are mutually distinct.
#[test]
fn c4_embedder_error_display_variants() {
    fn assert_is_error<E: std::error::Error>(_: &E) {}

    // Neutral context strings containing none of the failure-kind keywords, so any
    // keyword match comes from the format string itself.
    let missing = EmbedderError::MissingModelFile("ctx-alpha".to_string());
    let tokenizer = EmbedderError::TokenizerLoad("ctx-beta".to_string());
    let weights = EmbedderError::ModelLoad("ctx-gamma".to_string());

    // Error bound, checked at compile time.
    assert_is_error(&missing);
    assert_is_error(&tokenizer);
    assert_is_error(&weights);

    let m = missing.to_string();
    let t = tokenizer.to_string();
    let w = weights.to_string();

    // Non-empty.
    assert!(!m.is_empty(), "MissingModelFile Display must not be empty");
    assert!(!t.is_empty(), "TokenizerLoad Display must not be empty");
    assert!(!w.is_empty(), "ModelLoad Display must not be empty");

    // Names its failure kind.
    let ml = m.to_lowercase();
    assert!(
        ml.contains("missing") || ml.contains("not found") || ml.contains("no such"),
        "MissingModelFile Display must name the missing-file failure, got: {m:?}"
    );
    let tl = t.to_lowercase();
    assert!(
        tl.contains("tokenizer"),
        "TokenizerLoad Display must name the tokenizer failure, got: {t:?}"
    );
    let wl = w.to_lowercase();
    assert!(
        wl.contains("weights") || wl.contains("model"),
        "ModelLoad Display must name the model-weights failure, got: {w:?}"
    );

    // Mutually distinct — every Display arm is its own message.
    assert_ne!(m, t, "MissingModelFile and TokenizerLoad must differ");
    assert_ne!(m, w, "MissingModelFile and ModelLoad must differ");
    assert_ne!(t, w, "TokenizerLoad and ModelLoad must differ");
}
