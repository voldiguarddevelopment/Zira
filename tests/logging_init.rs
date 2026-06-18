// Repo-root integration test for T-00.04: Initialize structured logging.
//
// RED state: `zira_core::logging` does not exist yet — this file fails to
// compile.  That compile failure IS the RED state.  No `todo!` or
// `unimplemented!` tokens appear below; failure is purely structural
// (missing module / function, not a runtime stub).
//
// GREEN state: all tests pass once `zira_core::logging::{build_filter,
// init_logging}` are implemented in `zira-core`.
//
// Criterion → test mapping:
//
//   C1 -> test_build_filter_honors_rust_log_error,
//          test_build_filter_defaults_to_info,
//          test_malformed_rust_log_falls_back_to_info
//   C2 -> test_first_call_is_ok,
//          test_init_returns_typed_result,
//          test_init_is_idempotent
//   C3 -> test_default_level_enables_info,
//          test_default_level_excludes_debug,
//          test_default_level_is_not_silent

use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::prelude::*;

// ── Shared install guard ─────────────────────────────────────────────────────

// The global tracing subscriber can be installed exactly once per process.
// OnceLock captures the bool from the first install so test_first_call_is_ok
// can assert it, while all other tests call install_once() to ensure the
// subscriber is ready before they probe it.
static INIT_RESULT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

// Serialises direct RUST_LOG mutation across parallel tests.  install_once()
// does NOT hold this lock so that tests which hold ENV_MUTEX can safely call
// install_once() without deadlocking.
static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Install the global subscriber exactly once, with RUST_LOG absent so the
/// default-info behaviour is exercised.  Returns the cached bool.
fn install_once() -> bool {
    *INIT_RESULT.get_or_init(|| {
        // OnceLock ensures this closure runs exactly once across all threads.
        std::env::remove_var("RUST_LOG");
        zira_core::logging::init_logging().is_ok()
    })
}

// ── C2: first call returns Ok ─────────────────────────────────────────────────

/// C2: the first call to `init_logging` (with RUST_LOG absent) must return Ok.
#[test]
fn test_first_call_is_ok() {
    assert!(
        install_once(),
        "first call to init_logging must return Ok(())"
    );
}

// ── C2: typed result ──────────────────────────────────────────────────────────

/// C2: `init_logging` returns `Result<(), SetGlobalDefaultError>`.
/// The explicit type annotation is a compile-time assertion: any change to the
/// return type breaks this test at compile time.
#[test]
fn test_init_returns_typed_result() {
    install_once();
    // After install_once the subscriber is set; repeat calls return Err.
    let result: Result<(), SetGlobalDefaultError> = zira_core::logging::init_logging();
    assert!(
        result.is_err(),
        "a repeat call to init_logging must return Err (subscriber already set)"
    );
}

// ── C2: idempotency ───────────────────────────────────────────────────────────

/// C2: calling `init_logging` more than once never panics.
#[test]
fn test_init_is_idempotent() {
    install_once();
    let _a: Result<(), SetGlobalDefaultError> = zira_core::logging::init_logging();
    let _b: Result<(), SetGlobalDefaultError> = zira_core::logging::init_logging();
    // Reaching here without a panic proves idempotency.
}

// ── C3: default level is info ─────────────────────────────────────────────────

/// C3: with RUST_LOG absent, INFO events must be enabled by the installed
/// subscriber.
#[test]
fn test_default_level_enables_info() {
    install_once();
    assert!(
        tracing::enabled!(tracing::Level::INFO),
        "INFO must be enabled at the default level (expected: info)"
    );
}

/// C3: with RUST_LOG absent, DEBUG events must NOT be enabled (default is info,
/// not trace).
#[test]
fn test_default_level_excludes_debug() {
    install_once();
    assert!(
        !tracing::enabled!(tracing::Level::DEBUG),
        "DEBUG must NOT be enabled at the default info level"
    );
}

/// C3: the default level is not silent — ERROR events must be enabled.
#[test]
fn test_default_level_is_not_silent() {
    install_once();
    assert!(
        tracing::enabled!(tracing::Level::ERROR),
        "ERROR must be enabled; the default subscriber must not be silent"
    );
}

// ── C1: RUST_LOG is honoured ──────────────────────────────────────────────────

/// C1: `build_filter()` reads `RUST_LOG`.  With `RUST_LOG=error`, only ERROR+
/// events pass — INFO must not be enabled.
///
/// Uses `with_default` to install the filter thread-locally so we can probe it
/// without conflicting with the global subscriber.
#[test]
fn test_build_filter_honors_rust_log_error() {
    install_once();

    let filter = {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("RUST_LOG", "error");
        let f = zira_core::logging::build_filter();
        std::env::remove_var("RUST_LOG");
        f
    };

    let sub = tracing_subscriber::registry().with(filter);
    let info_enabled = tracing::subscriber::with_default(sub, || {
        tracing::enabled!(tracing::Level::INFO)
    });

    assert!(
        !info_enabled,
        "with RUST_LOG=error, INFO must not be enabled (EnvFilter must read RUST_LOG)"
    );
}

/// C1 + C3: `build_filter()` with `RUST_LOG` absent defaults to info — INFO is
/// enabled and DEBUG is not.
#[test]
fn test_build_filter_defaults_to_info() {
    install_once();

    let filter = {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("RUST_LOG");
        zira_core::logging::build_filter()
    };

    let sub = tracing_subscriber::registry().with(filter);
    let (info_enabled, debug_enabled) = tracing::subscriber::with_default(sub, || {
        (
            tracing::enabled!(tracing::Level::INFO),
            tracing::enabled!(tracing::Level::DEBUG),
        )
    });

    assert!(
        info_enabled,
        "with no RUST_LOG, INFO must be enabled (default: info)"
    );
    assert!(
        !debug_enabled,
        "with no RUST_LOG, DEBUG must not be enabled (default: info, not trace)"
    );
}

/// C1: a malformed RUST_LOG directive falls back to info rather than going
/// silent or panicking.
#[test]
fn test_malformed_rust_log_falls_back_to_info() {
    install_once();

    let filter = {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("RUST_LOG", "!!!not@a#valid$directive");
        let f = zira_core::logging::build_filter();
        std::env::remove_var("RUST_LOG");
        f
    };

    let sub = tracing_subscriber::registry().with(filter);
    let info_enabled = tracing::subscriber::with_default(sub, || {
        tracing::enabled!(tracing::Level::INFO)
    });

    assert!(
        info_enabled,
        "with a malformed RUST_LOG, the filter must fall back to info (not silent)"
    );
}
