//! Frozen tests for T-02.14 — "Add a vector".
//!
//! C1 — `VectorIndex::new()` builds an empty index (len == 0); `add(&mut self, id, vector)`
//!       stores the (id, vector) pair; `len(&self)` returns the count of stored vectors.
//! C2 — building an index, adding three vectors with distinct ids, and asserting len is 3.
//!
//! Acceptance criterion mapping:
//!
//!   C1 -> c1_new_builds_empty_index, c1_add_one_vector_len_is_one
//!   C2 -> c2_add_three_vectors_len_is_three

use zira_memory::VectorIndex;

// ---- C1 -----------------------------------------------------------------------

/// A freshly created `VectorIndex` must be empty: `len()` returns 0.
#[test]
fn c1_new_builds_empty_index() {
    let index = VectorIndex::new();
    assert_eq!(index.len(), 0, "a new VectorIndex must have len 0");
}

/// Adding one vector to an empty index must make `len()` return 1.
#[test]
fn c1_add_one_vector_len_is_one() {
    let mut index = VectorIndex::new();
    index.add(0, vec![1.0, 0.0, 0.0]);
    assert_eq!(index.len(), 1, "after one add, len must be 1");
}

// ---- C2 -----------------------------------------------------------------------

/// Building an index, adding three vectors with distinct ids, and asserting len is 3.
#[test]
fn c2_add_three_vectors_len_is_three() {
    let mut index = VectorIndex::new();
    index.add(10, vec![1.0, 0.0, 0.0]);
    index.add(20, vec![0.0, 1.0, 0.0]);
    index.add(30, vec![0.0, 0.0, 1.0]);
    assert_eq!(index.len(), 3, "after three adds with distinct ids, len must be 3");
}
