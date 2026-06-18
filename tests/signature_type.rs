//! Frozen tests for T-04.04 — "Define the Signature type".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_signature_struct_exists_with_to_hex,
//!          c1_from_hex_returns_manifest_error_on_failure
//!   C2 -> c2_signature_hex_round_trip
//!   C3 -> c3_from_hex_non_hex_string_returns_err

use zira_skills::{ManifestError, Signature};

// Compile-time proof that ManifestError is the error type in from_hex.
fn _assert_from_hex_result_type(r: Result<Signature, ManifestError>) -> bool {
    r.is_ok()
}

// ---- C1 — Signature struct exists with to_hex and from_hex --------------------------

/// Constructs a Signature from raw bytes and calls to_hex, confirming the struct and
/// its `to_hex(&self) -> String` accessor exist and return a non-empty string.
/// Also calls from_hex with that hex to confirm the `from_hex` constructor is present
/// and returns `Result<Signature, ManifestError>`.
#[test]
fn c1_signature_struct_exists_with_to_hex() {
    let bytes: Vec<u8> = vec![0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x7f];
    let sig = Signature::new(bytes);

    // to_hex must return a non-empty String.
    let hex: String = sig.to_hex();
    assert!(!hex.is_empty(), "to_hex must return a non-empty string");

    // from_hex must return the correct Result type (binds to the concrete type).
    let result: Result<Signature, ManifestError> = Signature::from_hex(&hex);
    assert!(result.is_ok(), "from_hex on valid hex must succeed");
}

/// Confirms that from_hex's error variant is ManifestError by binding the Err branch
/// explicitly and exercising the Display impl of the error.
#[test]
fn c1_from_hex_returns_manifest_error_on_failure() {
    let result: Result<Signature, ManifestError> = Signature::from_hex("zz-not-hex");
    match result {
        Err(e) => {
            // The error must implement Display (ManifestError does via thiserror).
            let msg = format!("{e}");
            assert!(!msg.is_empty(), "ManifestError Display must be non-empty");
        }
        Ok(_) => panic!("from_hex on invalid hex must return Err"),
    }
}

// ---- C2 — round-trip: to_hex then from_hex yields an equal Signature ----------------

/// Creates a Signature from known bytes, serialises it to hex with to_hex, then
/// deserialises back with from_hex and asserts the recovered value equals the original.
/// Exercises the PartialEq derive and the full round-trip invariant.
#[test]
fn c2_signature_hex_round_trip() {
    let original_bytes: Vec<u8> = vec![
        0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
        0xcd, 0xef,
    ];
    let original = Signature::new(original_bytes);

    let hex = original.to_hex();
    let recovered =
        Signature::from_hex(&hex).expect("from_hex on a to_hex output must succeed");

    assert_eq!(
        original, recovered,
        "from_hex(to_hex(sig)) must equal sig"
    );
}

// ---- C3 — from_hex on a non-hex string returns Err, never panics --------------------

/// Passes several classes of non-hex strings to from_hex and asserts each returns Err.
/// The test itself must not panic — verifying the error path is recoverable, not fatal.
#[test]
fn c3_from_hex_non_hex_string_returns_err() {
    let bad_inputs: &[&str] = &[
        "not-hex-at-all",
        "GGGGGGGG",      // 'G' is outside [0-9a-fA-F]
        "0xdeadbeef",    // '0x' prefix is not bare hex
        "hello, world!", // punctuation
        "  \t\n",        // whitespace only
    ];
    for input in bad_inputs {
        let result = Signature::from_hex(input);
        assert!(
            result.is_err(),
            "from_hex({input:?}) must return Err, got Ok"
        );
    }
}
