//! Frozen tests for T-02.15 — "Search the top-k vectors".
//!
//! C1 — `VectorIndex::search(&self, query: &[f32], k: usize) -> Vec<(usize, f32)>`
//!       returns up to `k` `(id, score)` pairs sorted by descending cosine similarity.
//! C2 — adds several vectors, searches with a query nearest one known id, asserts that
//!       id is the first result and results are in non-increasing score order.
//! C3 — `search` with `k` greater than index size returns all stored vectors (saturates
//!       at `len()`); `k` of 0 returns an empty vec.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_search_respects_k_limit, c1_search_returns_id_score_pairs
//!   C2 -> c2_nearest_id_is_first, c2_results_in_non_increasing_score_order
//!   C3 -> c3_k_over_capacity_returns_all, c3_k_zero_returns_empty

use zira_memory::VectorIndex;

// ---- C1 -----------------------------------------------------------------------

/// `search` returns exactly `k` pairs when the index holds more than `k` vectors.
#[test]
fn c1_search_respects_k_limit() {
    let mut index = VectorIndex::new();
    index.add(1, vec![1.0, 0.0, 0.0]);
    index.add(2, vec![0.0, 1.0, 0.0]);
    index.add(3, vec![0.0, 0.0, 1.0]);

    let results = index.search(&[1.0, 0.0, 0.0], 2);
    assert_eq!(
        results.len(),
        2,
        "search(k=2) over 3 vectors must return exactly 2 pairs"
    );
}

/// `search` returns `(id, score)` pairs — the `usize` matches the stored id and the
/// `f32` is the cosine-similarity score relative to the query.
#[test]
fn c1_search_returns_id_score_pairs() {
    let mut index = VectorIndex::new();
    index.add(42, vec![1.0, 0.0]);

    let results = index.search(&[1.0, 0.0], 1);
    assert_eq!(results.len(), 1, "single-entry index returns one result for k=1");
    let (id, score) = results[0];
    assert_eq!(id, 42, "returned id must match the stored id");
    assert!(
        (score - 1.0).abs() < 1e-6,
        "self-similarity must be ~1.0, got {score}"
    );
}

// ---- C2 -----------------------------------------------------------------------

/// Adds three orthogonal unit vectors with distinct ids; a query aligned to id 100
/// must have that id ranked first.
#[test]
fn c2_nearest_id_is_first() {
    let mut index = VectorIndex::new();
    index.add(100, vec![1.0, 0.0, 0.0]);
    index.add(200, vec![0.0, 1.0, 0.0]);
    index.add(300, vec![0.0, 0.0, 1.0]);

    let results = index.search(&[1.0, 0.0, 0.0], 3);
    assert!(!results.is_empty(), "search must return at least one result");
    assert_eq!(
        results[0].0, 100,
        "the vector nearest the query ([1,0,0]) must be id 100"
    );
}

/// Results must appear in non-increasing score order across the full result set.
///
/// Three 2-D vectors span the cosine range perfectly:
///   id=10 sim=1.0 (identical to query), id=20 sim=0.0 (orthogonal),
///   id=30 sim=-1.0 (opposite).
#[test]
fn c2_results_in_non_increasing_score_order() {
    let mut index = VectorIndex::new();
    index.add(10, vec![1.0, 0.0]);
    index.add(20, vec![0.0, 1.0]);
    index.add(30, vec![-1.0, 0.0]);

    let results = index.search(&[1.0, 0.0], 3);
    assert_eq!(results.len(), 3, "all three vectors must be returned for k=3");

    for window in results.windows(2) {
        assert!(
            window[0].1 >= window[1].1,
            "scores must be non-increasing: {} then {}",
            window[0].1,
            window[1].1
        );
    }
    assert_eq!(results[0].0, 10, "id 10 (sim ~1.0) must be ranked first");
}

// ---- C3 -----------------------------------------------------------------------

/// `search` with `k` larger than the number of stored vectors returns all of them
/// (length saturates at `len()` rather than padding or panicking).
#[test]
fn c3_k_over_capacity_returns_all() {
    let mut index = VectorIndex::new();
    index.add(1, vec![1.0, 0.0]);
    index.add(2, vec![0.0, 1.0]);

    let results = index.search(&[1.0, 0.0], 999);
    assert_eq!(
        results.len(),
        2,
        "k=999 over a 2-entry index must return exactly 2 results, got {}",
        results.len()
    );
}

/// `search` with `k = 0` returns an empty vec regardless of index contents.
#[test]
fn c3_k_zero_returns_empty() {
    let mut index = VectorIndex::new();
    index.add(1, vec![1.0, 0.0]);
    index.add(2, vec![0.0, 1.0]);

    let results = index.search(&[1.0, 0.0], 0);
    assert!(results.is_empty(), "search(k=0) must return an empty vec");
}
