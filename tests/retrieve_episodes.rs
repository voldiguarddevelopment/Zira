//! Frozen tests for T-02.16 — "Retrieve the relevant episodes".
//!
//! C1 — `zira_memory::retrieve(path, embedder, query, k)` loads the episodes,
//!       embeds each plus the query via the embedder, and returns the top-k
//!       episodes by cosine similarity to the query.
//! C2 — a test with near and far episodes using HashEmbedder asserts that k=1
//!       returns the episode whose text is identical to the query (cosine sim = 1.0).
//! C3 — retrieve over a missing or empty episode file returns Ok(vec![]).
//!
//! Acceptance criterion mapping:
//!   C1 -> c1_retrieve_returns_top_k_by_similarity
//!   C2 -> c2_identical_text_episode_is_nearest
//!   C3 -> c3_missing_file_returns_empty_vec, c3_empty_file_returns_empty_vec

use zira_memory::{append_episode, retrieve, Episode, HashEmbedder};

fn ep(role: &str, text: &str, ts: u64) -> Episode {
    Episode { role: role.to_string(), text: text.to_string(), timestamp: ts }
}

// ---- C1 -----------------------------------------------------------------------

/// retrieve(path, embedder, query, k=2) over three written episodes returns
/// exactly 2 Episode values whose texts originate from the written set.
#[test]
fn c1_retrieve_returns_top_k_by_similarity() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("ep.jsonl");
    let embedder = HashEmbedder::new(16);

    let ep1 = ep("user", "alpha text content", 1);
    let ep2 = ep("assistant", "beta text content", 2);
    let ep3 = ep("user", "gamma text content", 3);

    append_episode(&path, &ep1).expect("append ep1");
    append_episode(&path, &ep2).expect("append ep2");
    append_episode(&path, &ep3).expect("append ep3");

    let results: Vec<Episode> = retrieve(&path, &embedder, "alpha text content", 2)
        .expect("retrieve must not return an error");

    assert_eq!(results.len(), 2, "k=2 must return exactly 2 episodes");

    let known_texts = ["alpha text content", "beta text content", "gamma text content"];
    for ep in &results {
        assert!(
            known_texts.contains(&ep.text.as_str()),
            "returned episode text {:?} must be one of the written episodes",
            ep.text,
        );
    }
}

// ---- C2 -----------------------------------------------------------------------

/// An episode whose text is identical to the query has cosine similarity 1.0
/// (identical embedding). retrieve(k=1) must return that episode and only that
/// episode, even when a "far" episode is also present.
#[test]
fn c2_identical_text_episode_is_nearest() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("ep.jsonl");
    let embedder = HashEmbedder::new(32);

    let query = "rust programming language systems";
    let far = "zxqjvf completely unrelated nonsense xyzzy wombat";

    append_episode(&path, &ep("user", query, 1)).expect("append near episode");
    append_episode(&path, &ep("user", far, 2)).expect("append far episode");

    let results = retrieve(&path, &embedder, query, 1)
        .expect("retrieve must not return an error");

    assert_eq!(results.len(), 1, "k=1 must return exactly 1 episode");
    assert_eq!(
        results[0].text, query,
        "the episode with text identical to the query must be ranked nearest"
    );
}

// ---- C3 -----------------------------------------------------------------------

/// retrieve on a path that does not exist must return Ok(vec![]) — not an error.
#[test]
fn c3_missing_file_returns_empty_vec() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("nonexistent_episodes.jsonl");
    let embedder = HashEmbedder::new(8);

    assert!(!path.exists(), "precondition: file must not exist");

    let results = retrieve(&path, &embedder, "any query string", 5)
        .expect("retrieve over a missing file must return Ok, not Err");

    assert!(results.is_empty(), "missing episode file must yield an empty result vec");
}

/// retrieve on an empty file (zero bytes) must return Ok(vec![]).
#[test]
fn c3_empty_file_returns_empty_vec() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("empty_episodes.jsonl");
    std::fs::write(&path, b"").expect("create empty file");
    let embedder = HashEmbedder::new(8);

    let results = retrieve(&path, &embedder, "any query string", 5)
        .expect("retrieve over an empty file must return Ok, not Err");

    assert!(results.is_empty(), "empty episode file must yield an empty result vec");
}
