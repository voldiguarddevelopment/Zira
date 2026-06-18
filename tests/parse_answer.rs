//! Frozen tests for T-01.08 — "Extract the answer text".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_result_event_text_returned,
//!          c1_result_event_in_multiline_stream,
//!          c1_result_event_with_empty_text_returns_empty
//!   C2 -> c2_empty_stdout_returns_empty,
//!          c2_no_result_type_line_returns_empty,
//!          c2_only_assistant_lines_returns_empty

use zira_bridge::{parse_answer, RawOutput};

// ---- helpers ----------------------------------------------------------------

fn raw(stdout: &str) -> RawOutput {
    RawOutput {
        stdout: stdout.to_string(),
        status: 0,
    }
}

// ---- C1 — parse_answer returns the terminal result event's text -------------

/// A single-line stdout that contains exactly one `result` event must return
/// the string carried in its `result` field.
#[test]
fn c1_result_event_text_returned() {
    let stdout = r#"{"type":"result","subtype":"success","is_error":false,"result":"Hello from Claude!","num_turns":1}"#;
    let out = raw(stdout);
    assert_eq!(
        parse_answer(&out),
        "Hello from Claude!",
        "parse_answer must extract the 'result' field from a result event"
    );
}

/// A realistic multi-line stream-json output (system init + assistant message +
/// result) must return the text in the terminal result event, not some earlier line.
#[test]
fn c1_result_event_in_multiline_stream() {
    let stdout = concat!(
        r#"{"type":"system","subtype":"init","session_id":"s1","tools":[],"mcp_servers":[],"model":"claude-sonnet-4-6"}"#, "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hi there!"}],"usage":{"input_tokens":10,"output_tokens":3}},"session_id":"s1"}"#, "\n",
        r#"{"type":"result","subtype":"success","is_error":false,"result":"Hi there!","num_turns":1,"session_id":"s1"}"#, "\n",
    );
    let out = raw(stdout);
    assert_eq!(
        parse_answer(&out),
        "Hi there!",
        "parse_answer must return the result field from the terminal result event"
    );
}

/// When the result event's `result` field is the empty string, parse_answer must
/// return an empty String (not an error, not a fallback string).
#[test]
fn c1_result_event_with_empty_text_returns_empty() {
    let stdout = r#"{"type":"result","subtype":"success","is_error":false,"result":"","num_turns":1}"#;
    let out = raw(stdout);
    assert_eq!(
        parse_answer(&out),
        "",
        "a result event with an empty result field must yield an empty String"
    );
}

// ---- C2 — missing result event yields an empty String -----------------------

/// stdout that is entirely empty yields an empty String without panicking.
#[test]
fn c2_empty_stdout_returns_empty() {
    let out = raw("");
    assert_eq!(
        parse_answer(&out),
        "",
        "empty stdout must yield an empty String"
    );
}

/// stdout containing only non-result-type events (system, assistant) but no
/// `result` event must yield an empty String.
#[test]
fn c2_no_result_type_line_returns_empty() {
    let stdout = concat!(
        r#"{"type":"system","subtype":"init","session_id":"s2","tools":[],"mcp_servers":[],"model":"claude-sonnet-4-6"}"#, "\n",
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Partial..."}]}}"#, "\n",
    );
    let out = raw(stdout);
    assert_eq!(
        parse_answer(&out),
        "",
        "stdout with no result-type event must yield an empty String"
    );
}

/// stdout consisting solely of assistant-type lines (no `result` line) yields
/// an empty String — the assistant content blocks are not the answer source.
#[test]
fn c2_only_assistant_lines_returns_empty() {
    let stdout = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"I am thinking..."}]}}"#;
    let out = raw(stdout);
    assert_eq!(
        parse_answer(&out),
        "",
        "assistant-type lines alone do not constitute a result event; must return empty"
    );
}
