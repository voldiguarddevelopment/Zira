//! Frozen tests for T-01.11 — "Ask claude end-to-end".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_answer_struct_has_text_and_usage,
//!          c2_ask_success_returns_answer_from_stub
//!   C2 -> c2_ask_success_returns_answer_from_stub
//!   C3 -> c3_ask_non_zero_exit_returns_err

use std::fs;
use std::os::unix::fs::PermissionsExt;

use zira_bridge::{ask, Answer, BridgeError};
use zira_config::ZiraConfig;
use zira_proto::Transcript;

// ---- helpers ----------------------------------------------------------------

/// Write a small shell script to a temp dir, make it executable, and return its path.
fn make_stub(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join("zira_bridge_ask_tests");
    fs::create_dir_all(&dir).expect("create temp dir for stub scripts");
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write stub script");
    let mut perms = fs::metadata(&path).expect("stat stub").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod stub");
    path.to_string_lossy().into_owned()
}

/// Build a `ZiraConfig` with `binary_path` set to the given stub path.
fn make_cfg(binary_path: &str) -> ZiraConfig {
    let mut cfg = ZiraConfig::default();
    cfg.model.binary_path = binary_path.to_string();
    cfg.model.model_id = "stub-model".to_string();
    cfg
}

// ---- C1 — Answer struct shape -----------------------------------------------

/// `Answer` must be a public struct with `text: String` and `usage: Usage` so callers
/// can destructure or pattern-match the result.
#[test]
fn c1_answer_struct_has_text_and_usage() {
    let a = Answer {
        text: "hello".to_string(),
        usage: zira_proto::Usage {
            input_tokens: 1,
            output_tokens: 2,
        },
    };
    assert_eq!(a.text, "hello");
    assert_eq!(a.usage.input_tokens, 1);
    assert_eq!(a.usage.output_tokens, 2);
}

// ---- C1 + C2 — success path: ask returns Answer matching stub output ---------

/// Running `ask` against a stub that emits a valid stream-json `result` event must
/// return `Ok(Answer)` with `text` and `usage` matching the stub's output exactly.
/// This exercises C1 (ask composes prompt, invokes claude, returns Answer) and C2
/// (integration test against stub asserting text and usage values).
#[test]
fn c2_ask_success_returns_answer_from_stub() {
    // The stub emits one stream-json result event and exits 0.
    let stub = make_stub(
        "ask_success.sh",
        r#"printf '%s\n' '{"type":"result","subtype":"success","is_error":false,"result":"Hello from stub!","num_turns":1,"usage":{"input_tokens":7,"output_tokens":3}}'"#,
    );
    let cfg = make_cfg(&stub);
    let transcript = Transcript {
        text: "say hello".to_string(),
    };

    let answer = ask(&cfg, "be helpful", &transcript)
        .expect("ask must succeed when stub exits 0 with a valid result event");

    assert_eq!(
        answer.text, "Hello from stub!",
        "answer.text must match the result field in the stub output; got: {:?}",
        answer.text
    );
    assert_eq!(
        answer.usage.input_tokens, 7,
        "answer.usage.input_tokens must match stub output; got: {}",
        answer.usage.input_tokens
    );
    assert_eq!(
        answer.usage.output_tokens, 3,
        "answer.usage.output_tokens must match stub output; got: {}",
        answer.usage.output_tokens
    );
}

// ---- C3 — non-zero exit returns Err(BridgeError) ----------------------------

/// A stub that exits non-zero must cause `ask` to return `Err(BridgeError::NonZeroExit)`
/// carrying the exit code.  The caller must be able to inspect the code without
/// guessing at the error kind.
#[test]
fn c3_ask_non_zero_exit_returns_err() {
    let stub = make_stub("ask_fail.sh", "exit 1");
    let cfg = make_cfg(&stub);
    let transcript = Transcript {
        text: "trigger failure".to_string(),
    };

    let result = ask(&cfg, "be helpful", &transcript);

    assert!(
        result.is_err(),
        "ask must return Err when the subprocess exits non-zero"
    );
    match result.unwrap_err() {
        BridgeError::NonZeroExit(code) => assert_eq!(
            code, 1,
            "NonZeroExit must carry the actual exit code; got: {code}"
        ),
        e => panic!("expected BridgeError::NonZeroExit(1), got {e:?}"),
    }
}
