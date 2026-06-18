use zira_skills::{sign_manifest, SkillManifest};

fn sample_manifest() -> SkillManifest {
    SkillManifest {
        name: "demo-skill".to_string(),
        version: "0.1.0".to_string(),
        entry: "skill.wasm".to_string(),
        capabilities: vec!["fs:read".to_string()],
        allowed_roots: vec!["/tmp".to_string()],
    }
}

// C1 + C2: calling sign_manifest with the hmac+sha2 path produces a 32-byte (256-bit)
// tag and is deterministic — same key + same manifest → identical Signature twice.
#[test]
fn test_sign_manifest_is_deterministic() {
    let key = b"test-key-for-determinism";
    let m = sample_manifest();
    let sig1 = sign_manifest(key, &m);
    let sig2 = sign_manifest(key, &m);
    assert_eq!(
        sig1, sig2,
        "sign_manifest must return the same Signature for identical key and manifest"
    );
}

// C1: the HMAC-SHA256 output is exactly 32 bytes (256 bits).
#[test]
fn test_sign_manifest_output_is_hmac_sha256_length() {
    let key = b"length-check-key";
    let m = sample_manifest();
    let sig = sign_manifest(key, &m);
    assert_eq!(
        sig.to_hex().len(),
        64,
        "HMAC-SHA256 hex must be 64 characters (32 bytes)"
    );
}

// C3: two different keys produce different signatures for the same manifest.
#[test]
fn test_sign_manifest_key_sensitivity() {
    let m = sample_manifest();
    let sig_a = sign_manifest(b"key-alpha", &m);
    let sig_b = sign_manifest(b"key-beta", &m);
    assert_ne!(
        sig_a, sig_b,
        "distinct keys must produce distinct signatures"
    );
}

// C3: mutating a manifest field changes the signature.
#[test]
fn test_sign_manifest_content_sensitivity() {
    let key = b"shared-content-key";
    let original = sample_manifest();
    let mut mutated = original.clone();
    mutated.name = "different-skill".to_string();
    let sig_original = sign_manifest(key, &original);
    let sig_mutated = sign_manifest(key, &mutated);
    assert_ne!(
        sig_original, sig_mutated,
        "a mutated manifest field must change the signature"
    );
}
