use zira_skills::{sign_manifest, verify_manifest, Signature, SkillManifest};

fn sample_manifest() -> SkillManifest {
    SkillManifest {
        name: "verify-skill".to_string(),
        version: "1.0.0".to_string(),
        entry: "skill.wasm".to_string(),
        capabilities: vec!["net:fetch".to_string()],
        allowed_roots: vec!["/opt/skills".to_string()],
    }
}

// C1: verify_manifest returns true for a signature produced by sign_manifest
// with the same key and manifest (the ACCEPT path).
#[test]
fn test_verify_manifest_accept() {
    let key = b"accept-path-key";
    let m = sample_manifest();
    let sig = sign_manifest(key, &m);
    assert!(
        verify_manifest(key, &m, &sig),
        "verify_manifest must return true for a valid sign_manifest signature"
    );
}

// C2: verify_manifest returns false when the signature bytes are tampered (a flipped byte).
#[test]
fn test_verify_manifest_reject_tampered_sig() {
    let key = b"tamper-check-key";
    let m = sample_manifest();
    let sig = sign_manifest(key, &m);
    // Flip one nibble in the hex representation to produce a one-byte-altered signature.
    let hex = sig.to_hex();
    let last_char = hex.chars().last().unwrap();
    let replacement = if last_char != 'f' { 'f' } else { '0' };
    let tampered_hex = format!("{}{}", &hex[..hex.len() - 1], replacement);
    let tampered = Signature::from_hex(&tampered_hex).expect("tampered hex is valid");
    assert!(
        !verify_manifest(key, &m, &tampered),
        "verify_manifest must return false for a tampered (one-nibble-flipped) signature"
    );
}

// C3a: verify_manifest returns false when the manifest is altered after signing.
#[test]
fn test_verify_manifest_reject_altered_manifest() {
    let key = b"altered-manifest-key";
    let original = sample_manifest();
    let sig = sign_manifest(key, &original);
    let mut altered = original.clone();
    altered.name = "tampered-name".to_string();
    assert!(
        !verify_manifest(key, &altered, &sig),
        "verify_manifest must return false when the manifest differs from the signed one"
    );
}

// C3b: verify_manifest returns false when a different key is used for verification.
#[test]
fn test_verify_manifest_reject_wrong_key() {
    let signing_key = b"the-real-signing-key";
    let m = sample_manifest();
    let sig = sign_manifest(signing_key, &m);
    let wrong_key = b"a-completely-different-key";
    assert!(
        !verify_manifest(wrong_key, &m, &sig),
        "verify_manifest must return false when a different key is supplied"
    );
}
