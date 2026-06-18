//! Frozen tests for T-01.06 — "Compose the request prompt".
//!
//! Criterion coverage:
//!
//!   C1 -> c1_prompt_contains_constitution_then_transcript
//!   C2 -> c2_empty_transcript_prompt_contains_constitution

use zira_bridge::compose_prompt;
use zira_proto::Transcript;

// ---- C1 — prompt contains the full constitution text followed by the transcript text ----------

#[test]
fn c1_prompt_contains_constitution_then_transcript() {
    let constitution = "You are Zira, a voice-driven coding assistant.";
    let transcript = Transcript {
        text: "What is the capital of France?".to_string(),
    };

    let prompt = compose_prompt(constitution, &transcript);

    // The full constitution must be present.
    assert!(
        prompt.contains(constitution),
        "prompt must contain the full constitution text; got: {prompt:?}"
    );

    // The full transcript text must be present.
    assert!(
        prompt.contains(&transcript.text),
        "prompt must contain the transcript text; got: {prompt:?}"
    );

    // Constitution must appear before the transcript.
    let constitution_pos = prompt
        .find(constitution)
        .expect("constitution must appear in prompt");
    let transcript_pos = prompt
        .find(transcript.text.as_str())
        .expect("transcript text must appear in prompt");
    assert!(
        constitution_pos < transcript_pos,
        "constitution must appear before transcript; constitution at {constitution_pos}, transcript at {transcript_pos}"
    );
}

// ---- C2 — empty transcript still yields a prompt containing the complete constitution ----------

#[test]
fn c2_empty_transcript_prompt_contains_constitution() {
    let constitution = "You are Zira, a voice-driven coding assistant.";
    let transcript = Transcript {
        text: String::new(),
    };

    let prompt = compose_prompt(constitution, &transcript);

    assert!(
        prompt.contains(constitution),
        "prompt must contain the full constitution even when transcript is empty; got: {prompt:?}"
    );
}
