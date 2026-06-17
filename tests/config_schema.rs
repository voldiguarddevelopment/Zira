//! Frozen tests for T-00.09 — "Define the config schema".
//!
//! Each acceptance criterion maps to ≥1 test:
//!
//!   C1 -> c1_zira_config_has_nine_typed_subsections,
//!          c1_subsections_derive_serialize_deserialize
//!   C2 -> c2_empty_toml_deserializes_to_complete_config
//!   C3 -> c3_empty_doc_equals_default

use zira_config::{
    AvatarConfig, EmotionConfig, MemoryConfig, ModelConfig, PathsConfig, SttConfig, TtsConfig,
    VadConfig, WakewordConfig, ZiraConfig,
};

// ---- C1 -------------------------------------------------------------------------------

#[test]
fn c1_zira_config_has_nine_typed_subsections() {
    // Compile-time proof: ZiraConfig has all 9 sub-section fields with the correct
    // distinct typed sub-structs. Type-annotated bindings ensure both field name and type
    // are correct — wrong name or wrong type fails to compile.
    let cfg = ZiraConfig::default();
    let _: &PathsConfig = &cfg.paths;
    let _: &ModelConfig = &cfg.model;
    let _: &WakewordConfig = &cfg.wakeword;
    let _: &VadConfig = &cfg.vad;
    let _: &SttConfig = &cfg.stt;
    let _: &TtsConfig = &cfg.tts;
    let _: &EmotionConfig = &cfg.emotion;
    let _: &MemoryConfig = &cfg.memory;
    let _: &AvatarConfig = &cfg.avatar;
}

#[test]
fn c1_subsections_derive_serialize_deserialize() {
    // Each sub-section must carry serde derives. A successful JSON round-trip of a
    // default ZiraConfig proves Serialize and Deserialize are both implemented.
    let original = ZiraConfig::default();
    let json = serde_json::to_string(&original).expect("ZiraConfig must serialize");
    let restored: ZiraConfig =
        serde_json::from_str(&json).expect("ZiraConfig must deserialize");
    assert_eq!(
        original, restored,
        "ZiraConfig JSON round-trip must preserve equality"
    );
}

// ---- C2 -------------------------------------------------------------------------------

#[test]
fn c2_empty_toml_deserializes_to_complete_config() {
    // An empty TOML document must deserialize to a fully-populated ZiraConfig without
    // error. Every field must carry a serde default so no key is required in the document.
    let result: Result<ZiraConfig, _> = toml::from_str("");
    assert!(
        result.is_ok(),
        "empty TOML must deserialize to a complete ZiraConfig; got: {result:?}"
    );
    // Binding each sub-section confirms the returned config is fully populated.
    let cfg = result.unwrap();
    let _: &PathsConfig = &cfg.paths;
    let _: &ModelConfig = &cfg.model;
    let _: &WakewordConfig = &cfg.wakeword;
    let _: &VadConfig = &cfg.vad;
    let _: &SttConfig = &cfg.stt;
    let _: &TtsConfig = &cfg.tts;
    let _: &EmotionConfig = &cfg.emotion;
    let _: &MemoryConfig = &cfg.memory;
    let _: &AvatarConfig = &cfg.avatar;
}

// ---- C3 -------------------------------------------------------------------------------

#[test]
fn c3_empty_doc_equals_default() {
    // The canonical C3 test: an empty TOML document must deserialize to exactly the
    // same value as ZiraConfig::default().
    let from_empty: ZiraConfig =
        toml::from_str("").expect("empty TOML must deserialize to ZiraConfig");
    assert_eq!(
        from_empty,
        ZiraConfig::default(),
        "ZiraConfig from empty TOML must equal ZiraConfig::default()"
    );
}
