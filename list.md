# Zira — list.md (live state: status + frozen-test mapping)

The authoritative live document. `status` drives the loop; `criteria_map` + `test_files`
are filled by the RED phase and frozen.

### T-00.01  Scaffold the Cargo workspace
id: T-00.01
phase: 0
status: done
depends_on: []
stack: rust
criteria:
  - C1: the root `Cargo.toml` declares BOTH a `[package]` and a `[workspace]` table with `resolver = "2"`, so `cargo test` at the root runs repo-root `tests/`; `cargo build` at the root exits 0.
  - C2: the workspace `members` list the ten member crates under `crates/`: zira, zira-core, zira-bridge, zira-voice, zira-emotion, zira-avatar, zira-memory, zira-skills, zira-config, zira-proto.
  - C3: `cargo test` exits 0 across the workspace with zero tests defined; `zira` is a binary target and the other nine are library targets.
not_doing:
  - No crate internals beyond an empty lib/bin target each.
  - No dependency wiring beyond what empty crates need to build.
test_files: [tests/workspace_scaffold.rs]
criteria_map:
  C1: [c1_root_is_hybrid_package_and_workspace, c1_workspace_sets_resolver_two]
  C2: [c2_root_declares_workspace_members, c2_all_ten_member_crates_have_named_manifests]
  C3: [c3_zira_is_a_binary_target, c3_other_nine_are_library_targets]
attempts: 1
last_failure: ""
---
The root surface every other task attaches to. Inputs: Cargo manifests only. Outputs: a compiling ten-crate workspace whose root is also a package (so repo-root `tests/` run) and a green empty test run. Errors/edges: a manifest that fails to parse is the only failure, surfaced by cargo. Invariant: the workspace compiles from here forward. Done-check: the three cargo-observable criteria.

### T-00.02  Declare the shared dependencies
id: T-00.02
phase: 0
status: done
depends_on: [T-00.01]
stack: rust
criteria:
  - C1: the root `[workspace.dependencies]` table declares `tokio`, `serde`, `serde_json`, `thiserror`, `anyhow`, and `tracing` with pinned versions.
  - C2: at least one member crate consumes a shared dependency via `{ workspace = true }` and `cargo build` resolves it (proving the workspace-inheritance wiring works).
  - C3: `cargo metadata` exits 0 and the six shared deps appear exactly once in `[workspace.dependencies]` (no per-crate version drift for these).
not_doing:
  - No feature-flag tuning beyond what the crates need to compile.
  - No FFI / audio / GPU dependencies — those belong to later, blocked tasks.
test_files: [tests/shared_dependencies.rs]
criteria_map:
  C1: [c1_workspace_dependencies_declares_six_core_deps, c1_each_shared_dep_has_a_pinned_version]
  C2: [c2_a_member_inherits_a_shared_dep_via_workspace_true, c2_cargo_build_resolves_the_workspace]
  C3: [c3_cargo_metadata_exits_zero, c3_each_shared_dep_appears_exactly_once_in_workspace_dependencies]
attempts: 1
last_failure: ""
---
The shared dependency surface. Inputs: the root manifest. Outputs: `[workspace.dependencies]` with the six core deps and a proven inheritance into a member. Errors/edges: a version that fails to resolve fails `cargo build`. Invariant: core deps are declared once at the root. Done-check: the three criteria.

