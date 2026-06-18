//! zira-bridge — Claude Code stream-json driver.

use zira_config::ZiraConfig;
use zira_proto::Transcript;

/// Compose the prompt string that will be sent to the `claude` CLI.
///
/// The constitution text always appears first, followed by the transcript text.
pub fn compose_prompt(constitution: &str, transcript: &Transcript) -> String {
    format!("{}\n{}", constitution, transcript.text)
}

/// Build the argv for launching the `claude` CLI non-interactively with stream-json output.
///
/// The first element is the path to the `claude` binary (`cfg.model.binary_path`).
/// The returned vector is deterministic for a given `cfg`.
pub fn build_argv(cfg: &ZiraConfig) -> Vec<String> {
    vec![
        cfg.model.binary_path.clone(),
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--model".to_string(),
        cfg.model.model_id.clone(),
    ]
}
