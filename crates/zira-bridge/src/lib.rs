//! zira-bridge — Claude Code stream-json driver.

use zira_config::ZiraConfig;

/// Build the argv for launching the `claude` CLI non-interactively with stream-json output.
///
/// The first element is the path to the `claude` binary (`cfg.model.binary_path`).
/// The returned vector is deterministic for a given `cfg`.
pub fn build_argv(_cfg: &ZiraConfig) -> Vec<String> {
    todo!("T-01.05 not yet implemented: build_argv")
}
