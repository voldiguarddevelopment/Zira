//! Frozen tests for T-01.07 — "Capture the claude output".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_raw_output_struct_has_stdout_and_status,
//!          c1_invoke_writes_prompt_to_stdin
//!   C2 -> c2_invoke_against_stub_echoes_fixed_string

use std::fs;
use std::os::unix::fs::PermissionsExt;

use zira_bridge::{invoke, RawOutput};

/// Write a small shell script to a temp dir, make it executable, and return its path.
fn make_stub(name: &str, body: &str) -> String {
    let dir = std::env::temp_dir().join("zira_bridge_invoke_tests");
    fs::create_dir_all(&dir).expect("create temp dir for stub scripts");
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write stub script");
    let mut perms = fs::metadata(&path).expect("stat stub").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod stub");
    path.to_string_lossy().into_owned()
}

// ---- C1 — invoke(argv, prompt) -> std::io::Result<RawOutput> --------------------------------

/// `RawOutput` must be a public struct with `stdout: String` and `status: i32` so callers
/// can pattern-match or destructure the result.
#[test]
fn c1_raw_output_struct_has_stdout_and_status() {
    let out = RawOutput {
        stdout: "captured".to_string(),
        status: 42,
    };
    assert_eq!(out.stdout, "captured");
    assert_eq!(out.status, 42);
}

/// `invoke` must write the prompt to the spawned process's stdin before capturing output.
/// A stub that reads all stdin (`cat`) and echoes it back proves the write happens.
/// If `invoke` never writes to stdin the output will be empty and the assertion fails.
#[test]
fn c1_invoke_writes_prompt_to_stdin() {
    let stub = make_stub("echo_stdin.sh", "cat");
    let argv = vec![stub];
    let prompt = "unique-prompt-for-stdin-check";
    let result = invoke(&argv, prompt).expect("invoke must succeed for a well-behaved stub");
    assert!(
        result.stdout.contains(prompt),
        "stdout must contain the prompt written to stdin; got: {:?}",
        result.stdout
    );
    assert_eq!(
        result.status, 0,
        "echo-stdin stub must exit 0; got: {}",
        result.status
    );
}

// ---- C2 — repo-root integration test against a fixed-echo stub ----------------------------

/// Runs `invoke` against a stub that unconditionally echoes a fixed string to stdout and
/// exits 0.  Asserts that the returned `stdout` equals that string and `status` is 0.
#[test]
fn c2_invoke_against_stub_echoes_fixed_string() {
    let stub = make_stub("fixed_echo.sh", "echo 'hello from stub'");
    let argv = vec![stub];
    let result = invoke(&argv, "any prompt").expect("invoke must succeed for a well-behaved stub");
    assert_eq!(
        result.stdout.trim(),
        "hello from stub",
        "stdout must equal the fixed string echoed by the stub; got: {:?}",
        result.stdout
    );
    assert_eq!(
        result.status, 0,
        "fixed-echo stub must exit 0; got status {}",
        result.status
    );
}
