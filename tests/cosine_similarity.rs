//! RED-phase tests for T-02.13: `zira_memory::cosine_similarity`.
//!
//! `cosine_similarity` is not yet implemented; these tests will panic at
//! runtime until the implementation is provided — that IS the red state.

use zira_memory::cosine_similarity;

const EPSILON: f32 = 1e-6;

/// C1 + C2: cosine similarity of a vector with itself is ~1.0.
#[test]
fn test_cosine_similarity_self_is_one() {
    let v = vec![1.0_f32, 0.0, 0.0];
    let result = cosine_similarity(&v, &v);
    assert!(
        (result - 1.0).abs() < EPSILON,
        "self-similarity must be ~1.0, got {result}"
    );
}

/// C1 + C2: cosine similarity of two orthogonal vectors is ~0.0.
#[test]
fn test_cosine_similarity_orthogonal_is_zero() {
    let a = vec![1.0_f32, 0.0, 0.0];
    let b = vec![0.0_f32, 1.0, 0.0];
    let result = cosine_similarity(&a, &b);
    assert!(
        result.abs() < EPSILON,
        "orthogonal vectors must have similarity ~0.0, got {result}"
    );
}

/// C2: cosine similarity of a vector and its negation is ~-1.0.
#[test]
fn test_cosine_similarity_opposite_is_neg_one() {
    let a = vec![1.0_f32, 0.0, 0.0];
    let b = vec![-1.0_f32, 0.0, 0.0];
    let result = cosine_similarity(&a, &b);
    assert!(
        (result - (-1.0)).abs() < EPSILON,
        "opposite vectors must have similarity ~-1.0, got {result}"
    );
}

/// C3: a zero-magnitude vector yields 0.0, never NaN.
#[test]
fn test_cosine_similarity_zero_vector_guard() {
    let zero = vec![0.0_f32, 0.0, 0.0];
    let other = vec![1.0_f32, 0.0, 0.0];
    let result = cosine_similarity(&zero, &other);
    assert!(
        !result.is_nan(),
        "zero vector must not produce NaN, got {result}"
    );
    assert_eq!(result, 0.0, "zero vector must yield 0.0, got {result}");
}
