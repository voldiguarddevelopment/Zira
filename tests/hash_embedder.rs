//! RED-phase tests for T-02.12: `zira_memory::HashEmbedder`.
//!
//! `HashEmbedder` does not exist yet; these tests will fail to compile until
//! the implementation is added — that IS the red state.

use zira_memory::{Embedder, HashEmbedder};

/// C1 + C3: `HashEmbedder` implements `Embedder`; `dim()` returns the configured
/// dimension; and every `embed` output vector has length equal to `dim()`.
#[test]
fn test_hash_embedder_dim_matches_configured() {
    let embedder = HashEmbedder::new(64);
    assert_eq!(embedder.dim(), 64, "dim() must return the configured dimension");

    let v = embedder.embed("some text");
    assert_eq!(
        v.len(),
        embedder.dim(),
        "embed() output length must equal dim()"
    );
}

/// C3: embed output length equals dim() for a variety of inputs, including the
/// empty string edge case explicitly called out in the spec.
#[test]
fn test_hash_embedder_embed_len_matches_dim() {
    let embedder = HashEmbedder::new(32);
    for text in &["", "hello", "a longer sentence with several words"] {
        let v = embedder.embed(text);
        assert_eq!(
            v.len(),
            embedder.dim(),
            "embed({:?}).len() must equal dim() = {}",
            text,
            embedder.dim()
        );
    }
}

/// C2: the same input text always produces the same output vector (determinism),
/// and two distinct input texts produce different output vectors (distinctness).
#[test]
fn test_hash_embedder_deterministic_and_distinct() {
    let embedder = HashEmbedder::new(16);

    let v1a = embedder.embed("hello world");
    let v1b = embedder.embed("hello world");
    assert_eq!(
        v1a, v1b,
        "embed() must be deterministic: same input must yield equal vectors"
    );

    let v2 = embedder.embed("completely different text");
    assert_ne!(
        v1a, v2,
        "embed() must be distinct: different inputs must yield different vectors"
    );
}

/// C1: `HashEmbedder` is usable as a trait object (`dyn Embedder`), confirming it
/// satisfies the trait contract through dynamic dispatch.
#[test]
fn test_hash_embedder_as_trait_object() {
    let embedder: Box<dyn Embedder> = Box::new(HashEmbedder::new(8));
    let v = embedder.embed("trait object test");
    assert_eq!(
        v.len(),
        embedder.dim(),
        "embed() via trait object must still produce dim()-length vector"
    );
}