### T-00.03  Configure the lint policy
id: T-00.03
phase: 0
status: done
depends_on: [T-00.01]
stack: rust
criteria:
  - C1: a `rust-toolchain.toml` pins `channel = "stable"` and includes the `rustfmt` and `clippy` components.
  - C2: a `rustfmt.toml` exists at the workspace root and parses as valid TOML; a repo-root integration test `tests/lint_policy.rs` asserts the file is present and loadable.
  - C3: the root `Cargo.toml` declares a clippy lint policy (a `[workspace.lints.clippy]` table, or `[workspace.lints]` with a clippy entry); the test asserts that table exists. (NOTE: deliberately a CONFIG-PRESENCE check — NOT a workspace-wide `cargo fmt`/`cargo clippy` run, which would be a fragile frozen invariant that later tasks' code could break.)
not_doing:
  - No custom lint authoring beyond enabling the standard rustfmt + clippy gates.
  - No CI changes — the GitHub Actions workflow is maintained outside the loop.
test_files: [tests/lint_policy.rs]
criteria_map:
  C1: [c1_rust_toolchain_file_exists, c1_toolchain_pins_stable_channel, c1_toolchain_includes_rustfmt_component, c1_toolchain_includes_clippy_component]
  C2: [c2_rustfmt_file_exists, c2_rustfmt_file_is_loadable]
  C3: [c3_cargo_toml_has_workspace_lints_clippy_table]
attempts: 3
last_failure: ""
---
The style + lint floor. Inputs: the workspace root. Outputs: toolchain + fmt + clippy config that pass clean on the scaffold. Errors/edges: a malformed toml fails the respective tool. Invariant: fmt and clippy are green from here. Done-check: the three tool-observable criteria.

### T-00.04  Initialize structured logging
id: T-00.04
phase: 0
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: the `zira` binary initializes `tracing-subscriber` with an `EnvFilter` honoring the `RUST_LOG` environment variable.
  - C2: a repo-root integration test `tests/logging_init.rs` asserts the init function is idempotent (a second call does not panic or double-install) and returns a typed result.
  - C3: with `RUST_LOG` unset the subscriber installs at a sane default level (info) rather than silent or trace.
not_doing:
  - No log routing to files or external sinks — stdout/stderr only.
  - No per-crate log configuration beyond the global env filter.
test_files: [tests/logging_init.rs]
criteria_map:
  C1: [test_build_filter_honors_rust_log_error, test_build_filter_defaults_to_info, test_malformed_rust_log_falls_back_to_info]
  C2: [test_first_call_is_ok, test_init_returns_typed_result, test_init_is_idempotent]
  C3: [test_default_level_enables_info, test_default_level_excludes_debug, test_default_level_is_not_silent, test_build_filter_defaults_to_info]
attempts: 1
last_failure: ""
---
Observability from first boot. Inputs: the `RUST_LOG` env var. Outputs: an installed tracing subscriber + an idempotent init. Errors/edges: a malformed filter falls back to the default level, never a panic. Invariant: logging is safe to initialize once. Done-check: the three criteria.

BUILD NOTES (constraints learned from prior blocked attempts — honor exactly):
1. IMPL LOCATION: put `init_logging` in the existing LIBRARY crate `zira-core` (`crates/zira-core/src/lib.rs`, e.g. a `logging` module). The `crates/zira` binary's `main()` merely CALLS it. Do NOT add a `lib.rs` to `crates/zira` — the frozen test `c3_zira_is_a_binary_target` forbids it, and the `[lib] path` work-around is rejected by the checker. The repo-root integration test imports the function as `zira_core::...`.
2. TYPED RESULT (C2): return tracing's OWN error type, e.g. `Result<(), tracing::subscriber::SetGlobalDefaultError>`. Do NOT define a custom error enum with a hand-written `Display`/`Error` impl: the frozen test only checks `is_ok()`, so a custom `Display` is never exercised and its operators survive mutation. An external error type lives outside the task's diff, so it is not mutated.
3. ANTI-STUB TOKEN: never write the literal macro token `todo!(` or `unimplemented!(` ANYWHERE in the test file — not even inside `//` or `///` comments. The anti-stub checker is a line-based substring grep and flags the token regardless of context (this blocked the last attempt at lines 29/49). Describe the RED-fail state in plain words ("init is absent; the test fails to compile / panics until implemented") with no macro token.

### T-00.05  Define the Emotion type
id: T-00.05
phase: 0
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_proto::Emotion` is an enum with exactly the ten variants `Neutral, Happy, Sad, Angry, Excited, Calm, Curious, Concerned, Playful, Tired`, derives `Serialize`/`Deserialize`, and `Default` returns `Neutral`.
  - C2: a repo-root integration test `tests/emotion_type.rs` round-trips every variant through serde JSON and back to the same value.
  - C3: parsing an unknown or malformed tag string maps to `Emotion::Neutral` (case-insensitive match on the known names), pinned by the test.
not_doing:
  - No prosody or expression tables here (those are `zira-emotion`, a later task).
  - No streaming tag parser here (that is `zira-emotion`).
test_files: [tests/emotion_type.rs]
criteria_map:
  C1: [c1_default_is_neutral, c1_all_ten_variants_exist]
  C2: [c2_serde_json_round_trip]
  C3: [c3_from_tag_unknown_maps_to_neutral, c3_from_tag_case_insensitive]
attempts: 1
last_failure: ""
---
The fixed emotion vocabulary shared across TTS and the avatar. Inputs: a variant or a tag string. Outputs: a serde-stable enum defaulting to Neutral, with unknown->Neutral parsing. Errors/edges: an unknown name is Neutral, never an error. Invariant: the ten-name vocabulary is the single source. Done-check: the three criteria.

### T-00.06  Define the State type
id: T-00.06
phase: 0
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_proto::State` is an enum with exactly `Idle, Listening, Transcribing, Thinking, PlanReview, Speaking`, derives `Serialize`/`Deserialize` + `Copy` + `PartialEq`, and `Default` returns `Idle`.
  - C2: a repo-root integration test `tests/state_type.rs` round-trips every variant through serde and asserts `State::default() == State::Idle`.
not_doing:
  - No transition logic here (that is the state machine in `zira-core`).
test_files: [tests/state_type.rs]
criteria_map:
  C1: [c1_default_is_idle, c1_all_six_variants_exist, c1_copy_semantics, c1_partial_eq]
  C2: [c2_serde_json_round_trip, c2_default_is_idle_serde_context]
attempts: 2
last_failure: ""
---
The conversation-state alphabet. Inputs: a variant. Outputs: a serde-stable, copyable enum defaulting to Idle. Errors/edges: none beyond serde. Invariant: these six states are the only states. Done-check: the two criteria.

### T-00.07  Define the payload types
id: T-00.07
phase: 0
status: done
depends_on: [T-00.02, T-00.05]
stack: rust
criteria:
  - C1: `zira_proto` defines structs `Transcript`, `AudioChunk`, `Segment`, `VisemeFrame`, `PlanSummary`, and `Usage`, each deriving `Serialize`/`Deserialize` + `Clone`.
  - C2: `Segment` carries an `Emotion` and the spoken `text`, so an emotion-tagged segment is representable.
  - C3: a repo-root integration test `tests/payload_types.rs` round-trips a populated instance of each of the six structs through serde JSON unchanged.
not_doing:
  - No event wrapping here (the `Event` enum is the next task).
  - No audio decoding — `AudioChunk` is a typed PCM container only.
test_files: [tests/payload_types.rs]
criteria_map:
  C1: [c1_all_six_structs_derive_clone, c1_all_six_structs_derive_serialize_deserialize]
  C2: [c2_segment_carries_emotion_and_text]
  C3: [c3_round_trip_transcript, c3_round_trip_audio_chunk, c3_round_trip_segment, c3_round_trip_viseme_frame, c3_round_trip_plan_summary, c3_round_trip_usage]
attempts: 1
last_failure: ""
---
The data carried between stages. Inputs: stage-produced values. Outputs: six serde-stable payload structs, with `Segment` carrying an `Emotion`. Errors/edges: none beyond serde. Invariant: cross-stage data is typed, not ad-hoc maps. Done-check: the three criteria.

### T-00.08  Define the Event type
id: T-00.08
phase: 0
status: done
depends_on: [T-00.06, T-00.07]
stack: rust
criteria:
  - C1: `zira_proto::Event` is an enum covering the runtime vocabulary: `WakeDetected, SpeechStarted, SpeechEnded, AudioChunk, TranscriptReady, TurnStarted, TextDelta, EmotionSegment, PlanReady, SpeakRequest, VisemeFrame, ExpressionChange, BargeIn, TurnComplete, Error`.
  - C2: the payload-bearing variants carry the matching `zira_proto` payload types (`TranscriptReady(Transcript)`, `EmotionSegment(Segment)`, `VisemeFrame(VisemeFrame)`, `TurnComplete(Usage)`, `Error(String)`), and `Event` derives `Clone` + `Serialize`/`Deserialize`.
  - C3: a repo-root integration test `tests/event_type.rs` round-trips a representative payload-bearing variant and a unit variant through serde unchanged.
not_doing:
  - No bus or dispatch here (that is `zira-core`).
test_files: [tests/event_type.rs]
criteria_map:
  C1: [c1_all_fifteen_variants_exist]
  C2: [c2_payload_bearing_variants_carry_typed_payloads, c2_event_derives_clone, c2_event_derives_serialize_deserialize]
  C3: [c3_round_trip_payload_bearing_variant, c3_round_trip_unit_variant]
attempts: 1
last_failure: ""
---
The single message type on the bus. Inputs: a stage emitting an event. Outputs: a serde-stable enum whose payload variants wrap the typed payloads. Errors/edges: none beyond serde. Invariant: every cross-crate message is an `Event`. Done-check: the three criteria.

### T-00.09  Define the config schema
id: T-00.09
phase: 0
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_config::ZiraConfig` is a serde struct with the sub-sections `paths`, `model`, `wakeword`, `vad`, `stt`, `tts`, `emotion`, `memory`, and `avatar`, each a typed sub-struct.
  - C2: every field has a serde default so a fully-empty TOML document deserializes to a complete `ZiraConfig`.
  - C3: a repo-root integration test `tests/config_schema.rs` deserializes `""` (empty doc) into `ZiraConfig` and asserts the defaults match `ZiraConfig::default()`.
not_doing:
  - No file IO here (loading is the next task).
  - No validation logic here (a later task).
test_files: [tests/config_schema.rs]
criteria_map:
  C1: [c1_zira_config_has_nine_typed_subsections, c1_subsections_derive_serialize_deserialize]
  C2: [c2_empty_toml_deserializes_to_complete_config]
  C3: [c3_empty_doc_equals_default]
attempts: 1
last_failure: ""
---
The typed configuration surface. Inputs: a TOML document (possibly empty). Outputs: a fully-defaulted `ZiraConfig`. Errors/edges: an absent field uses its serde default. Invariant: config is always complete after deserialization. Done-check: the three criteria.

### T-00.10  Load the config file
id: T-00.10
phase: 0
status: done
depends_on: [T-00.09]
stack: rust
criteria:
  - C1: `zira_config::load_from(path)` reads a TOML file into `ZiraConfig`, applying serde defaults for absent fields.
  - C2: a missing file returns `ZiraConfig::default()` (not an error), and a present-but-partial file overlays only its set fields.
  - C3: a repo-root integration test `tests/config_load.rs` writes a partial TOML fixture to a temp dir, loads it, and asserts the set field overrides while unset fields keep their defaults; a missing path yields the default config.
not_doing:
  - No XDG path resolution here (the next task); the loader takes an explicit path.
  - No environment-variable overlay.
test_files: [tests/config_load.rs]
criteria_map:
  C1: [c1_load_from_reads_toml_file_into_zira_config, c1_load_from_applies_serde_defaults_for_absent_fields]
  C2: [c2_missing_file_returns_default_not_error, c2_partial_file_overlays_only_set_fields]
  C3: [c3_partial_fixture_overrides_set_field_while_keeping_defaults, c3_missing_path_yields_default_config]
attempts: 2
last_failure: ""
---
Turning a file into config. Inputs: a filesystem path. Outputs: a `ZiraConfig` with file values over defaults; default on absent file. Errors/edges: a malformed TOML is a typed error; an absent file is the default, not an error. Invariant: loading never panics. Done-check: the three criteria.

### T-00.11  Resolve the data paths
id: T-00.11
phase: 0
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_config` exposes helpers for the config, data, memory, and skills directories rooted under the XDG base dirs (honoring `XDG_CONFIG_HOME`/`XDG_DATA_HOME` when set).
  - C2: a `ensure_dirs()` helper creates any missing directory and is idempotent (a second call succeeds).
  - C3: a repo-root integration test `tests/config_paths.rs` points the XDG env vars at a temp dir, calls the helpers, and asserts the four directories resolve under it and are created.
not_doing:
  - No file content management — directory resolution + creation only.
test_files: [tests/config_paths.rs]
criteria_map:
  C1: [c1_config_dir_under_xdg_config_home, c1_data_dir_under_xdg_data_home, c1_memory_dir_under_data_dir, c1_skills_dir_under_data_dir]
  C2: [c2_ensure_dirs_creates_missing_directories, c2_ensure_dirs_is_idempotent]
  C3: [c3_xdg_env_temp_dir_resolves_and_creates_four_dirs]
attempts: 1
last_failure: ""
---
Where Zira keeps its state on disk. Inputs: the XDG environment. Outputs: four resolved, created directories. Errors/edges: an un-creatable path is a typed error; an existing dir is fine. Invariant: paths honor XDG and are create-if-missing. Done-check: the three criteria.

### T-00.12  Embed the constitution
id: T-00.12
phase: 0
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_config::Constitution` is loaded from an embedded default via `include_str!` (compiled into the binary), so the baseline constitution is always present without a file on disk.
  - C2: the loaded `Constitution` exposes its rules through read-only accessors with no public mutator (immutable after load).
  - C3: a repo-root integration test `tests/constitution.rs` loads the embedded constitution, asserts it is non-empty, and confirms there is no public API to mutate a loaded rule set.
not_doing:
  - No enforcement logic here (that is the `zira-skills` safety gate, a later phase).
  - No on-disk override format yet.
test_files: [tests/constitution.rs]
criteria_map:
  C1: [c1_load_default_requires_no_file, c1_load_default_returns_constitution_directly]
  C2: [c2_rules_readable_from_immutable_binding, c2_rules_returns_shared_slice]
  C3: [c3_embedded_constitution_is_nonempty, c3_no_public_mutator_on_immutable_binding]
attempts: 1
last_failure: ""
---
The immutable baseline policy compiled into Zira. Inputs: the embedded default text. Outputs: a read-only `Constitution`. Errors/edges: a malformed embedded default fails at parse, loudly. Invariant: a loaded constitution cannot be mutated. Done-check: the three criteria.

### T-00.13  Validate the config
id: T-00.13
phase: 0
status: done
depends_on: [T-00.09]
stack: rust
criteria:
  - C1: `ZiraConfig::validate()` returns `Result<(), ConfigError>` where `ConfigError` is a typed `thiserror` enum naming the offending field and reason.
  - C2: validation rejects at least: a non-positive sample rate, an empty model/binary path where one is required, and an out-of-range threshold — each as a distinct `ConfigError` variant.
  - C3: a repo-root integration test `tests/config_validate.rs` asserts a default config validates Ok and that each invalid fixture yields the specific expected `ConfigError`.
not_doing:
  - No auto-repair — validation reports, it does not silently fix.
test_files: [tests/config_validate.rs]
criteria_map:
  C1: [c1_validate_returns_result_unit_config_error, c1_error_names_offending_field_and_reason]
  C2: [c2_zero_sample_rate_is_invalid_sample_rate, c2_empty_binary_path_is_empty_path, c2_threshold_above_range_is_out_of_range, c2_threshold_below_range_is_out_of_range, c2_three_invalid_fields_yield_distinct_variants]
  C3: [c3_default_config_validates_ok, c3_valid_custom_config_validates_ok, c2_zero_sample_rate_is_invalid_sample_rate, c2_empty_binary_path_is_empty_path, c2_threshold_above_range_is_out_of_range]
attempts: 1
last_failure: ""
---
Catching bad config loudly. Inputs: a `ZiraConfig`. Outputs: Ok or a field-specific `ConfigError`. Errors/edges: each invalid field maps to a distinct typed error. Invariant: an invalid config never reaches the runtime silently. Done-check: the three criteria.

### T-00.14  Define the Orchestrator
id: T-00.14
phase: 0
status: done
depends_on: [T-00.06, T-00.08]
stack: rust
criteria:
  - C1: `zira_core::Orchestrator` holds the current `State` (starting `Idle`) and the channel handles for the command + event buses.
  - C2: a constructor builds an `Orchestrator` in `Idle` and exposes a read-only `state()` accessor.
  - C3: a repo-root integration test `tests/orchestrator_new.rs` constructs an `Orchestrator` and asserts its initial `state()` is `State::Idle`.
not_doing:
  - No transition or run-loop logic here (later tasks).
test_files: [tests/orchestrator_new.rs]
criteria_map:
  C1: [c1_orchestrator_accepts_channel_handles]
  C2: [c2_new_builds_orchestrator_in_idle, c2_state_accessor_is_read_only]
  C3: [c3_initial_state_is_idle]
attempts: 1
last_failure: ""
---
The runtime's owner of conversation state. Inputs: channel handles. Outputs: an `Orchestrator` in Idle with a state accessor. Errors/edges: none. Invariant: a fresh orchestrator is Idle. Done-check: the three criteria.

### T-00.15  Build the event bus
id: T-00.15
phase: 0
status: done
depends_on: [T-00.02, T-00.08]
stack: rust
criteria:
  - C1: `zira_core` constructs an mpsc command channel and a broadcast event channel typed over `zira_proto::Event`, returning the sender/receiver handles.
  - C2: a published `Event` is observed by every subscribed broadcast receiver, and the command channel delivers to its single consumer.
  - C3: a repo-root integration test `tests/event_bus.rs` (tokio) publishes an `Event` to two broadcast subscribers and asserts both receive it, and that a command sent on the mpsc channel is received once.
not_doing:
  - No orchestrator wiring here (that is the run loop task).
test_files: [tests/event_bus.rs]
criteria_map:
  C1: [test_create_bus_returns_handles]
  C2: [test_broadcast_fanout_two_subscribers, test_command_channel_single_consumer]
  C3: [test_broadcast_fanout_two_subscribers, test_command_channel_single_consumer]
attempts: 1
last_failure: ""
---
The fan-out spine. Inputs: events + commands. Outputs: a broadcast event channel + an mpsc command channel over `Event`. Errors/edges: a lagging subscriber follows tokio broadcast semantics. Invariant: events fan out to all subscribers. Done-check: the three criteria.

### T-00.16  Define the transition table
id: T-00.16
phase: 0
status: done
depends_on: [T-00.06, T-00.08]
stack: rust
criteria:
  - C1: `zira_core::next_state(current: State, event: &Event) -> Option<State>` implements the PLAN.md §5 table (e.g. `Idle` + `WakeDetected` -> `Listening`; `Speaking` + `BargeIn` -> `Listening`; `Thinking` + `PlanReady` -> `PlanReview`).
  - C2: an event with no defined transition from the current state returns `None` (a no-op), never a panic or a wrong state.
  - C3: a repo-root integration test `tests/transitions.rs` asserts every valid `(state, event)` pair from the table yields the expected next state, and that a sampling of undefined pairs return `None`.
not_doing:
  - No side effects here — `next_state` is a pure function.
  - No timers (the silence timeout is a separate task).
test_files: [tests/transitions.rs]
criteria_map:
  C1: [test_idle_wake_detected_to_listening, test_listening_speech_ended_to_transcribing, test_transcribing_transcript_ready_to_thinking, test_thinking_speak_request_to_speaking, test_thinking_plan_ready_to_plan_review, test_plan_review_turn_started_to_thinking, test_plan_review_error_to_idle, test_speaking_turn_complete_to_idle, test_speaking_barge_in_to_listening, test_thinking_barge_in_to_listening]
  C2: [test_undefined_idle_speech_ended_is_none, test_undefined_idle_barge_in_is_none, test_undefined_idle_turn_complete_is_none, test_undefined_listening_wake_detected_is_none, test_undefined_transcribing_barge_in_is_none, test_undefined_plan_review_speech_ended_is_none, test_undefined_speaking_plan_ready_is_none]
  C3: [test_idle_wake_detected_to_listening, test_listening_speech_ended_to_transcribing, test_transcribing_transcript_ready_to_thinking, test_thinking_speak_request_to_speaking, test_thinking_plan_ready_to_plan_review, test_plan_review_turn_started_to_thinking, test_plan_review_error_to_idle, test_speaking_turn_complete_to_idle, test_speaking_barge_in_to_listening, test_thinking_barge_in_to_listening, test_undefined_idle_speech_ended_is_none, test_undefined_idle_barge_in_is_none, test_undefined_idle_turn_complete_is_none, test_undefined_listening_wake_detected_is_none, test_undefined_transcribing_barge_in_is_none, test_undefined_plan_review_speech_ended_is_none, test_undefined_speaking_plan_ready_is_none]
attempts: 1
last_failure: ""
---
The pure heart of the state machine. Inputs: the current state + an event. Outputs: `Some(next)` for a defined transition, `None` otherwise. Errors/edges: undefined pairs are no-ops. Invariant: transitions are total and pure. Done-check: the three criteria.

### T-00.17  Run the orchestrator loop
id: T-00.17
phase: 0
status: done
depends_on: [T-00.14, T-00.15, T-00.16]
stack: rust
criteria:
  - C1: `Orchestrator::run()` is an async select-loop that consumes events from the bus, applies `next_state`, and updates the held `State` on each defined transition.
  - C2: an undefined transition leaves the state unchanged and the loop continues; a shutdown command exits the loop cleanly.
  - C3: a repo-root integration test `tests/orchestrator_run.rs` (tokio) feeds a scripted event sequence and asserts the orchestrator's `state()` advances through the expected states, then exits on shutdown.
not_doing:
  - No real stages here — events are injected directly in the test.
test_files: [tests/orchestrator_run.rs]
criteria_map:
  C1: [c1_run_advances_state_on_defined_transition]
  C2: [c2_undefined_transition_leaves_state_unchanged, c2_channel_close_exits_loop_cleanly]
  C3: [c3_scripted_sequence_advances_through_expected_states]
attempts: 1
last_failure: ""
---
The live driver. Inputs: events from the bus. Outputs: an advancing `State` + clean shutdown. Errors/edges: undefined transitions are ignored; shutdown exits. Invariant: state only changes via `next_state`. Done-check: the three criteria.

### T-00.18  Log the transitions
id: T-00.18
phase: 0
status: done
depends_on: [T-00.16]
stack: rust
criteria:
  - C1: each applied transition emits a `tracing` event recording `from`, `to`, and the triggering event's discriminant.
  - C2: a no-op (undefined) transition does not emit a state-change log line.
  - C3: a repo-root integration test `tests/transition_log.rs` installs a capturing tracing subscriber, drives one valid and one invalid transition, and asserts exactly one state-change record with the correct from/to was emitted.
not_doing:
  - No metrics or external telemetry — tracing only.
test_files: [tests/transition_log.rs]
criteria_map:
  C1: [test_valid_transition_emits_one_log_record, test_valid_transition_log_has_correct_from_to]
  C2: [test_noop_transition_emits_no_log_record]
  C3: [test_one_valid_one_noop_emits_exactly_one_record]
attempts: 2
last_failure: ""
---
An auditable trail of the conversation flow. Inputs: applied transitions. Outputs: one structured tracing record per real transition. Errors/edges: no-ops are silent. Invariant: every real state change is logged once. Done-check: the three criteria.

### T-00.19  Add the silence timeout
id: T-00.19
phase: 0
status: done
depends_on: [T-00.17]
stack: rust
criteria:
  - C1: while in `Listening`, a configurable silence timeout elapsing with no `SpeechStarted`/`SpeechEnded` drives `Listening -> Idle`.
  - C2: the timer is cancelled/reset when speech activity arrives before it fires, so an active utterance is never cut to Idle.
  - C3: a repo-root integration test `tests/silence_timeout.rs` (tokio, with a paused/advanced clock) asserts the timeout fires `Listening -> Idle` on silence and does NOT fire when speech activity arrives first.
not_doing:
  - No VAD here — the test injects activity events directly.
test_files: [tests/silence_timeout.rs]
criteria_map:
  C1: [c1_silence_timeout_elapses_drives_listening_to_idle]
  C2: [c2_speech_started_cancels_silence_timeout]
  C3: [c3_full_scenario_silence_fires_and_activity_prevents]
attempts: 1
last_failure: ""
---
Returning to rest after silence. Inputs: the Listening state + a clock. Outputs: a `Listening -> Idle` transition on timeout, cancelled by activity. Errors/edges: activity resets the timer. Invariant: only genuine silence returns to Idle. Done-check: the three criteria (deterministic via a controlled clock).

### T-00.20  Define the stage traits
id: T-00.20
phase: 0
status: done
depends_on: [T-00.08]
stack: rust
criteria:
  - C1: `zira_core` defines the stage traits `WakeSource`, `VadGate`, `SttEngine`, `Brain`, `TtsEngine`, `AvatarSink`, and `MemoryStore`, each with the minimal async method(s) the orchestrator needs.
  - C2: a mock implementation of each trait exists (test-only or feature-gated) that emits scripted events without touching real hardware/FFI.
  - C3: a repo-root integration test `tests/stage_traits.rs` drives each mock through its trait method and asserts it produces the expected scripted `Event`(s).
not_doing:
  - No real engines here — the real STT/TTS/wake/avatar impls are blocked-on-human (hardware/FFI/GPU).
test_files: [tests/stage_traits.rs]
criteria_map:
  C1: [c1_wake_source_trait_drives_mock, c1_vad_gate_trait_emits_speech_boundaries, c1_stt_engine_trait_emits_transcript, c1_brain_trait_emits_response_stream, c1_tts_engine_trait_emits_visemes, c1_avatar_sink_trait_emits_expression_change, c1_memory_store_trait_round_trips_event]
  C2: [c2_mock_stt_is_deterministic, c2_mock_brain_is_deterministic, c1_wake_source_trait_drives_mock, c1_vad_gate_trait_emits_speech_boundaries, c1_stt_engine_trait_emits_transcript, c1_brain_trait_emits_response_stream, c1_tts_engine_trait_emits_visemes, c1_avatar_sink_trait_emits_expression_change, c1_memory_store_trait_round_trips_event]
  C3: [c1_wake_source_trait_drives_mock, c1_vad_gate_trait_emits_speech_boundaries, c1_stt_engine_trait_emits_transcript, c1_brain_trait_emits_response_stream, c1_tts_engine_trait_emits_visemes, c1_avatar_sink_trait_emits_expression_change, c1_memory_store_trait_round_trips_event, c2_mock_stt_is_deterministic, c2_mock_brain_is_deterministic]
attempts: 3
last_failure: ""
---
The seam that lets devices be mocked. Inputs: the orchestrator's needs. Outputs: seven traits + a mock each. Errors/edges: mocks are deterministic. Invariant: the orchestrator depends on traits, never concrete engines. Done-check: the three criteria.

### T-00.21  Integrate the mock cycle
id: T-00.21
phase: 0
status: done
depends_on: [T-00.17, T-00.20]
stack: rust
criteria:
  - C1: the orchestrator can be assembled from the seven mock stages and run end-to-end on injected events.
  - C2: a repo-root integration test `tests/mock_cycle.rs` (tokio) drives a full `Idle -> Listening -> Transcribing -> Thinking -> Speaking -> Idle` cycle through the mocked stages and asserts the state path is exactly that sequence.
  - C3: a barge-in event during `Speaking` drives the mocked cycle back to `Listening`, asserted by the same test.
not_doing:
  - No real audio/brain — every stage is a mock; this proves the wiring, not the devices.
test_files: [tests/mock_cycle.rs]
criteria_map:
  C1: [c1_orchestrator_assembled_from_mocks_runs_end_to_end]
  C2: [c2_happy_path_state_sequence_is_exact]
  C3: [c3_barge_in_during_speaking_returns_to_listening]
attempts: 1
last_failure: ""
---
The Phase-0 acceptance: the whole loop cycles on mocks. Inputs: the mock stages + injected events. Outputs: a verified Idle->...->Idle path plus a barge-in path. Errors/edges: barge-in re-enters Listening. Invariant: the state machine + bus + traits compose correctly. Done-check: the three criteria.

### T-01.01  Parse the emotion tag
id: T-01.01
phase: 1
status: done
depends_on: [T-00.05]
stack: rust
criteria:
  - C1: `zira_emotion::parse_tag(s: &str) -> (Emotion, &str)` returns the `Emotion` named by a leading `[emotion:NAME]` marker (case-insensitive, resolved through `Emotion::from_tag`) and the text following the marker with leading whitespace trimmed.
  - C2: input with no leading `[emotion:...]` marker returns `(Emotion::Neutral, s)` with the returned slice byte-for-byte equal to the input.
not_doing:
  - Markers anywhere but the start of the string.
  - Handling more than one marker — that is the segmenter.
test_files: [tests/parse_tag.rs]
criteria_map:
  C1: [c1_leading_marker_extracts_emotion, c1_leading_whitespace_trimmed, c1_case_insensitive_name, c1_unknown_name_resolves_to_neutral, c1_all_known_variants_parseable]
  C2: [c2_no_marker_returns_neutral_and_input, c2_empty_input_returns_neutral_and_empty, c2_no_leading_marker_slice_is_same_bytes, c2_marker_not_at_start_is_no_op]
attempts: 1
last_failure: ""
---
The atom the segmenter is built from. Inputs: a text slice. Outputs: the leading emotion + the remaining text. Edge: an unknown name resolves to Neutral via the proto helper. Invariant: never panics. Done-check: the two criteria.

### T-01.02  Strip the emotion tags
id: T-01.02
phase: 1
status: done
depends_on: [T-00.05]
stack: rust
criteria:
  - C1: `zira_emotion::strip_tags(s: &str) -> String` returns `s` with every `[emotion:...]` marker removed and all surrounding text preserved.
  - C2: a string containing no marker returns a `String` equal to the input.
not_doing:
  - Trimming or normalising prose beyond marker removal.
test_files: [tests/strip_tags.rs]
criteria_map:
  C1: [c1_single_marker_removed, c1_surrounding_text_preserved, c1_multiple_markers_all_removed, c1_marker_at_start_removed, c1_marker_at_end_removed, c1_only_marker_becomes_empty, c1_consecutive_markers_all_removed, c1_all_known_variant_markers_removed]
  C2: [c2_no_marker_returns_equal_string, c2_empty_input_returns_empty_string]
attempts: 1
last_failure: ""
---
Produces the clean text handed to speech. Inputs: tagged text. Outputs: untagged text. Invariant: only markers are removed. Done-check: the two criteria.

### T-01.03  Segment the tagged reply
id: T-01.03
phase: 1
status: done
depends_on: [T-00.07]
stack: rust
criteria:
  - C1: `zira_emotion::segment(s: &str) -> Vec<Segment>` splits `s` at each `[emotion:...]` marker, emitting one `Segment { emotion, text }` per span carrying the emotion in effect for that span.
  - C2: text preceding the first marker becomes a `Segment` with `Emotion::Neutral`; empty input returns an empty `Vec`.
  - C3: a marker immediately followed by another marker or end-of-string emits no empty-text `Segment`.
not_doing:
  - Sentence/clause segmentation — only emotion boundaries split.
test_files: [tests/segment_tags.rs]
criteria_map:
  C1: [c1_single_marker_splits_into_segments, c1_multiple_markers_emit_ordered_spans, c1_emotion_in_effect_for_each_span, c1_all_known_emotions_segmented, c1_concatenated_text_equals_stripped_reply]
  C2: [c2_leading_untagged_text_becomes_neutral_segment, c2_empty_input_returns_empty_vec, c2_only_tagged_text_no_leading_prose]
  C3: [c3_consecutive_markers_emit_no_empty_segment, c3_marker_at_end_emits_no_empty_segment, c3_multiple_consecutive_markers_all_dropped, c3_only_markers_returns_empty_vec]
attempts: 1
last_failure: ""
---
The main emotion parser. Inputs: a full reply. Outputs: ordered emotion spans. Edge: leading untagged prose is Neutral; empty spans are dropped. Invariant: concatenated span text equals the stripped reply. Done-check: the three criteria.

### T-01.04  Map emotion to prosody
id: T-01.04
phase: 1
status: done
depends_on: [T-00.05]
stack: rust
criteria:
  - C1: `zira_emotion::prosody(e: Emotion) -> Prosody` is total over all ten `Emotion` variants and returns a `Prosody { rate: f32, pitch: f32, volume: f32 }`.
  - C2: `prosody(Emotion::Neutral)` equals the baseline `Prosody { rate: 1.0, pitch: 1.0, volume: 1.0 }`.
  - C3: for every variant each of `rate`, `pitch`, `volume` lies within the inclusive range `0.5..=2.0`.
not_doing:
  - Viseme / lip-sync mapping.
  - Per-voice or per-TTS-engine tuning.
test_files: [tests/prosody.rs]
criteria_map:
  C1: [c1_prosody_neutral, c1_prosody_happy, c1_prosody_sad, c1_prosody_angry, c1_prosody_excited, c1_prosody_calm, c1_prosody_curious, c1_prosody_concerned, c1_prosody_playful, c1_prosody_tired, c1_all_ten_variants_return_prosody]
  C2: [c2_neutral_is_baseline]
  C3: [c3_all_variants_in_bounds]
attempts: 1
last_failure: ""
---
The synthesis-facing table. Inputs: an emotion. Outputs: prosody multipliers. Invariant: total and bounded. Done-check: the three criteria.

### T-01.05  Build the claude invocation
id: T-01.05
phase: 1
status: done
depends_on: [T-00.10]
stack: rust
criteria:
  - C1: `zira_bridge::build_argv(cfg: &ZiraConfig) -> Vec<String>` returns the argv that launches the `claude` CLI non-interactively with stream-json output.
  - C2: the model string from the config appears in the argv as the value immediately following the model flag.
not_doing:
  - Spawning the process.
  - Environment or credential handling.
test_files: [tests/build_argv.rs]
criteria_map:
  C1: [c1_argv_starts_with_binary_path, c1_argv_contains_non_interactive_flag, c1_argv_contains_stream_json_output_format]
  C2: [c2_model_id_follows_model_flag]
attempts: 1
last_failure: ""
---
Pure argv construction. Inputs: the config. Outputs: the command vector. Invariant: deterministic for a given config. Done-check: the two criteria.

### T-01.06  Compose the request prompt
id: T-01.06
phase: 1
status: done
depends_on: [T-00.12, T-00.07]
stack: rust
criteria:
  - C1: `zira_bridge::compose_prompt(constitution: &str, transcript: &Transcript) -> String` returns a prompt containing the full constitution text followed by the transcript text, in that order.
  - C2: an empty transcript (`text` is empty) still yields a prompt containing the complete constitution.
not_doing:
  - Memory / context injection (Phase 2).
  - Tool or skill definitions.
test_files: [tests/compose_prompt.rs]
criteria_map:
  C1: [c1_prompt_contains_constitution_then_transcript]
  C2: [c2_empty_transcript_prompt_contains_constitution]
attempts: 1
last_failure: ""
---
Pure prompt assembly. Inputs: constitution + transcript. Outputs: the prompt string. Invariant: constitution is always present and first. Done-check: the two criteria.

### T-01.07  Capture the claude output
id: T-01.07
phase: 1
status: done
depends_on: [T-00.07]
stack: rust
criteria:
  - C1: `zira_bridge::invoke(argv: &[String], prompt: &str) -> std::io::Result<RawOutput>` spawns the program named by `argv`, writes `prompt` to its stdin, and returns a `RawOutput { stdout: String, status: i32 }`.
  - C2: a repo-root integration test `tests/bridge_invoke.rs` runs `invoke` against a stub script that echoes a fixed string and asserts `stdout` equals that string and `status` is `0`.
not_doing:
  - Parsing the captured output — later tasks own that.
test_files: [tests/bridge_invoke.rs]
criteria_map:
  C1: [c1_raw_output_struct_has_stdout_and_status, c1_invoke_writes_prompt_to_stdin]
  C2: [c2_invoke_against_stub_echoes_fixed_string]
attempts: 1
last_failure: ""
---
The subprocess boundary, proven against a stub `claude`. Inputs: argv + prompt. Outputs: raw stdout + exit code. Invariant: stdin is fully written before capture. Done-check: the two criteria.

### T-01.08  Extract the answer text
id: T-01.08
phase: 1
status: done
depends_on: [T-01.07]
stack: rust
criteria:
  - C1: `zira_bridge::parse_answer(raw: &RawOutput) -> String` returns the assistant's final text decoded from claude's stream-json stdout (the terminal `result` event's text).
  - C2: stdout containing no assistant/result text yields an empty `String`.
not_doing:
  - Usage or plan parsing.
test_files: [tests/parse_answer.rs]
criteria_map:
  C1: [c1_result_event_text_returned, c1_result_event_in_multiline_stream, c1_result_event_with_empty_text_returns_empty]
  C2: [c2_empty_stdout_returns_empty, c2_no_result_type_line_returns_empty, c2_only_assistant_lines_returns_empty]
attempts: 1
last_failure: ""
---
Pull the spoken answer from the stream. Inputs: raw output. Outputs: answer text. Edge: missing result yields empty. Done-check: the two criteria.

### T-01.09  Parse the usage totals
id: T-01.09
phase: 1
status: done
depends_on: [T-01.07]
stack: rust
criteria:
  - C1: `zira_bridge::parse_usage(raw: &RawOutput) -> Usage` returns the `Usage { input_tokens, output_tokens }` read from claude's terminal result event.
  - C2: output missing the usage fields yields `Usage { input_tokens: 0, output_tokens: 0 }`.
not_doing:
  - Cost computation — tokens only.
test_files: [tests/parse_usage.rs]
criteria_map:
  C1: [c1_result_event_usage_returned, c1_result_event_usage_in_multiline_stream]
  C2: [c2_empty_stdout_returns_zero_usage, c2_no_result_type_line_returns_zero_usage, c2_result_event_without_usage_field_returns_zero]
attempts: 1
last_failure: ""
---
Read token accounting from the stream. Inputs: raw output. Outputs: a Usage. Edge: absent fields default to zero. Done-check: the two criteria.

### T-01.10  Type the bridge errors
id: T-01.10
phase: 1
status: done
depends_on: [T-01.07]
stack: rust
criteria:
  - C1: `zira_bridge::BridgeError` is an enum implementing `std::error::Error` and `Display` with distinct variants for a spawn failure, a non-zero exit, and output missing a terminal result event.
  - C2: a unit test asserts the `Display` text of every variant is non-empty and names its failure — every variant's message is exercised.
not_doing:
  - Recovery or retry policy.
test_files: [tests/bridge_errors.rs]
criteria_map:
  C1: [test_bridge_error_spawn_failed_variant_exists, test_bridge_error_non_zero_exit_variant_exists, test_bridge_error_missing_result_variant_exists, test_bridge_error_implements_error_trait, test_bridge_error_implements_display]
  C2: [test_display_spawn_failed_non_empty_names_failure, test_display_non_zero_exit_non_empty_names_failure, test_display_missing_result_non_empty_names_failure, test_display_all_variants_produce_distinct_messages]
attempts: 1
last_failure: ""
---
The bridge's typed failure surface. NOTE: C2 deliberately exercises every Display arm so no arm is an unexercised mutation survivor (the T-00.04 lesson). Done-check: the two criteria.

### T-01.11  Ask claude end-to-end
id: T-01.11
phase: 1
status: done
depends_on: [T-01.06, T-01.07, T-01.10]
stack: rust
criteria:
  - C1: `zira_bridge::ask(cfg: &ZiraConfig, constitution: &str, transcript: &Transcript) -> Result<Answer, BridgeError>` composes the prompt, invokes claude, and returns `Answer { text: String, usage: Usage }` on success.
  - C2: a repo-root integration test `tests/bridge_ask.rs` runs `ask` against a stub claude script and asserts the returned `text` and `usage` match the stub output.
  - C3: a stub that exits non-zero makes `ask` return `Err(BridgeError)`, asserted by the same test.
not_doing:
  - Streaming partial deltas to the caller.
test_files: [tests/bridge_ask.rs]
criteria_map:
  C1: [c1_answer_struct_has_text_and_usage, c2_ask_success_returns_answer_from_stub]
  C2: [c2_ask_success_returns_answer_from_stub]
  C3: [c3_ask_non_zero_exit_returns_err]
attempts: 1
last_failure: ""
---
The bridge's public entry point, end-to-end against a stub. Inputs: config + constitution + transcript. Outputs: an Answer or a typed error. Done-check: the three criteria.

### T-01.12  Implement the claude brain
id: T-01.12
phase: 1
status: done
depends_on: [T-00.20, T-01.11, T-01.03]
stack: rust
criteria:
  - C1: `ClaudeBrain` implements the `Brain` trait; `respond()` calls `zira_bridge::ask` and returns a `Vec<Event>`.
  - C2: on success the answer text is run through `zira_emotion::segment` and emitted as one `Event::EmotionSegment(Segment)` per span in order, followed by exactly one `Event::TurnComplete(Usage)`.
not_doing:
  - Streaming `TextDelta` events.
  - The plan-review path.
test_files: [tests/claude_brain.rs]
criteria_map:
  C1: [c1_claude_brain_implements_brain, c1_respond_calls_bridge_ask]
  C2: [c2_respond_emits_segments_then_turn_complete, c2_exactly_one_turn_complete_terminates_turn, c2_turn_complete_carries_bridge_usage]
attempts: 1
last_failure: ""
---
The real Thinking stage, replacing MockBrain. Inputs: a transcript turn. Outputs: emotion-segment events + a turn-complete. Invariant: exactly one TurnComplete terminates a successful turn. Done-check: the two criteria.

### T-01.13  Emit the bridge error event
id: T-01.13
phase: 1
status: done
depends_on: [T-01.12]
stack: rust
criteria:
  - C1: when `zira_bridge::ask` returns `Err`, `ClaudeBrain::respond()` returns exactly one `Event::Error(String)` carrying the error's `Display` message and never panics.
not_doing:
  - Retry or backoff — the orchestrator decides recovery.
test_files: [tests/bridge_error_event.rs]
criteria_map:
  C1: [c1_spawn_failed_emits_single_error_event, c1_non_zero_exit_emits_single_error_event, c1_missing_result_emits_single_error_event, c1_error_carries_display_message]
attempts: 2
last_failure: ""
---
The failure path of the Thinking stage. Inputs: a failing ask. Outputs: a single Error event. Invariant: no panic on bridge failure. Done-check: the one criterion.

### T-01.14  Test the thinking spine
id: T-01.14
phase: 1
status: pending
depends_on: [T-01.12, T-01.13]
stack: rust
criteria:
  - C1: a repo-root integration test `tests/thinking_spine.rs` (tokio) drives `ClaudeBrain::respond()` against a stub claude script and asserts the emitted `Event` sequence is the expected `EmotionSegment`(s) then `TurnComplete`.
  - C2: a stub reply carrying multiple `[emotion:...]` spans produces one `Event::EmotionSegment` per span in source order; a stub that fails produces a single `Event::Error`.
not_doing:
  - Audio stages — those stay mocked / blocked-on-human.
test_files: []
criteria_map: {}
attempts: 1
last_failure: ""
---
Phase-1 acceptance for the gateable half: transcript -> claude -> emotion-segmented events. Done-check: the two criteria.

### T-01.15  Detect the wake word
id: T-01.15
phase: 1
status: blocked
depends_on: [T-00.20, T-00.10]
stack: rust
criteria:
  - C1: a `WakeSource` implementation backed by a real wake-word model emits `Event::WakeDetected` when the configured wake phrase is spoken into the default input device.
not_doing:
  - Mock wake source — that already exists from Phase 0.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: needs audio hardware + FFI models; tracked, not attempted by the loop."
---
Real wake detection. Blocked-on-human: requires microphone hardware + a wake-word model (FFI). Done-check: the one criterion, measured on target hardware.

### T-01.16  Gate the voice activity
id: T-01.16
phase: 1
status: blocked
depends_on: [T-00.20]
stack: rust
criteria:
  - C1: a `VadGate` implementation emits `Event::SpeechStarted` and `Event::SpeechEnded` from live microphone audio using a real voice-activity detector.
not_doing:
  - Mock VAD gate — exists from Phase 0.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: needs audio hardware + FFI models; tracked, not attempted by the loop."
---
Real endpointing. Blocked-on-human: microphone hardware + a VAD model. Done-check: the one criterion on target hardware.

### T-01.17  Transcribe the speech
id: T-01.17
phase: 1
status: blocked
depends_on: [T-00.20]
stack: rust
criteria:
  - C1: an `SttEngine` implementation transcribes captured microphone audio into an `Event::TranscriptReady(Transcript)` via a real speech-to-text model.
not_doing:
  - Mock STT engine — exists from Phase 0.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: needs audio hardware + FFI models; tracked, not attempted by the loop."
---
Real transcription. Blocked-on-human: an STT model/FFI + audio capture. Done-check: the one criterion on target hardware.

### T-01.18  Synthesize the speech
id: T-01.18
phase: 1
status: blocked
depends_on: [T-00.20, T-01.04]
stack: rust
criteria:
  - C1: a `TtsEngine` implementation synthesizes a `Segment`'s text into audible speech on the default output device, modulated by the segment emotion's `Prosody`.
not_doing:
  - Mock TTS engine — exists from Phase 0.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: needs audio hardware + FFI models; tracked, not attempted by the loop."
---
Real emotion-inflected speech. Blocked-on-human: a TTS model + audio output. Done-check: the one criterion on target hardware.
