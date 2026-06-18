//! zira-bridge — Claude Code stream-json driver.

use std::io::Write;
use std::process::{Command, Stdio};

use zira_config::ZiraConfig;
use zira_proto::Transcript;

/// Raw output captured from a subprocess invocation.
pub struct RawOutput {
    pub stdout: String,
    pub status: i32,
}

/// Spawn the program named by `argv`, write `prompt` to its stdin, and return
/// the captured stdout and exit code.
///
/// Stdin is fully written and closed before stdout is collected, preventing
/// deadlocks when the child buffers until it sees EOF.
pub fn invoke(argv: &[String], prompt: &str) -> std::io::Result<RawOutput> {
    let (program, args) = argv.split_first().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "argv must not be empty")
    })?;

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write prompt then close stdin so the child sees EOF.
    {
        let mut stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "child stdin unavailable")
        })?;
        stdin.write_all(prompt.as_bytes())?;
        // stdin is dropped here, closing the pipe
    }

    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let status = output.status.code().unwrap_or(-1);

    Ok(RawOutput { stdout, status })
}

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
