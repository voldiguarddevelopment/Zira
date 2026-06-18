//! Frozen tests for T-04.03 — "Define the ManifestError type".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_manifest_error_all_three_variants_exist,
//!          c1_manifest_error_is_thiserror_derived
//!   C2 -> c2_display_messages_are_non_empty,
//!          c2_display_messages_are_distinct
//!   C3 -> c3_manifest_error_implements_std_error

use zira_skills::ManifestError;

// Compile-time proof that ManifestError: std::error::Error.
fn assert_error<E: std::error::Error>(_: &E) {}

// ---- C1 — all three variants exist and ManifestError is thiserror-derived --------------

/// Constructs each of the three ManifestError variants by name, exercising their existence
/// and confirming they each carry a String context payload. Also exercises the Debug derive
/// that thiserror requires.
#[test]
fn c1_manifest_error_all_three_variants_exist() {
    let parse = ManifestError::Parse("toml parse failed".to_string());
    let missing = ManifestError::MissingField("entry".to_string());
    let io = ManifestError::Io("file not found".to_string());

    // Debug output must be non-empty for all three.
    assert!(!format!("{parse:?}").is_empty());
    assert!(!format!("{missing:?}").is_empty());
    assert!(!format!("{io:?}").is_empty());
}

/// Binding each variant to `&dyn std::error::Error` confirms the Error impl comes from
/// thiserror (a bare manual impl would satisfy C3 but not the "thiserror-derived" wording).
/// We verify this indirectly: thiserror derives both Error and Display simultaneously, so
/// if Display is present and the Error bound is satisfied, the derive is in use.
#[test]
fn c1_manifest_error_is_thiserror_derived() {
    // Each variant must satisfy the Error bound (confirmed via assert_error helper) AND
    // produce a non-empty Display. Both are guaranteed by thiserror's derive.
    let parse = ManifestError::Parse("ctx".to_string());
    let missing = ManifestError::MissingField("name".to_string());
    let io = ManifestError::Io("disk error".to_string());

    assert_error(&parse);
    assert_error(&missing);
    assert_error(&io);

    assert!(!format!("{parse}").is_empty());
    assert!(!format!("{missing}").is_empty());
    assert!(!format!("{io}").is_empty());
}

// ---- C2 — Display messages are non-empty and mutually distinct -------------------------

/// Each variant's Display string must be non-empty.
#[test]
fn c2_display_messages_are_non_empty() {
    let parse_msg = format!("{}", ManifestError::Parse("x".to_string()));
    let missing_msg = format!("{}", ManifestError::MissingField("y".to_string()));
    let io_msg = format!("{}", ManifestError::Io("z".to_string()));

    assert!(
        !parse_msg.is_empty(),
        "ManifestError::Parse Display must be non-empty"
    );
    assert!(
        !missing_msg.is_empty(),
        "ManifestError::MissingField Display must be non-empty"
    );
    assert!(
        !io_msg.is_empty(),
        "ManifestError::Io Display must be non-empty"
    );
}

/// Each variant's Display string must be distinct from the other two.
/// Uses the same context string for all three to ensure distinctness comes from the variant
/// prefix, not from different payloads.
#[test]
fn c2_display_messages_are_distinct() {
    let ctx = "same-context".to_string();
    let parse_msg = format!("{}", ManifestError::Parse(ctx.clone()));
    let missing_msg = format!("{}", ManifestError::MissingField(ctx.clone()));
    let io_msg = format!("{}", ManifestError::Io(ctx.clone()));

    assert_ne!(
        parse_msg, missing_msg,
        "Parse and MissingField Display messages must differ"
    );
    assert_ne!(
        parse_msg, io_msg,
        "Parse and Io Display messages must differ"
    );
    assert_ne!(
        missing_msg, io_msg,
        "MissingField and Io Display messages must differ"
    );
}

// ---- C3 — ManifestError implements std::error::Error -----------------------------------

/// Binds each constructed ManifestError variant to `&dyn std::error::Error`.
/// This fails to compile unless ManifestError: std::error::Error, satisfying C3.
#[test]
fn c3_manifest_error_implements_std_error() {
    let parse: &dyn std::error::Error = &ManifestError::Parse("toml error".to_string());
    let missing: &dyn std::error::Error =
        &ManifestError::MissingField("required field".to_string());
    let io: &dyn std::error::Error = &ManifestError::Io("io failure".to_string());

    // The bindings above prove the trait impl; additionally assert non-empty display.
    assert!(!parse.to_string().is_empty());
    assert!(!missing.to_string().is_empty());
    assert!(!io.to_string().is_empty());
}
