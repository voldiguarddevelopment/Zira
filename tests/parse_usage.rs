//! Frozen tests for T-01.09 — "Parse the usage totals".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_result_event_usage_returned,
//!          c1_result_event_usage_in_multiline_stream
//!   C2 -> c2_empty_stdout_returns_zero_usage,
//!          c2_no_result_type_line_returns_zero_usage,
//!          c2_result_event_without_usage_field_returns_zero

use zira_bridge::{parse_usage, RawOutput};
use zira_proto::Usage;

// ---- helpers ----------------------------------------------------------------

fn raw(stdout: &str) -> RawOutput {
    RawOutput {
        stdout: stdout.to_string(),
        status: 0,
    }
}

fn assert_usage(u: &Usage, input: u64, output: u64) {
    assert_eq!(
        u.input_tokens, input,
        "input_tokens: expected {input}, got {}",
        u.input_tokens
    );
    assert_eq!(
        u.output_tokens, output,
        "output_tokens: expected {output}, got {}",
        u.output_tokens
    );
}

// ---- C1 — parse_usage extracts tokens from the terminal result event --------

/// A single result event carrying both usage fields must return the exact counts.
#[test]
fn c1_result_event_usage_returned() {
    let stdout = r#"{"type":"result","subtype":"success","is_error":false,"result":"Hello!","num_turns":1,"usage":{"input_tokens":42,"output_tokens":17}}"#;
    let u = parse_usage(&raw(stdout));
    assert_usage(&u, 42, 17);
}

/// In a multiline stream, usage is read from the terminal result event and not
/// from intermediate assistant-type lines which carry their own (partial) usage.
#[test]
fn c1_result_event_usage_in_multiline_stream() {
    let stdout = concat!(
        r#"{"type":"system","subtype":"init","session_id":"s1","tools":[],"mcp_servers":[],"model":"claude-sonnet-4-6"}"#, "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hi!"}],"usage":{"input_tokens":5,"output_tokens":1}},"session_id":"s1"}"#, "\n",
        r#"{"type":"result","subtype":"success","is_error":false,"result":"Hi!","num_turns":1,"usage":{"input_tokens":100,"output_tokens":50},"session_id":"s1"}"#, "\n",
    );
    let u = parse_usage(&raw(stdout));
    assert_usage(&u, 100, 50);
}

// ---- C2 — missing usage fields yield zeroes ---------------------------------

/// An entirely empty stdout must return Usage { input_tokens: 0, output_tokens: 0 }
/// without panicking.
#[test]
fn c2_empty_stdout_returns_zero_usage() {
    let u = parse_usage(&raw(""));
    assert_usage(&u, 0, 0);
}

/// stdout containing only non-result-type events (system, assistant) with no
/// result event must return zero usage.
#[test]
fn c2_no_result_type_line_returns_zero_usage() {
    let stdout = concat!(
        r#"{"type":"system","subtype":"init","session_id":"s2","tools":[],"mcp_servers":[],"model":"claude-sonnet-4-6"}"#, "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Partial..."}]}}"#, "\n",
    );
    let u = parse_usage(&raw(stdout));
    assert_usage(&u, 0, 0);
}

/// A result event that exists but carries no usage object must yield zero usage,
/// not a panic or error.
#[test]
fn c2_result_event_without_usage_field_returns_zero() {
    let stdout = r#"{"type":"result","subtype":"success","is_error":false,"result":"Done","num_turns":1}"#;
    let u = parse_usage(&raw(stdout));
    assert_usage(&u, 0, 0);
}
