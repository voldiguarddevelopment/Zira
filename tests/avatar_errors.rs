//! Frozen tests for T-03.11 — "Type the avatar errors".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_avatar_error_missing_vrm_path_variant_exists,
//!          c1_avatar_error_model_unreadable_variant_exists,
//!          c1_avatar_error_unsupported_viseme_variant_exists,
//!          c1_avatar_error_implements_error_trait,
//!          c1_avatar_error_implements_display
//!   C2 -> c2_display_missing_vrm_path_nonempty_names_failure,
//!          c2_display_model_unreadable_nonempty_names_failure,
//!          c2_display_unsupported_viseme_nonempty_names_failure,
//!          c2_display_all_variants_produce_distinct_messages

use zira_avatar::AvatarError;

// Compile-time proof that AvatarError: std::error::Error + std::fmt::Display.
fn assert_error_trait<E: std::error::Error + std::fmt::Display>(_: &E) {}

// ---- C1 — AvatarError enum with three distinct variants ----------------------------------

/// The MissingVrmPath variant must be constructible without arguments (unit variant).
#[test]
fn c1_avatar_error_missing_vrm_path_variant_exists() {
    let e = AvatarError::MissingVrmPath;
    let _ = e;
}

/// The ModelUnreadable variant must be constructible with a path/description string.
#[test]
fn c1_avatar_error_model_unreadable_variant_exists() {
    let e = AvatarError::ModelUnreadable("/path/to/avatar.vrm".to_string());
    let _ = e;
}

/// The UnsupportedViseme variant must be constructible with an unsupported label string.
#[test]
fn c1_avatar_error_unsupported_viseme_variant_exists() {
    let e = AvatarError::UnsupportedViseme("blink".to_string());
    let _ = e;
}

/// AvatarError must implement std::error::Error (verified at compile time via the
/// assert_error_trait bound).
#[test]
fn c1_avatar_error_implements_error_trait() {
    assert_error_trait(&AvatarError::MissingVrmPath);
    assert_error_trait(&AvatarError::ModelUnreadable(String::new()));
    assert_error_trait(&AvatarError::UnsupportedViseme(String::new()));
}

/// AvatarError must implement Display (confirmed by calling to_string() at runtime).
#[test]
fn c1_avatar_error_implements_display() {
    let _ = AvatarError::MissingVrmPath.to_string();
    let _ = AvatarError::ModelUnreadable(String::new()).to_string();
    let _ = AvatarError::UnsupportedViseme(String::new()).to_string();
}

// ---- C2 — Display text is non-empty and names the failure --------------------------------

/// MissingVrmPath's Display must be non-empty and contain a keyword that identifies the
/// failure as a missing/absent VRM path configuration, so a reader knows what went wrong.
#[test]
fn c2_display_missing_vrm_path_nonempty_names_failure() {
    let msg = AvatarError::MissingVrmPath.to_string();
    assert!(!msg.is_empty(), "MissingVrmPath Display must not be empty");
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("vrm") || lower.contains("path") || lower.contains("missing"),
        "MissingVrmPath Display must name the missing-VRM-path failure, got: {msg:?}"
    );
}

/// ModelUnreadable's Display must be non-empty and identify the failure as an unreadable
/// or absent model file. A neutral context string is injected so any keyword match comes
/// from the format string, not the payload.
#[test]
fn c2_display_model_unreadable_nonempty_names_failure() {
    let msg = AvatarError::ModelUnreadable("ctx-alpha".to_string()).to_string();
    assert!(!msg.is_empty(), "ModelUnreadable Display must not be empty");
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("model") || lower.contains("file") || lower.contains("read"),
        "ModelUnreadable Display must name the model-file failure, got: {msg:?}"
    );
}

/// UnsupportedViseme's Display must be non-empty and contain a keyword identifying the
/// failure as an unrecognized or unsupported viseme label. A neutral context string is
/// injected so any keyword match comes from the format string.
#[test]
fn c2_display_unsupported_viseme_nonempty_names_failure() {
    let msg = AvatarError::UnsupportedViseme("ctx-beta".to_string()).to_string();
    assert!(!msg.is_empty(), "UnsupportedViseme Display must not be empty");
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("viseme") || lower.contains("label") || lower.contains("unsupported"),
        "UnsupportedViseme Display must name the unsupported-viseme failure, got: {msg:?}"
    );
}

/// All three Display messages must be mutually distinct — a mutation that returns one
/// constant string for every variant would otherwise satisfy the per-variant non-empty
/// checks without distinguishing failures.
#[test]
fn c2_display_all_variants_produce_distinct_messages() {
    let vrm_msg = AvatarError::MissingVrmPath.to_string();
    let model_msg = AvatarError::ModelUnreadable("x".to_string()).to_string();
    let viseme_msg = AvatarError::UnsupportedViseme("y".to_string()).to_string();
    assert_ne!(
        vrm_msg, model_msg,
        "MissingVrmPath and ModelUnreadable must have distinct Display messages"
    );
    assert_ne!(
        vrm_msg, viseme_msg,
        "MissingVrmPath and UnsupportedViseme must have distinct Display messages"
    );
    assert_ne!(
        model_msg, viseme_msg,
        "ModelUnreadable and UnsupportedViseme must have distinct Display messages"
    );
}
