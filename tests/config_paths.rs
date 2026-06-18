//! Frozen tests for T-00.11 — "Resolve the data paths".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_config_dir_under_xdg_config_home,
//!          c1_data_dir_under_xdg_data_home,
//!          c1_memory_dir_under_data_dir,
//!          c1_skills_dir_under_data_dir
//!   C2 -> c2_ensure_dirs_creates_missing_directories,
//!          c2_ensure_dirs_is_idempotent
//!   C3 -> c3_xdg_env_temp_dir_resolves_and_creates_four_dirs
//!
//! The four directory helpers read the XDG environment at call time, so these tests
//! mutate `XDG_CONFIG_HOME` / `XDG_DATA_HOME` — process-global state. The libtest
//! harness runs tests in parallel threads within this one binary, so every test that
//! touches the environment first takes `ENV_LOCK`, guaranteeing the set-var → call →
//! assert sequence is never interleaved with another test's.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use zira_config::{config_dir, data_dir, ensure_dirs, memory_dir, skills_dir};

/// Serializes every test that reads or writes the XDG environment variables.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// A fresh, unique base directory for one test, under the Cargo-managed temp dir.
/// Nothing is created here — the helpers under test own creation.
fn unique_base(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("config_paths")
        .join(test_name)
}

/// Point both XDG base-dir variables at `config` / `data` for the duration of a test.
fn set_xdg(config: &Path, data: &Path) {
    std::env::set_var("XDG_CONFIG_HOME", config);
    std::env::set_var("XDG_DATA_HOME", data);
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_config_dir_under_xdg_config_home() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_base("c1_config");
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    let resolved = config_dir().expect("config_dir must resolve when XDG_CONFIG_HOME is set");
    assert!(
        resolved.starts_with(&xdg_config),
        "config_dir {resolved:?} must be rooted under XDG_CONFIG_HOME {xdg_config:?}"
    );
    assert_ne!(
        resolved, xdg_config,
        "config_dir must be a strict descendant of XDG_CONFIG_HOME, not the base itself"
    );
}

#[test]
fn c1_data_dir_under_xdg_data_home() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_base("c1_data");
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    let resolved = data_dir().expect("data_dir must resolve when XDG_DATA_HOME is set");
    assert!(
        resolved.starts_with(&xdg_data),
        "data_dir {resolved:?} must be rooted under XDG_DATA_HOME {xdg_data:?}"
    );
    assert_ne!(
        resolved, xdg_data,
        "data_dir must be a strict descendant of XDG_DATA_HOME, not the base itself"
    );
}

#[test]
fn c1_memory_dir_under_data_dir() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_base("c1_memory");
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    let data = data_dir().expect("data_dir must resolve");
    let memory = memory_dir().expect("memory_dir must resolve when XDG_DATA_HOME is set");
    assert!(
        memory.starts_with(&xdg_data),
        "memory_dir {memory:?} must be rooted under XDG_DATA_HOME {xdg_data:?}"
    );
    assert!(
        memory.starts_with(&data),
        "memory_dir {memory:?} must live under the data dir {data:?}"
    );
    assert_ne!(
        memory, data,
        "memory_dir must be a distinct directory, not the data dir itself"
    );
}

#[test]
fn c1_skills_dir_under_data_dir() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_base("c1_skills");
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    let data = data_dir().expect("data_dir must resolve");
    let memory = memory_dir().expect("memory_dir must resolve");
    let skills = skills_dir().expect("skills_dir must resolve when XDG_DATA_HOME is set");
    assert!(
        skills.starts_with(&xdg_data),
        "skills_dir {skills:?} must be rooted under XDG_DATA_HOME {xdg_data:?}"
    );
    assert!(
        skills.starts_with(&data),
        "skills_dir {skills:?} must live under the data dir {data:?}"
    );
    assert_ne!(
        skills, data,
        "skills_dir must be a distinct directory, not the data dir itself"
    );
    assert_ne!(
        skills, memory,
        "skills_dir and memory_dir must be distinct directories"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_ensure_dirs_creates_missing_directories() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_base("c2_creates");
    // Start clean: remove any residue from a previous run so the dirs are truly missing.
    let _ = std::fs::remove_dir_all(&base);
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    let cfg = config_dir().expect("config_dir must resolve");
    let data = data_dir().expect("data_dir must resolve");
    let memory = memory_dir().expect("memory_dir must resolve");
    let skills = skills_dir().expect("skills_dir must resolve");

    // Precondition: none of the four directories exist yet.
    for dir in [&cfg, &data, &memory, &skills] {
        assert!(
            !dir.exists(),
            "precondition: {dir:?} must not exist before ensure_dirs()"
        );
    }

    ensure_dirs().expect("ensure_dirs must create every missing directory");

    for dir in [&cfg, &data, &memory, &skills] {
        assert!(
            dir.is_dir(),
            "ensure_dirs must have created {dir:?} as a directory"
        );
    }
}

#[test]
fn c2_ensure_dirs_is_idempotent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = unique_base("c2_idempotent");
    let _ = std::fs::remove_dir_all(&base);
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    ensure_dirs().expect("first ensure_dirs call must succeed");
    // A second call against already-existing directories must also succeed (an existing
    // dir is fine, never an error).
    ensure_dirs().expect("second ensure_dirs call must succeed (idempotent)");

    for dir in [
        config_dir().expect("config_dir must resolve"),
        data_dir().expect("data_dir must resolve"),
        memory_dir().expect("memory_dir must resolve"),
        skills_dir().expect("skills_dir must resolve"),
    ] {
        assert!(
            dir.is_dir(),
            "directory {dir:?} must still exist after two ensure_dirs calls"
        );
    }
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_xdg_env_temp_dir_resolves_and_creates_four_dirs() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // Point the XDG env vars at a fresh temp dir, then drive the helpers exactly as C3
    // specifies: resolve the four dirs, create them, and assert all four resolve under
    // the temp roots and exist on disk.
    let base = unique_base("c3_temp");
    let _ = std::fs::remove_dir_all(&base);
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    set_xdg(&xdg_config, &xdg_data);

    let cfg = config_dir().expect("config_dir must resolve under the temp XDG_CONFIG_HOME");
    let data = data_dir().expect("data_dir must resolve under the temp XDG_DATA_HOME");
    let memory = memory_dir().expect("memory_dir must resolve under the temp XDG_DATA_HOME");
    let skills = skills_dir().expect("skills_dir must resolve under the temp XDG_DATA_HOME");

    // The config dir resolves under the temp config root; the data/memory/skills dirs
    // resolve under the temp data root.
    assert!(
        cfg.starts_with(&xdg_config),
        "config_dir {cfg:?} must resolve under {xdg_config:?}"
    );
    for dir in [&data, &memory, &skills] {
        assert!(
            dir.starts_with(&xdg_data),
            "{dir:?} must resolve under the temp XDG_DATA_HOME {xdg_data:?}"
        );
    }

    ensure_dirs().expect("ensure_dirs must create the four directories under the temp roots");

    for dir in [&cfg, &data, &memory, &skills] {
        assert!(
            dir.is_dir(),
            "{dir:?} must exist as a directory under the temp dir after ensure_dirs()"
        );
    }
}
