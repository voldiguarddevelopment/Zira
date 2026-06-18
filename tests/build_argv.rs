//! Frozen tests for T-01.05 — "Build the claude invocation".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_argv_starts_with_binary_path,
//!          c1_argv_contains_non_interactive_flag,
//!          c1_argv_contains_stream_json_output_format
//!   C2 -> c2_model_id_follows_model_flag

use zira_bridge::build_argv;
use zira_config::{ModelConfig, ZiraConfig};

/// Returns a minimal but fully-populated config with distinct values for each
/// field that `build_argv` must reflect.
fn make_cfg(binary_path: &str, model_id: &str) -> ZiraConfig {
    ZiraConfig {
        model: ModelConfig {
            binary_path: binary_path.to_string(),
            model_id: model_id.to_string(),
        },
        ..ZiraConfig::default()
    }
}

// ---- C1 — argv launches the claude CLI non-interactively with stream-json output -----------

#[test]
fn c1_argv_starts_with_binary_path() {
    let cfg = make_cfg("/usr/local/bin/claude", "claude-sonnet-4-6");
    let argv = build_argv(&cfg);
    assert!(
        !argv.is_empty(),
        "build_argv must return a non-empty argv vector"
    );
    assert_eq!(
        argv[0], "/usr/local/bin/claude",
        "argv[0] must be the binary path from cfg.model.binary_path"
    );
}

#[test]
fn c1_argv_contains_non_interactive_flag() {
    // The `--print` / `-p` flag makes claude non-interactive (reads from stdin / arg,
    // writes to stdout, then exits — no interactive REPL).
    let cfg = make_cfg("/usr/local/bin/claude", "claude-sonnet-4-6");
    let argv = build_argv(&cfg);
    let has_print = argv.iter().any(|a| a == "--print" || a == "-p");
    assert!(
        has_print,
        "argv must contain a non-interactive flag (--print or -p); got: {argv:?}"
    );
}

#[test]
fn c1_argv_contains_stream_json_output_format() {
    // `--output-format stream-json` selects the machine-readable streaming JSON format.
    let cfg = make_cfg("/usr/local/bin/claude", "claude-sonnet-4-6");
    let argv = build_argv(&cfg);
    let fmt_pos = argv.iter().position(|a| a == "--output-format");
    assert!(
        fmt_pos.is_some(),
        "argv must contain --output-format; got: {argv:?}"
    );
    let fmt_pos = fmt_pos.unwrap();
    assert!(
        fmt_pos + 1 < argv.len(),
        "--output-format must be followed by a value; got: {argv:?}"
    );
    assert_eq!(
        argv[fmt_pos + 1], "stream-json",
        "the value after --output-format must be 'stream-json'; got: {argv:?}"
    );
}

// ---- C2 — model string from config is the value immediately after the model flag -----------

#[test]
fn c2_model_id_follows_model_flag() {
    let model_id = "claude-opus-4-8";
    let cfg = make_cfg("/usr/local/bin/claude", model_id);
    let argv = build_argv(&cfg);
    let flag_pos = argv.iter().position(|a| a == "--model" || a == "-m");
    assert!(
        flag_pos.is_some(),
        "argv must contain a model flag (--model or -m); got: {argv:?}"
    );
    let flag_pos = flag_pos.unwrap();
    assert!(
        flag_pos + 1 < argv.len(),
        "the model flag must be followed by a value; got: {argv:?}"
    );
    assert_eq!(
        argv[flag_pos + 1], model_id,
        "the element immediately after the model flag must equal cfg.model.model_id"
    );
}
