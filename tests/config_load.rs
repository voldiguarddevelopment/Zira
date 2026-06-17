//! Frozen tests for T-00.10 — "Load the config file".
//!
//! Criterion → test mapping:
//!
//!   C1 -> c1_load_from_reads_toml_file_into_zira_config,
//!          c1_load_from_applies_serde_defaults_for_absent_fields
//!   C2 -> c2_missing_file_returns_default_not_error,
//!          c2_partial_file_overlays_only_set_fields
//!   C3 -> c3_partial_fixture_overrides_set_field_while_keeping_defaults,
//!          c3_missing_path_yields_default_config

use std::path::{Path, PathBuf};
use zira_config::{
    AvatarConfig, EmotionConfig, MemoryConfig, ModelConfig, PathsConfig, SttConfig, TtsConfig,
    VadConfig, WakewordConfig, ZiraConfig,
};

/// Write `content` into a per-test TOML file under the Cargo-managed temp dir and return the path.
fn write_fixture(test_name: &str, content: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let dir = base.join("config_load").join(test_name);
    std::fs::create_dir_all(&dir).expect("create fixture subdir");
    let path = dir.join("config.toml");
    std::fs::write(&path, content).expect("write TOML fixture");
    path
}

/// Return a path that is guaranteed not to exist (inside an uncreated subdir).
fn absent_path() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    base.join("config_load_absent").join("does_not_exist.toml")
}

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_load_from_reads_toml_file_into_zira_config() {
    // A file with a concrete value must produce a ZiraConfig carrying that value.
    let path = write_fixture(
        "c1_reads",
        "[model]\nmodel_id = \"claude-3-opus\"\nbinary_path = \"/usr/bin/claude\"\n",
    );
    let cfg = zira_config::load_from(&path)
        .expect("load_from must succeed for a valid TOML file");
    assert_eq!(
        cfg.model.model_id, "claude-3-opus",
        "model_id must match the value written to the TOML file"
    );
    assert_eq!(
        cfg.model.binary_path, "/usr/bin/claude",
        "binary_path must match the value written to the TOML file"
    );
}

#[test]
fn c1_load_from_applies_serde_defaults_for_absent_fields() {
    // A file that sets only one section must leave all other sections at their defaults.
    let path = write_fixture(
        "c1_defaults",
        "[model]\nmodel_id = \"set-value\"\n",
    );
    let cfg = zira_config::load_from(&path)
        .expect("load_from must succeed for a partial TOML file");
    assert_eq!(
        cfg.vad,
        VadConfig::default(),
        "absent [vad] section must equal VadConfig::default()"
    );
    assert_eq!(
        cfg.emotion,
        EmotionConfig::default(),
        "absent [emotion] section must equal EmotionConfig::default()"
    );
    assert_eq!(
        cfg.memory,
        MemoryConfig::default(),
        "absent [memory] section must equal MemoryConfig::default()"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_missing_file_returns_default_not_error() {
    // A path that does not exist must yield Ok(ZiraConfig::default()), never Err.
    let path = absent_path();
    // Confirm the path is truly absent (the parent dir was never created).
    assert!(
        !path.exists(),
        "test precondition: absent_path() must not exist, but found {path:?}"
    );
    let result = zira_config::load_from(&path);
    let cfg = result.expect("missing file must return Ok(ZiraConfig::default()), not Err");
    assert_eq!(
        cfg,
        ZiraConfig::default(),
        "missing file must yield exactly ZiraConfig::default()"
    );
}

#[test]
fn c2_partial_file_overlays_only_set_fields() {
    // A partial TOML that sets only [tts].voice must not disturb any other section.
    let path = write_fixture(
        "c2_partial",
        "[tts]\nvoice = \"en_US-amy\"\n",
    );
    let cfg = zira_config::load_from(&path)
        .expect("partial TOML must load successfully");
    assert_eq!(
        cfg.tts.voice, "en_US-amy",
        "set field [tts].voice must carry the TOML value"
    );
    // Every other top-level section must be the default.
    let _: &PathsConfig = &cfg.paths;
    let _: &ModelConfig = &cfg.model;
    let _: &WakewordConfig = &cfg.wakeword;
    let _: &VadConfig = &cfg.vad;
    let _: &SttConfig = &cfg.stt;
    let _: &EmotionConfig = &cfg.emotion;
    let _: &MemoryConfig = &cfg.memory;
    let _: &AvatarConfig = &cfg.avatar;
    assert_eq!(cfg.model, ModelConfig::default(), "unset [model] must remain default");
    assert_eq!(cfg.wakeword, WakewordConfig::default(), "unset [wakeword] must remain default");
    assert_eq!(cfg.vad, VadConfig::default(), "unset [vad] must remain default");
    assert_eq!(cfg.stt, SttConfig::default(), "unset [stt] must remain default");
    assert_eq!(cfg.emotion, EmotionConfig::default(), "unset [emotion] must remain default");
    assert_eq!(cfg.memory, MemoryConfig::default(), "unset [memory] must remain default");
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_partial_fixture_overrides_set_field_while_keeping_defaults() {
    // Write a partial TOML fixture to a temp dir (as C3 specifies), load it, and assert
    // that the set field is overridden while every unset field keeps its default.
    let path = write_fixture(
        "c3_partial_fixture",
        "[memory]\nmax_episodes = 42\n",
    );
    let cfg = zira_config::load_from(Path::new(&path))
        .expect("partial fixture must load without error");
    assert_eq!(
        cfg.memory.max_episodes, 42,
        "set field [memory].max_episodes must equal 42"
    );
    assert_eq!(cfg.paths, PathsConfig::default(), "[paths] must be default");
    assert_eq!(cfg.model, ModelConfig::default(), "[model] must be default");
    assert_eq!(cfg.wakeword, WakewordConfig::default(), "[wakeword] must be default");
    assert_eq!(cfg.vad, VadConfig::default(), "[vad] must be default");
    assert_eq!(cfg.stt, SttConfig::default(), "[stt] must be default");
    assert_eq!(cfg.tts, TtsConfig::default(), "[tts] must be default");
    assert_eq!(cfg.emotion, EmotionConfig::default(), "[emotion] must be default");
    assert_eq!(cfg.avatar, AvatarConfig::default(), "[avatar] must be default");
}

#[test]
fn c3_missing_path_yields_default_config() {
    // The C3 criterion also requires: a missing path yields the default config.
    let path = absent_path();
    assert!(
        !path.exists(),
        "test precondition: absent_path() must not exist, but found {path:?}"
    );
    let cfg = zira_config::load_from(&path)
        .expect("absent path must yield Ok(ZiraConfig::default()), not Err");
    assert_eq!(
        cfg,
        ZiraConfig::default(),
        "absent path must equal ZiraConfig::default()"
    );
}
