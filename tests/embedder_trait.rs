//! RED-phase tests for T-02.11: `zira_memory::Embedder` trait.
//!
//! These tests compile only once `Embedder` is defined in `zira-memory`.
//! Until then, the missing trait causes a build failure — that IS the red state.

use zira_memory::Embedder;

/// Minimal in-test implementor: always returns a vec of `DIM` zeros.
const DIM: usize = 4;

struct ZeroEmbedder;

impl Embedder for ZeroEmbedder {
    fn dim(&self) -> usize {
        DIM
    }

    fn embed(&self, _text: &str) -> Vec<f32> {
        vec![0.0; DIM]
    }
}

/// C1 + C2: the trait exists with the required method signatures, and a
/// concrete in-test implementor satisfies them.  The returned vec length
/// must equal `dim()`.
#[test]
fn test_embed_len_matches_dim() {
    let embedder = ZeroEmbedder;
    let v = embedder.embed("hello world");
    assert_eq!(
        v.len(),
        embedder.dim(),
        "embed() result length must equal dim()"
    );
}

/// C1: `embed` and `dim` are usable through a trait object (`dyn Embedder`),
/// confirming the trait is object-safe.
#[test]
fn test_embedder_trait_object_safe() {
    let embedder: Box<dyn Embedder> = Box::new(ZeroEmbedder);
    assert_eq!(embedder.embed("test").len(), embedder.dim());
}
