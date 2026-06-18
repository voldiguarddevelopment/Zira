use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::EnvFilter;

/// Build an [`EnvFilter`] honoring `RUST_LOG`, falling back to `"info"` when
/// the variable is absent, empty, or contains an unparsable directive.
pub fn build_filter() -> EnvFilter {
    match std::env::var("RUST_LOG") {
        Ok(val) if !val.is_empty() => {
            EnvFilter::try_new(&val).unwrap_or_else(|_| EnvFilter::new("info"))
        }
        _ => EnvFilter::new("info"),
    }
}

/// Install the global tracing subscriber with an [`EnvFilter`] honoring `RUST_LOG`.
///
/// Returns `Ok(())` on the first call. Subsequent calls return
/// `Err(SetGlobalDefaultError)` without panicking.
pub fn init_logging() -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(build_filter())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}
