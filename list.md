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
status: done
depends_on: [T-01.12, T-01.13]
stack: rust
criteria:
  - C1: a repo-root integration test `tests/thinking_spine.rs` (tokio) drives `ClaudeBrain::respond()` against a stub claude script and asserts the emitted `Event` sequence is the expected `EmotionSegment`(s) then `TurnComplete`.
  - C2: a stub reply carrying multiple `[emotion:...]` spans produces one `Event::EmotionSegment` per span in source order; a stub that fails produces a single `Event::Error`.
not_doing:
  - Audio stages — those stay mocked / blocked-on-human.
test_files: [tests/thinking_spine.rs]
criteria_map:
  C1: [c1_thinking_spine_emits_segments_then_turn_complete]
  C2: [c2_multiple_emotion_spans_produce_segments_in_source_order, c2_bridge_failure_produces_single_error_event]
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
status: done
depends_on: [T-00.20]
stack: rust
criteria:
  - C1: `zira_voice::WhisperStt::load(model_dir: &std::path::Path) -> Result<WhisperStt, SttError>` loads a Candle whisper model (config.json + tokenizer.json + model.safetensors + melfilters.bytes) on the CPU, and `WhisperStt` implements `zira_core::SttEngine`.
  - C2: `WhisperStt::transcribe_pcm(&mut self, pcm: &[f32]) -> Result<String, SttError>` pads/clamps the 16 kHz audio to whisper's 30-second window, computes the log-mel spectrogram, runs the encoder + a greedy decode, and returns the transcript text.
  - C3: a repo-root integration test `tests/whisper_stt.rs` (env-gated on `ZIRA_STT_MODEL`, default `~/.cache/zira/models/whisper-tiny.en`, returning early when the dir is absent so a model-less CI stays green) loads the model + the 16 kHz `jfk.wav` fixture in that dir and asserts the transcript lowercased contains both `country` and `americans` and is at least 60 characters — proving real ASR, not a stub.
  - C4: constructed with the fixture audio via `WhisperStt::with_audio(pcm)`, the `SttEngine::transcribe` impl yields `Event::TranscriptReady(Transcript { text })` whose text equals the direct `transcribe_pcm` result; a decode failure yields `Event::Error` rather than a panic.
  - C5: `zira_voice::SttError` implements `std::error::Error` + `Display` with distinct variants for a missing model file, a model-load failure, and an audio/decode failure; a unit test exercises every variant's `Display`.
not_doing:
  - No live microphone capture — the engine transcribes a supplied PCM buffer; mic I/O stays device-bound.
  - No GPU/CUDA — CPU only.
  - No streaming/partial transcription — one utterance at a time.
test_files: [tests/whisper_stt.rs]
criteria_map:
  C1: [_c1_c2_api_pins, c1_c2_c3_real_asr]
  C2: [_c1_c2_api_pins, c1_c2_c3_real_asr]
  C3: [c1_c2_c3_real_asr]
  C4: [_c4_api_pins, c4_transcribe_impl_emits_transcript_ready_matching_pcm, c4_transcribe_impl_maps_failure_to_error_event]
  C5: [c5_stt_error_implements_error_and_display, c5_display_exercises_every_variant]
attempts: 2
last_failure: ""
---
PROVEN RECIPE (a spike transcribed the jfk fixture verbatim — reproduce it). Deps are in `crates/zira-voice/Cargo.toml`: candle-core/nn/transformers 0.8, tokenizers 0.21, hound 3.5, byteorder 1. LOAD: `Config` via serde_json from config.json; `Tokenizer::from_file(tokenizer.json)`; `VarBuilder::from_mmaped_safetensors([model.safetensors], whisper::DTYPE, &Device::Cpu)`; `whisper::model::Whisper::load(&vb, cfg)`; read melfilters.bytes as little-endian f32 (`byteorder` `read_f32_into`) into a Vec<f32> (80*201). TRANSCRIBE_PCM(pcm): call `model.reset_kv_cache()`; pad pcm to `16000*30` with 0.0; `mel = whisper::audio::pcm_to_mel(&cfg, &pcm, &mel_filters)`; `frames = mel.len()/cfg.num_mel_bins`; build a `(1, num_mel_bins, frames)` Tensor and if frames>3000 `narrow(2, 0, 3000)`; `features = model.encoder.forward(&mel, true)`. GREEDY: `tokens = vec![50257u32 /*SOT*/, 50362u32 /*no_timestamps*/]`; loop up to 224: `t = Tensor::new(tokens, dev).unsqueeze(0)`; `ys = model.decoder.forward(&t, &features, i == 0)` (flush kv-cache ONLY on step 0 so the cache persists; flushing every step is O(n^2) and far too slow in debug); `last = ys.narrow(1, ys.dim(1)-1, 1)`; `logits = model.decoder.final_linear(&last).squeeze(0).squeeze(0)`; `next = logits.argmax(0).to_scalar::<u32>()`; break if next==50256 (EOT) else push. `text = tokenizer.decode(&tokens[2..], true)`. The model needs `&mut self` (kv-cache). The model dir + jfk.wav live at $ZIRA_STT_MODEL — never commit weights. Map every candle/io/tokenizer failure to an `SttError` variant and exercise each Display (the T-01.10 lesson). Verified spike output: the full JFK quote. Done-check: the five criteria.

### T-01.18  Synthesize the speech
id: T-01.18
phase: 1
status: pending
depends_on: [T-00.20]
stack: rust
criteria:
  - C1: `zira_voice::PiperTts::load(voice_dir: &std::path::Path) -> Result<PiperTts, TtsError>` loads a Piper VITS voice (`<voice>.onnx` + `<voice>.onnx.json`) via the ONNX runtime, and `PiperTts` implements `zira_core::TtsEngine`.
  - C2: `PiperTts::synth(&mut self, text: &str) -> Result<Vec<f32>, TtsError>` phonemizes the text with espeak-ng, maps the phonemes to the voice's phoneme ids, runs the model, and returns the f32 PCM audio at the voice's sample rate.
  - C3: a repo-root integration test `tests/piper_tts.rs` (env-gated on `ZIRA_TTS_MODEL`, default `~/.cache/zira/models/piper/en_US-lessac-medium`, returning early when the dir is absent so a model-less CI stays green) synthesizes a non-trivial phrase and asserts the audio has at least 6000 samples (~0.27s) and a peak amplitude strictly within `(0.05, 2.0)` — proving real speech, not silence or garbage.
  - C4: constructed with a phrase, the `TtsEngine::speak` impl returns a `Vec<Event>` of `Event::VisemeFrame(VisemeFrame { .. })` — one frame per phoneme — and every frame's `weight` is within `0.0..=1.0`.
  - C5: `zira_voice::TtsError` implements `std::error::Error` + `Display` with distinct variants for a missing model file, a phonemizer (espeak-ng) failure, and an inference failure; a unit test exercises every variant's `Display`.
not_doing:
  - No live audio playback — the engine returns a PCM buffer; speaker I/O stays device-bound.
  - No emotion/prosody modulation yet — flat synthesis for now.
  - No streaming — one phrase synthesized at once.
test_files: []
criteria_map: {}
attempts: 3
last_failure: |
  surviving mutant at crates/zira-voice/src/tts.rs:235 (bool-and-to-or) — frozen tests do not kill it
  surviving mutant at crates/zira-voice/src/tts.rs:296 (bool-or-to-and) — frozen tests do not kill it
---
PROVEN RECIPE (a spike synthesized 2.01s of real audio — reproduce it). Deps in `crates/zira-voice/Cargo.toml`: `ort = "=2.0.0-rc.10"` (features download-binaries), serde_json; plus the system `espeak-ng` binary (present). LOAD: parse `<voice>.onnx.json` with serde_json for `phoneme_id_map`, `audio.sample_rate` (22050), and `inference` {noise_scale 0.667, length_scale 1.0, noise_w 0.8}; `ort::session::Session::builder()?.commit_from_file(onnx)`. SYNTH(text): run `espeak-ng -q --ipa -v en-us <text>` via std::process::Command and read stdout as the phoneme string. Build ids with `let id = |k: &str| phoneme_id_map[k][0].as_i64()`: `ids = vec![id("^") /*BOS*/, id("_") /*pad*/]`; for each char of the phonemes, if it maps push `id(char)` then the pad; finally push `id("$")` /*EOS*/. INFERENCE: `input` = i64 Tensor shape [1, len]; `input_lengths` = i64 Tensor [len]; `scales` = f32 Tensor [noise_scale, length_scale, noise_w]; the `session` must be `mut`; `session.run(ort::inputs!["input"=>input, "input_lengths"=>il, "scales"=>sc])`; extract with `outputs["output"].try_extract_tensor::<f32>()` (the output shape is [1,1,1,N]; flatten to the N PCM samples). VISEMES (C4): map each phoneme to one `VisemeFrame` (a vowel to an open mouth shape weight ~1.0, a consonant to a partial weight; a short viseme label string), in order. Map espeak/ort/io failures to a `TtsError` variant and exercise each Display (the T-01.10 lesson). The voice lives at $ZIRA_TTS_MODEL — never commit the 63MB onnx. TTS is a single forward pass (fast), no decode loop. Verified spike: 'hello world...' -> 2.01s @ 22050Hz, peak 0.653. Done-check: the five criteria.

### T-02.01  Declare the memory dependencies
id: T-02.01
phase: 2
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: the root `[workspace.dependencies]` table declares `redb` with a pinned version, and `crates/zira-memory/Cargo.toml` inherits `redb`, `serde`, `serde_json`, and `zira-proto` via `{ workspace = true }` (or path); `cargo build -p zira-memory` exits 0.
  - C2: a repo-root integration test `tests/memory_deps.rs` asserts `cargo metadata` exits 0 and that `redb` appears exactly once in `[workspace.dependencies]`, proving no per-crate version drift.
not_doing:
  - No memory logic — manifests and dependency wiring only.
  - No FFI / GPU / model-download dependencies; those belong to the blocked embedder task.
test_files: [tests/memory_deps.rs]
criteria_map:
  C1: [c1_redb_declared_in_workspace_dependencies, c1_redb_has_pinned_version_in_workspace, c1_zira_memory_inherits_memory_deps, c1_cargo_build_zira_memory_exits_zero]
  C2: [c2_cargo_metadata_exits_zero, c2_redb_appears_exactly_once_in_workspace_dependencies]
attempts: 2
last_failure: ""
---
The dependency surface Phase-2 builds on. Inputs: the workspace and zira-memory manifests. Outputs: redb declared once at the root and inherited into zira-memory alongside the serde/proto wiring, proven by a green build. Edge: a version that fails to resolve fails cargo build. Invariant: redb is declared exactly once at the root. Done-check: the two cargo-observable criteria.

### T-02.02  Define the episode record
id: T-02.02
phase: 2
status: done
depends_on: [T-02.01]
stack: rust
criteria:
  - C1: `zira_memory::Episode` is a struct with at least `role: String`, `text: String`, and `timestamp: u64` fields, deriving `serde::Serialize`, `serde::Deserialize`, `Clone`, `PartialEq`, and `Debug`.
  - C2: a unit test round-trips an `Episode` through `serde_json::to_string` then `serde_json::from_str` and asserts the recovered value equals the original (`PartialEq`).
not_doing:
  - No persistence — the on-disk JSONL append/load is later tasks.
  - No embedding vector field — retrieval stores vectors in the index, not the episode.
test_files: [tests/episode_record.rs]
criteria_map:
  C1: [c1_episode_has_required_fields, c1_episode_derives_clone, c1_episode_derives_partial_eq, c1_episode_derives_debug, c1_empty_string_fields_are_valid]
  C2: [c2_episode_serde_json_round_trip, c2_episode_empty_strings_round_trip]
attempts: 1
last_failure: ""
---
The episodic memory unit. Inputs: constructed in-memory. Outputs: a serde-round-trippable record. Edge: an empty-string field is still a valid episode. Invariant: an episode serialized then deserialized is unchanged. Done-check: the struct shape and the round-trip criterion.

### T-02.03  Append one episode
id: T-02.03
phase: 2
status: done
depends_on: [T-02.02]
stack: rust
criteria:
  - C1: `zira_memory::append_episode(path: &std::path::Path, episode: &zira_memory::Episode) -> std::io::Result<()>` serializes `episode` to one JSON line and appends it (with a trailing newline) to the file at `path`, creating the file if absent.
  - C2: a test appends two episodes to a temp-dir path, reads the raw file back, and asserts it has exactly two newline-terminated lines each parsing as the corresponding `Episode`.
not_doing:
  - No cap enforcement — that is T-02.05.
  - No locking or concurrent-writer coordination.
test_files: [tests/append_episode.rs]
criteria_map:
  C1: [c1_append_creates_file_when_absent, c1_append_writes_valid_json_line]
  C2: [c2_append_two_episodes_exact_lines]
attempts: 1
last_failure: ""
---
The episodic write primitive. Inputs: a path and an episode. Outputs: one appended JSONL line; the io::Result surfaces any filesystem error verbatim (reusing std::io::Error so no custom Display needs exercising). Edge: a missing file is created, not an error. Invariant: each call adds exactly one line and never rewrites prior lines. Done-check: the append-and-read-back criterion.

### T-02.04  Load the episodes
id: T-02.04
phase: 2
status: done
depends_on: [T-02.03]
stack: rust
criteria:
  - C1: `zira_memory::load_episodes(path: &std::path::Path) -> std::io::Result<Vec<zira_memory::Episode>>` reads the JSONL file and returns its episodes in file order; a non-existent path returns `Ok(vec![])` rather than an error.
  - C2: a test writes three episodes via `append_episode`, calls `load_episodes`, and asserts the returned vec equals the three originals in append order.
not_doing:
  - No filtering, ranking, or dedup — load is a faithful read.
  - No tolerance design for malformed lines beyond surfacing a parse error.
test_files: [tests/load_episodes.rs]
criteria_map:
  C1: [c1_missing_file_returns_empty_vec, c1_load_episodes_reads_file_in_order]
  C2: [c2_append_then_load_three_episodes]
attempts: 1
last_failure: ""
---
The episodic read primitive, the inverse of append. Inputs: a path. Outputs: the episodes in order, or an empty vec for a missing file. Edge: a missing file is empty-not-error; a malformed line surfaces an io::Error. Invariant: load after appends returns exactly what was appended, in order. Done-check: the append-then-load round-trip criterion.

### T-02.05  Enforce the episode cap
id: T-02.05
phase: 2
status: done
depends_on: [T-02.04]
stack: rust
criteria:
  - C1: `zira_memory::cap_episodes(path: &std::path::Path, max_episodes: usize) -> std::io::Result<()>` rewrites the JSONL file to retain only the most recent `max_episodes` episodes, preserving their order; if the file already holds `<= max_episodes` it is left unchanged.
  - C2: a test appends five episodes, calls `cap_episodes(path, 3)`, then `load_episodes`, and asserts exactly the last three episodes remain in order.
  - C3: a test with `max_episodes` of 0 leaves the file empty (zero episodes) and returns `Ok(())`.
not_doing:
  - No reading of the cap from config — the caller passes `MemoryConfig::max_episodes`.
  - No time-based or size-based eviction; count-based only.
test_files: [tests/cap_episodes.rs]
criteria_map:
  C1: [c1_under_cap_file_is_unchanged, c2_cap_five_to_three_retains_last_three]
  C2: [c2_cap_five_to_three_retains_last_three]
  C3: [c3_cap_zero_empties_the_file]
attempts: 2
last_failure: ""
---
The laziness-breaking bound on episodic growth, fed by `zira_config::MemoryConfig::max_episodes`. Inputs: a path and a max count. Outputs: a truncated-from-the-front file. Edge: a cap of 0 empties the file; an under-cap file is untouched. Invariant: after capping, the file holds at most `max_episodes` most-recent episodes in order. Done-check: the truncate-to-three and cap-zero criteria.

### T-02.06  Type the fact-store errors
id: T-02.06
phase: 2
status: done
depends_on: [T-02.01]
stack: rust
criteria:
  - C1: `zira_memory::FactStoreError` is an enum implementing `std::error::Error` and `Display`, with distinct variants for an open/database failure, a transaction failure, and a (de)serialization failure.
  - C2: a unit test asserts the `Display` text of every variant is non-empty, names its failure, and that all variants produce mutually distinct messages — every Display arm is exercised.
not_doing:
  - No retry or recovery policy.
  - No variant for a missing key — a missing get returns `Ok(None)`, not an error.
test_files: [tests/fact_store_error.rs]
criteria_map:
  C1: [c1_fact_store_error_implements_error_and_display]
  C2: [c2_display_messages_are_nonempty_named_and_distinct]
attempts: 1
last_failure: ""
---
The fact store's typed failure surface, declared before the store so its operations can return it. NOTE: C2 deliberately exercises every Display arm so no arm is an unexercised mutation survivor (the T-01.10 lesson). Inputs: none — constructed in tests. Outputs: distinct, non-empty, failure-naming messages. Invariant: each variant has its own message. Done-check: the two criteria.

### T-02.07  Open the fact store
id: T-02.07
phase: 2
status: done
depends_on: [T-02.06]
stack: rust
criteria:
  - C1: `zira_memory::FactStore::open(path: &std::path::Path) -> Result<zira_memory::FactStore, zira_memory::FactStoreError>` opens (creating if absent) a redb database at `path` and returns a usable handle.
  - C2: a test opens a store at a fresh temp-dir path, asserts `Ok`, then re-opens the same path and asserts `Ok` again (the on-disk database persists across opens).
not_doing:
  - No put/get/delete — those are T-02.08 through T-02.10.
  - No schema migration handling.
test_files: [tests/fact_store_open.rs]
criteria_map:
  C1: [c1_open_creates_database_when_absent]
  C2: [c2_reopen_same_path_returns_ok]
attempts: 1
last_failure: ""
---
The redb-backed semantic store's lifecycle entry point. Inputs: a database path. Outputs: an open FactStore handle, or a typed FactStoreError. Edge: a missing database file is created; a second open of the same path reuses it. Invariant: opening then re-opening a path yields a working handle over the same data. Done-check: the open-and-reopen criterion.

### T-02.08  Put a fact
id: T-02.08
phase: 2
status: done
depends_on: [T-02.07]
stack: rust
criteria:
  - C1: `zira_memory::FactStore::put(&self, key: &str, value: &str) -> Result<(), zira_memory::FactStoreError>` commits a `key -> value` entry to the redb store durably.
  - C2: a test puts a fact, opens a fresh `FactStore` over the same path, and (via the get primitive's underlying read) confirms the committed value is present after the write transaction returns `Ok`.
not_doing:
  - No batch puts or transactions spanning multiple keys.
  - No value typing beyond `&str` — facts are stored as text.
test_files: [tests/fact_store_put.rs]
criteria_map:
  C1: [c1_put_returns_ok, c1_put_overwrite_returns_ok]
  C2: [c2_put_persists_across_reopen]
attempts: 1
last_failure: ""
---
The semantic write primitive. Inputs: a key and value. Outputs: a durably committed entry. Edge: putting an existing key overwrites it. Invariant: a put that returns `Ok` is persisted past the transaction. Done-check: the persisted-after-commit criterion.

### T-02.09  Get a fact
id: T-02.09
phase: 2
status: done
depends_on: [T-02.08]
stack: rust
criteria:
  - C1: `zira_memory::FactStore::get(&self, key: &str) -> Result<Option<String>, zira_memory::FactStoreError>` returns `Ok(Some(value))` for a stored key and `Ok(None)` for an absent key (a miss is not an error).
  - C2: a test puts `"a" -> "1"`, asserts `get("a")` returns `Ok(Some("1".into()))` and `get("absent")` returns `Ok(None)`.
not_doing:
  - No prefix or range scans — single-key lookup only.
  - No caching layer.
test_files: [tests/fact_store_get.rs]
criteria_map:
  C1: [test_get_hit_returns_ok_some, test_get_miss_returns_ok_none]
  C2: [test_get_hit_and_miss]
attempts: 1
last_failure: ""
---
The semantic read primitive. Inputs: a key. Outputs: the stored value or None. Edge: a missing key is `Ok(None)`, never an error variant. Invariant: get reflects the latest put for a key. Done-check: the hit-and-miss criterion.

### T-02.10  Delete a fact
id: T-02.10
phase: 2
status: done
depends_on: [T-02.09]
stack: rust
criteria:
  - C1: `zira_memory::FactStore::delete(&self, key: &str) -> Result<(), zira_memory::FactStoreError>` removes the entry for `key`; deleting an absent key is `Ok(())` (idempotent).
  - C2: a test puts a fact, deletes it, and asserts a subsequent `get` of that key returns `Ok(None)`; a second test asserts deleting an absent key returns `Ok(())`.
not_doing:
  - No bulk or prefix deletes.
  - No tombstone or soft-delete semantics.
test_files: [tests/fact_store_delete.rs]
criteria_map:
  C1: [test_delete_present_returns_ok, test_delete_absent_returns_ok]
  C2: [test_delete_then_get_returns_none, test_delete_absent_is_idempotent]
attempts: 1
last_failure: ""
---
The semantic removal primitive, completing the fact-store CRUD. Inputs: a key. Outputs: the entry removed. Edge: deleting an absent key is a no-op success. Invariant: after delete, get of that key is None. Done-check: the delete-then-miss and idempotent-delete criteria.

### T-02.11  Define the embedder trait
id: T-02.11
phase: 2
status: done
depends_on: [T-02.01]
stack: rust
criteria:
  - C1: `zira_memory::Embedder` is a trait with a method `embed(&self, text: &str) -> Vec<f32>` and an associated/accessor `dim(&self) -> usize` giving the embedding dimensionality.
  - C2: a unit test defines a trivial in-test implementor and asserts `embed` returns a vec whose length equals `dim()`.
not_doing:
  - No concrete embedder here — the hash test-embedder is T-02.12 and the real model is the blocked task.
  - No async — embedding is a synchronous CPU call.
test_files: [tests/embedder_trait.rs]
criteria_map:
  C1: [test_embed_len_matches_dim, test_embedder_trait_object_safe]
  C2: [test_embed_len_matches_dim, test_embedder_trait_object_safe]
attempts: 1
last_failure: ""
---
The embedding seam that decouples retrieval from the model, mirroring zira-core's stage-trait pattern. Inputs: text. Outputs: a fixed-dimension f32 vector. Invariant: every `embed` result has length `dim()`. Done-check: the length-matches-dim criterion against a test implementor.

### T-02.12  Implement the hash embedder
id: T-02.12
phase: 2
status: done
depends_on: [T-02.11]
stack: rust
criteria:
  - C1: `zira_memory::HashEmbedder` implements `zira_memory::Embedder`; its `embed` is deterministic and pure-Rust — the same input text always yields the same vector — and `dim()` returns its configured dimension.
  - C2: a test asserts `embed` of the same text twice yields equal vectors, and that two different texts yield different vectors.
  - C3: a test asserts every `embed` output vector has length equal to `dim()`.
not_doing:
  - No semantic quality claims — this is a deterministic stand-in for gateable tests, not the real model.
  - No external assets or downloads.
test_files: [tests/hash_embedder.rs]
criteria_map:
  C1: [test_hash_embedder_dim_matches_configured, test_hash_embedder_as_trait_object]
  C2: [test_hash_embedder_deterministic_and_distinct]
  C3: [test_hash_embedder_dim_matches_configured, test_hash_embedder_embed_len_matches_dim]
attempts: 2
last_failure: ""
---
A deterministic CPU/hash embedder so retrieval is fully gateable without the real model weights. Inputs: text and a fixed dimension. Outputs: a reproducible f32 vector. Edge: empty text still produces a dim-length vector. Invariant: same input maps to the same vector, distinct inputs differ. Done-check: the determinism, distinctness, and dim-length criteria.

### T-02.13  Compute the cosine similarity
id: T-02.13
phase: 2
status: done
depends_on: [T-02.01]
stack: rust
criteria:
  - C1: `zira_memory::cosine_similarity(a: &[f32], b: &[f32]) -> f32` returns the cosine similarity of two equal-length vectors; identical normalized vectors return ~1.0 and orthogonal vectors return ~0.0 (within a small epsilon).
  - C2: a test asserts `cosine_similarity` of a vector with itself is ~1.0, of two orthogonal vectors is ~0.0, and of opposite vectors is ~-1.0, each within epsilon.
  - C3: a test asserts a zero vector yields 0.0 rather than a NaN (the divide-by-zero guard).
not_doing:
  - No mismatched-length handling beyond a documented precondition; callers pass equal-length vectors.
  - No SIMD or perf tuning.
test_files: [tests/cosine_similarity.rs]
criteria_map:
  C1: [test_cosine_similarity_self_is_one, test_cosine_similarity_orthogonal_is_zero]
  C2: [test_cosine_similarity_self_is_one, test_cosine_similarity_orthogonal_is_zero, test_cosine_similarity_opposite_is_neg_one]
  C3: [test_cosine_similarity_zero_vector_guard]
attempts: 6
last_failure: ""
---
The vector-math kernel of the index. Inputs: two equal-length f32 slices. Outputs: a similarity in [-1, 1]. Edge: a zero-magnitude vector yields 0.0, never NaN. Invariant: self-similarity is 1.0, opposite is -1.0. Done-check: the identity/orthogonal/opposite and zero-guard criteria.

### T-02.14  Add a vector
id: T-02.14
phase: 2
status: done
depends_on: [T-02.13]
stack: rust
criteria:
  - C1: `zira_memory::VectorIndex::new() -> zira_memory::VectorIndex` builds an empty index and `add(&mut self, id: usize, vector: Vec<f32>)` stores the `(id, vector)` pair; `len(&self) -> usize` returns the count of stored vectors.
  - C2: a test builds an index, adds three vectors with distinct ids, and asserts `len()` is 3.
not_doing:
  - No removal or update of stored vectors.
  - No on-disk persistence — the index is rebuilt from the store/episodes.
test_files: [tests/vector_index.rs]
criteria_map:
  C1: [c1_new_builds_empty_index, c1_add_one_vector_len_is_one]
  C2: [c2_add_three_vectors_len_is_three]
attempts: 2
last_failure: ""
---
The pure-Rust vector index's insertion primitive. Inputs: an id and its vector. Outputs: a growing in-memory index. Edge: adding to an empty index yields len 1. Invariant: len equals the number of add calls with distinct ids. Done-check: the add-three-then-len criterion.

### T-02.15  Search the top-k vectors
id: T-02.15
phase: 2
status: done
depends_on: [T-02.14]
stack: rust
criteria:
  - C1: `zira_memory::VectorIndex::search(&self, query: &[f32], k: usize) -> Vec<(usize, f32)>` returns up to `k` `(id, score)` pairs sorted by descending cosine similarity to `query`.
  - C2: a test adds several vectors, searches with a query nearest one known id, and asserts that id is the first result and that results are in non-increasing score order.
  - C3: a test asserts `search` with `k` greater than the index size returns all stored vectors (length saturates at `len()`), and `k` of 0 returns an empty vec.
not_doing:
  - No approximate-NN tuning — exact brute-force search over the stored vectors is sufficient at this scale.
  - No filtering by a score threshold.
test_files: [tests/search_top_k.rs]
criteria_map:
  C1: [c1_search_respects_k_limit, c1_search_returns_id_score_pairs]
  C2: [c2_nearest_id_is_first, c2_results_in_non_increasing_score_order]
  C3: [c3_k_over_capacity_returns_all, c3_k_zero_returns_empty]
attempts: 1
last_failure: ""
---
The retrieval kernel over the index, built on cosine_similarity. Inputs: a query vector and k. Outputs: the top-k nearest ids with scores, descending. Edge: k over capacity returns all; k of 0 returns empty. Invariant: results are sorted by descending similarity and the true nearest is first. Done-check: the nearest-first, ordering, and k-bounds criteria.

### T-02.16  Retrieve the relevant episodes
id: T-02.16
phase: 2
status: done
depends_on: [T-02.04, T-02.12, T-02.15]
stack: rust
criteria:
  - C1: `zira_memory::retrieve(path: &std::path::Path, embedder: &impl zira_memory::Embedder, query: &str, k: usize) -> std::io::Result<Vec<zira_memory::Episode>>` loads the episodes, embeds each plus the query via `embedder`, and returns the top-k episodes by cosine similarity to the query.
  - C2: a test writes episodes whose texts are clearly near and far from a query, uses `HashEmbedder`, and asserts `retrieve(..., k=1)` returns the episode whose text is identical to the query (its own embedding is the nearest).
  - C3: a test asserts `retrieve` over a missing/empty episode file returns `Ok(vec![])`.
not_doing:
  - No fact-store retrieval here — episodic retrieval only.
  - No persisted index — vectors are computed per call from the episodes.
test_files: [tests/retrieve_episodes.rs]
criteria_map:
  C1: [c1_retrieve_returns_top_k_by_similarity]
  C2: [c2_identical_text_episode_is_nearest]
  C3: [c3_missing_file_returns_empty_vec, c3_empty_file_returns_empty_vec]
attempts: 2
last_failure: ""
---
The retrieval stage tying episodes, the embedder, and the index together. Inputs: a path, an embedder, a query, and k. Outputs: the top-k most relevant episodes. Edge: no episodes yields an empty vec, not an error. Invariant: an episode identical to the query is its own nearest match. Done-check: the nearest-episode and empty-store criteria.

### T-02.17  Format the context preamble
id: T-02.17
phase: 2
status: done
depends_on: [T-02.02]
stack: rust
criteria:
  - C1: `zira_memory::format_preamble(episodes: &[zira_memory::Episode]) -> String` renders the retrieved episodes into a single prompt-preamble string; an empty slice returns an empty string (no preamble).
  - C2: a test asserts the preamble for two episodes contains both episodes' `text` substrings, and that an empty slice yields exactly an empty string.
not_doing:
  - No truncation to a token budget.
  - No fact-store entries in the preamble — episodes only.
test_files: [tests/format_preamble.rs]
criteria_map:
  C1: [c1_empty_slice_returns_empty_string, c1_nonempty_slice_returns_nonempty_string]
  C2: [c2_preamble_contains_both_episode_texts, c2_empty_slice_yields_exact_empty_string]
attempts: 1
last_failure: ""
---
The injection stage that turns retrieved episodes into context the bridge prepends to a turn. Inputs: the retrieved episodes. Outputs: a preamble string. Edge: an empty slice produces an empty string so no noise is injected. Invariant: every episode's text appears in a non-empty preamble. Done-check: the contains-both and empty-yields-empty criteria.

### T-02.18  Consolidate the episodes
id: T-02.18
phase: 2
status: done
depends_on: [T-02.04, T-02.08]
stack: rust
criteria:
  - C1: `zira_memory::consolidate(episode_path: &std::path::Path, store: &zira_memory::FactStore) -> Result<usize, zira_memory::FactStoreError>` runs a stateless pass that derives deduplicated facts from the episodes and `put`s each into `store`, returning the count of facts written.
  - C2: a test writes episodes containing a duplicated piece of information, runs `consolidate`, and asserts the duplicate is collapsed to a single fact (the returned count and a follow-up `get` confirm dedup).
  - C3: a test asserts `consolidate` over an empty episode file writes zero facts and returns `Ok(0)`.
not_doing:
  - No LLM-driven summarization — the consolidation rule is deterministic and gateable (no model call).
  - No pruning of the episodic log here — capping is T-02.05.
test_files: [tests/consolidate_episodes.rs]
criteria_map:
  C1: [c1_consolidate_signature_and_count]
  C2: [c2_consolidate_deduplicates_repeated_text]
  C3: [c3_consolidate_empty_file_returns_zero, c3_consolidate_missing_file_returns_zero]
attempts: 3
last_failure: ""
---
The stateless consolidation pass distilling episodic logs into semantic facts, re-derivable from disk on every run. Inputs: the episode path and an open fact store. Outputs: deduplicated facts written to the store plus a written-count. Edge: an empty log writes nothing and returns 0. Invariant: duplicated information collapses to one fact; the pass holds no state between runs. Done-check: the dedup and empty-log criteria.

### T-02.19  Load the embedding model
id: T-02.19
phase: 2
status: done
depends_on: [T-02.11]
stack: rust
criteria:
  - C1: `zira_memory::CandleEmbedder::load(model_dir: &std::path::Path) -> Result<CandleEmbedder, EmbedderError>` loads a BERT sentence-embedding model (`config.json` + `tokenizer.json` + `model.safetensors`) from `model_dir` on the CPU via candle-transformers, and `CandleEmbedder` implements `zira_memory::Embedder`.
  - C2: `dim()` returns the model hidden size and `embed(text)` tokenizes the text, runs the model, mean-pools the last hidden state over the sequence, and returns a `Vec<f32>` of length `dim()`.
  - C3: a repo-root integration test `tests/candle_embedder.rs` loads the on-disk model (directory from `ZIRA_EMBED_MODEL`, defaulting to `~/.cache/zira/models/all-MiniLM-L6-v2`; it returns early when that directory is absent so a model-less CI stays green), embeds two distinct sentences, and asserts each vector has length `dim()`, is non-zero, and the two vectors differ — proving real weights, not the hash stand-in.
  - C4: `zira_memory::EmbedderError` implements `std::error::Error` + `Display` with distinct variants for a missing model file, a tokenizer-load failure, and a model-weights load failure; a unit test exercises every variant's `Display`.
  - C5: `embed` returns the un-normalized mean-pooled vector (it must NOT L2-normalize), and `tests/candle_embedder.rs` asserts each embedding's L2 norm lies within `0.5..=30.0`; a correct mean-pool gives a norm of roughly 5–8, but a magnitude error in the pooling (e.g. multiplying by the sequence length instead of dividing) scales the norm by about seq² to well over 30, so this bound pins the pooling arithmetic and kills the div-to-mul mutant.
not_doing:
  - No GPU/CUDA path — CPU only.
  - No quantization or model conversion — the safetensors asset is consumed as provided.
  - No in-code network download — the model is placed on disk out-of-band.
test_files: [tests/candle_embedder.rs]
criteria_map:
  C1: [c1_c2_c3_c5_real_model_embeds]
  C2: [c1_c2_c3_c5_real_model_embeds]
  C3: [c1_c2_c3_c5_real_model_embeds]
  C4: [c4_embedder_error_display_variants]
  C5: [c1_c2_c3_c5_real_model_embeds]
attempts: 1
last_failure: ""
---
PROVEN RECIPE (a spike compiled + ran this against the real model; reproduce it). Deps are already in `crates/zira-memory/Cargo.toml`: candle-core/candle-nn/candle-transformers 0.8, tokenizers 0.21, serde_json. LOAD: parse `config.json` into `candle_transformers::models::bert::Config` via serde_json; `tokenizers::Tokenizer::from_file(dir.join("tokenizer.json"))`; `let vb = unsafe { candle_nn::VarBuilder::from_mmaped_safetensors(&[dir.join("model.safetensors")], candle_transformers::models::bert::DTYPE, &candle_core::Device::Cpu)? }`; `BertModel::load(vb, &config)`. EMBED(text): `enc = tok.encode(text, true)`; `token_ids` = a `candle_core::Tensor` of `enc.get_ids()` unsqueezed to shape (1, seq); `type_ids = token_ids.zeros_like()`; `attn` = a Tensor of `enc.get_attention_mask()` unsqueezed; `out = model.forward(&token_ids, &type_ids, Some(&attn))` gives (1, seq, hidden); mean-pool `(out.sum(1)? / seq as f64)?` then `.squeeze(0)?.to_vec1::<f32>()?`. `dim()` = config.hidden_size (384 for this model). Map every candle/tokenizer/io failure to an `EmbedderError` variant and exercise each variant's Display in C4's test (the T-01.10 lesson). The 87MB weights live OUTSIDE the repo at $ZIRA_EMBED_MODEL — never commit them. Verified spike output: dim=384, two distinct sentences cosine ~0.55. Done-check: the four criteria.

MUTATION-CRITICAL (added after a surviving div-to-mul mutant at the pooling line): do NOT L2-normalize the output — return the raw mean-pooled vector (measured norm ~5–8 for this model). Normalizing would make the `/ seq` an equivalent, unkillable mutant; leaving it raw lets C5's `0.5..=30.0` norm bound kill a div→mul scale error (which blows the norm to ~60+).

### T-03.01  Define the expression preset
id: T-03.01
phase: 3
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_avatar::ExpressionPreset` is a struct of named blendshape weights `{ joy: f32, sorrow: f32, anger: f32, surprise: f32, fun: f32 }` deriving `Debug` + `Clone` + `PartialEq`, with a `ExpressionPreset::neutral()` constructor whose every weight is `0.0`.
  - C2: `ExpressionPreset` exposes a `clamped(&self) -> ExpressionPreset` that returns a copy with every weight constrained to the inclusive range `0.0..=1.0`, leaving already-in-range values unchanged.
not_doing:
  - No emotion mapping here (that is the next task).
  - No GPU/VRM blendshape application — this is a pure data struct only.
test_files: [tests/expression_preset.rs]
criteria_map:
  C1: [c1_struct_fields, c1_neutral_all_zeros, c1_neutral_debug, c1_neutral_clone, c1_neutral_partial_eq]
  C2: [c2_clamped_in_range_unchanged, c2_clamped_above_one, c2_clamped_below_zero, c2_clamped_mixed, c2_clamped_neutral_is_noop]
attempts: 1
last_failure: ""
---
The pure data carrier for a VRM expression: a fixed set of named blendshape weights that the emotion map fills and the avatar state holds. Inputs: weight values. Outputs: a clamped, comparable preset with a zeroed neutral baseline. Edge: out-of-range weights are clamped, never rejected. Invariant: weights are always reportable within [0,1] via `clamped`. Done-check: the two criteria.

### T-03.02  Map emotion to expression
id: T-03.02
phase: 3
status: done
depends_on: [T-00.05, T-03.01]
stack: rust
criteria:
  - C1: `zira_avatar::expression_for(e: Emotion) -> ExpressionPreset` is total over all ten `Emotion` variants and returns an `ExpressionPreset`; a test calls it for each of the ten variants and asserts every returned preset is already self-equal to its own `clamped()` (all weights within `0.0..=1.0`).
  - C2: `expression_for(Emotion::Neutral)` equals `ExpressionPreset::neutral()` (every weight `0.0`).
  - C3: at least two distinct emotions map to distinct presets (e.g. `expression_for(Emotion::Happy) != expression_for(Emotion::Sad)`), pinning that the table is not a constant.
not_doing:
  - No prosody mapping — that is `zira-emotion`, already done.
  - No blending/interpolation between presets here.
test_files: [tests/emotion_expression.rs]
criteria_map:
  C1: [c1_expression_for_total_and_bounded]
  C2: [c2_neutral_maps_to_neutral_preset]
  C3: [c3_distinct_emotions_map_to_distinct_presets]
attempts: 1
last_failure: ""
---
The avatar-facing twin of the prosody table: the single total function from the fixed emotion vocabulary to a blendshape preset. Inputs: an emotion. Outputs: a bounded `ExpressionPreset`. Edge: unknown emotions cannot occur — the type is closed over ten variants. Invariant: total, bounded, and not collapsed to one constant. Done-check: the three criteria.

### T-03.03  Define the viseme vocabulary
id: T-03.03
phase: 3
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_avatar::Viseme` is an enum with the mouth-shape variants `Sil, A, I, U, E, O` deriving `Debug` + `Clone` + `Copy` + `PartialEq`, with `Default` returning `Viseme::Sil`.
  - C2: `zira_avatar::Viseme::as_label(self) -> &'static str` returns the lowercase shape name (`"sil"`, `"a"`, `"i"`, `"u"`, `"e"`, `"o"`), and a test asserts every variant's label is non-empty and distinct.
not_doing:
  - No phoneme/char selection here (the next task).
  - No timing — frames are ordered in a later task.
test_files: [tests/viseme_type.rs]
criteria_map:
  C1: [c1_variants_exist, c1_derives_debug, c1_derives_clone, c1_derives_copy, c1_derives_partial_eq, c1_default_is_sil]
  C2: [c2_sil_label, c2_a_label, c2_i_label, c2_u_label, c2_e_label, c2_o_label, c2_all_labels_nonempty_and_distinct]
attempts: 1
last_failure: ""
---
The closed alphabet of mouth shapes the lip-sync drives, mirroring the A/I/U/E/O blendshapes plus silence. Inputs: a variant. Outputs: a comparable enum with a stable label. Edge: silence is the default rest shape. Invariant: these six shapes are the only visemes. Done-check: the two criteria.

### T-03.04  Select the viseme
id: T-03.04
phase: 3
status: done
depends_on: [T-03.03]
stack: rust
criteria:
  - C1: `zira_avatar::viseme_for_char(c: char) -> Viseme` maps each vowel character to its mouth shape (`a`->`A`, `e`->`E`, `i`->`I`, `o`->`O`, `u`->`U`, case-insensitive), pinned per vowel by the test.
  - C2: a non-vowel character (e.g. a consonant, digit, or whitespace) maps to `Viseme::Sil`.
not_doing:
  - No full grapheme/IPA phoneme analysis — a coarse vowel-to-shape pick only.
  - No weighting — weight assignment belongs to the frame builder.
test_files: [tests/viseme_for_char.rs]
criteria_map:
  C1: [c1_lowercase_a_maps_to_a, c1_lowercase_e_maps_to_e, c1_lowercase_i_maps_to_i, c1_lowercase_o_maps_to_o, c1_lowercase_u_maps_to_u, c1_uppercase_a_maps_to_a, c1_uppercase_e_maps_to_e, c1_uppercase_i_maps_to_i, c1_uppercase_o_maps_to_o, c1_uppercase_u_maps_to_u]
  C2: [c2_consonant_maps_to_sil, c2_digit_maps_to_sil, c2_whitespace_maps_to_sil]
attempts: 1
last_failure: ""
---
The character-to-mouth-shape rule the frame builder calls per character. Inputs: one `char`. Outputs: a `Viseme`. Edge: anything that is not a recognised vowel rests at `Sil`. Invariant: total over all `char` values, never panics. Done-check: the two criteria.

### T-03.05  Clamp the viseme weight
id: T-03.05
phase: 3
status: done
depends_on: [T-00.07]
stack: rust
criteria:
  - C1: `zira_avatar::clamp_weight(w: f32) -> f32` returns `w` constrained to the inclusive range `0.0..=1.0`: a value below `0.0` returns `0.0`, a value above `1.0` returns `1.0`, and an in-range value returns unchanged.
  - C2: a `f32::NAN` input returns `0.0` (the rest weight) rather than propagating `NaN`, pinned by the test.
not_doing:
  - No frame construction here — this is the scalar clamp the builder reuses.
  - No interpolation between weights.
test_files: [tests/clamp_weight.rs]
criteria_map:
  C1: [c1_below_zero_returns_zero, c1_above_one_returns_one, c1_in_range_returns_unchanged, c1_exactly_zero_returns_zero, c1_exactly_one_returns_one]
  C2: [c2_nan_returns_zero]
attempts: 1
last_failure: ""
---
The single scalar guard that keeps every lip-sync weight inside the renderable range. Inputs: a raw `f32`. Outputs: a weight in [0,1]. Edge: `NaN` collapses to the rest weight so a bad amplitude never poisons a frame. Invariant: the result is always a finite value within [0,1]. Done-check: the two criteria.

### T-03.06  Order the viseme frames
id: T-03.06
phase: 3
status: done
depends_on: [T-00.07, T-03.05]
stack: rust
criteria:
  - C1: `zira_avatar::timed_frames(frames: &[VisemeFrame], frame_ms: u32) -> Vec<(u32, VisemeFrame)>` returns one entry per input frame, in input order, pairing each with a monotonically increasing start time in milliseconds (`0`, `frame_ms`, `2*frame_ms`, ...).
  - C2: every returned `VisemeFrame.weight` equals `zira_avatar::clamp_weight` applied to the corresponding input weight (out-of-range input weights are clamped into [0,1]).
  - C3: an empty input slice returns an empty `Vec`.
not_doing:
  - No real-clock scheduling — start times are computed, not awaited.
  - No audio alignment — `frame_ms` is the supplied cadence.
test_files: [tests/timed_frames.rs]
criteria_map:
  C1: [c1_single_frame_time_is_zero, c1_three_frames_monotonic_times, c1_count_matches_input, c1_viseme_order_preserved]
  C2: [c2_weight_above_one_clamped, c2_weight_below_zero_clamped, c2_in_range_weight_unchanged, c2_nan_weight_collapsed]
  C3: [c3_empty_input_returns_empty]
attempts: 1
last_failure: ""
---
Turns an unordered-cadence viseme stream into a deterministic timeline of (start_ms, frame) pairs with clamped weights, ready for either GPU blendshapes or the 2D fallback. Inputs: a viseme-frame slice + a per-frame duration. Outputs: an ordered, timed, weight-clamped sequence. Edge: an empty stream yields an empty timeline. Invariant: input order and count are preserved; times are strictly increasing. Done-check: the three criteria.

### T-03.07  Define the avatar state
id: T-03.07
phase: 3
status: done
depends_on: [T-03.02, T-03.03]
stack: rust
criteria:
  - C1: `zira_avatar::AvatarState` is a struct `{ expression: ExpressionPreset, mouth: Viseme }` deriving `Debug` + `Clone` + `PartialEq`, with an `AvatarState::resting()` constructor equal to `{ expression: ExpressionPreset::neutral(), mouth: Viseme::Sil }`.
  - C2: `AvatarState::for_emotion(e: Emotion) -> AvatarState` builds a state whose `expression` equals `expression_for(e)` and whose `mouth` is `Viseme::Sil`.
not_doing:
  - No GPU/2D rendering — this is the renderer-agnostic state the sink emits.
  - No transition logic — the sink owns advancing this state.
test_files: [tests/avatar_state.rs]
criteria_map:
  C1: [c1_struct_fields, c1_derives_debug, c1_derives_clone, c1_derives_partial_eq, c1_resting_expression, c1_resting_mouth, c1_resting_full]
  C2: [c2_for_emotion_expression_tracks_emotion, c2_for_emotion_mouth_is_sil, c2_for_emotion_neutral, c2_for_emotion_all_variants]
attempts: 1
last_failure: ""
---
The renderer-agnostic snapshot both the 3D avatar and the 2D fallback consume: the active expression preset plus the current mouth shape. Inputs: an emotion (and later a viseme). Outputs: a comparable state with a defined resting value. Edge: a fresh state rests with a neutral face and a closed mouth. Invariant: the expression always tracks the emotion map. Done-check: the two criteria.

### T-03.08  Describe the fallback frame
id: T-03.08
phase: 3
status: done
depends_on: [T-03.07]
stack: rust
criteria:
  - C1: `zira_avatar::FallbackFrame` is a struct `{ sprite: String, mouth: Viseme }` deriving `Debug` + `Clone` + `PartialEq`, and `zira_avatar::fallback_frame(state: &AvatarState) -> FallbackFrame` returns a frame whose `mouth` equals `state.mouth`.
  - C2: `fallback_frame` selects the `sprite` name from the dominant expression weight in `state.expression` (the highest-weighted blendshape names the sprite, e.g. all-zero -> `"neutral"`, joy-dominant -> `"happy"`), pinned for the neutral case and one non-neutral case by the test.
not_doing:
  - No image loading or drawing — this is a pure description of which 2D sprite + mouth is active.
  - No GPU — the fallback is explicitly the non-GPU path.
test_files: [tests/fallback_frame.rs]
criteria_map:
  C1: [c1_fallback_frame_derives_debug, c1_fallback_frame_derives_clone, c1_fallback_frame_derives_partial_eq, c1_mouth_passthrough_sil, c1_mouth_passthrough_non_sil]
  C2: [c2_neutral_expression_yields_neutral_sprite, c2_joy_dominant_yields_happy_sprite]
attempts: 1
last_failure: ""
---
The pure 2D-fallback projection of an `AvatarState`: which static face sprite and mouth shape a GPU-less box should show. Inputs: an avatar state. Outputs: a sprite name + mouth shape. Edge: an all-zero expression names the neutral sprite. Invariant: the mouth shape passes through unchanged; the sprite is chosen deterministically from the dominant weight. Done-check: the two criteria.

### T-03.09  Select the renderer
id: T-03.09
phase: 3
status: done
depends_on: [T-00.09, T-03.08]
stack: rust
criteria:
  - C1: `zira_avatar::RendererKind` is an enum `{ Vrm, Fallback2d }` deriving `Debug` + `Clone` + `Copy` + `PartialEq`, and `zira_avatar::select_renderer(cfg: &AvatarConfig) -> RendererKind` returns `RendererKind::Fallback2d` when `cfg.vrm_path` is `None`.
  - C2: `select_renderer` returns `RendererKind::Vrm` when `cfg.vrm_path` is `Some(path)` with a non-empty path string.
  - C3: `cfg.vrm_path` of `Some("")` (an empty path) is treated as absent and yields `RendererKind::Fallback2d`, pinned by the test.
not_doing:
  - No GPU capability probe here — the choice is config-driven (a VRM path requests the 3D renderer; its absence selects 2D).
  - No actual window/render-loop start — that is a blocked task.
test_files: [tests/select_renderer.rs]
criteria_map:
  C1: [c1_renderer_kind_derives_debug, c1_renderer_kind_derives_clone, c1_renderer_kind_derives_copy, c1_renderer_kind_derives_partial_eq, c1_select_renderer_none_gives_fallback2d]
  C2: [c2_select_renderer_some_nonempty_gives_vrm]
  C3: [c3_select_renderer_some_empty_gives_fallback2d]
attempts: 1
last_failure: ""
---
The deterministic, GPU-free decision of which renderer the avatar should run: the 3D VRM path when a model is configured, the 2D fallback otherwise. Inputs: the avatar config. Outputs: a renderer kind. Edge: a `Some("")` empty path counts as no model and falls back to 2D. Invariant: a missing or empty `vrm_path` always selects the GPU-less fallback. Done-check: the three criteria.

### T-03.10  Drive the avatar sink
id: T-03.10
phase: 3
status: done
depends_on: [T-00.20, T-03.06, T-03.07]
stack: rust
criteria:
  - C1: `zira_avatar::AvatarDriver` holds an `AvatarState` (initially `AvatarState::resting()`) and exposes `apply_emotion(&mut self, e: Emotion)` (setting `expression` from `expression_for`) and `apply_viseme(&mut self, v: Viseme)` (setting `mouth`), with a read-only `state(&self) -> &AvatarState` accessor.
  - C2: `AvatarDriver::on_emotion(&mut self, e: Emotion) -> Event` applies the emotion and returns `Event::ExpressionChange`; a test asserts the returned variant is `Event::ExpressionChange` and that `state().expression` now equals `expression_for(e)`.
  - C3: a repo-root integration test `tests/avatar_driver.rs` feeds an emotion then a sequence of visemes and asserts the driver's `state()` tracks the latest emotion's expression and the latest viseme's mouth shape in order.
not_doing:
  - No GPU/window — the driver produces pure STATE + an `ExpressionChange` event; rendering it is a blocked task.
  - No async `AvatarSink` trait impl over hardware — this is the renderer-agnostic logic the mock/real sinks share.
test_files: [tests/avatar_driver.rs]
criteria_map:
  C1: [c1_new_driver_starts_resting, c1_apply_emotion_sets_expression, c1_apply_viseme_sets_mouth, c1_state_returns_ref]
  C2: [c2_on_emotion_returns_expression_change, c2_on_emotion_updates_expression]
  C3: [c3_emotion_then_viseme_sequence]
attempts: 1
last_failure: ""
---
The pure state machine behind the avatar: given an emotion and a viseme stream it advances an `AvatarState` and emits `Event::ExpressionChange`, with no GPU, window, or model. Inputs: emotions + visemes. Outputs: an updated `AvatarState` + an `ExpressionChange` event. Edge: a fresh driver rests neutral and silent. Invariant: state changes only through `apply_emotion`/`apply_viseme`. Done-check: the three criteria.

### T-03.11  Type the avatar errors
id: T-03.11
phase: 3
status: done
depends_on: [T-03.09]
stack: rust
criteria:
  - C1: `zira_avatar::AvatarError` is an enum implementing `std::error::Error` + `Display` with distinct variants for a missing VRM path when the VRM renderer was selected, an unreadable/absent model file, and an unsupported viseme label.
  - C2: a unit test exercises the `Display` text of every variant, asserting each is non-empty, names its failure, and that all variant messages are distinct (no arm is an unexercised mutation survivor).
not_doing:
  - No recovery or retry policy — these are reported, not handled.
  - No GPU/device errors here — those belong to the blocked render-loop task.
test_files: [tests/avatar_errors.rs]
criteria_map:
  C1: [c1_avatar_error_missing_vrm_path_variant_exists, c1_avatar_error_model_unreadable_variant_exists, c1_avatar_error_unsupported_viseme_variant_exists, c1_avatar_error_implements_error_trait, c1_avatar_error_implements_display]
  C2: [c2_display_missing_vrm_path_nonempty_names_failure, c2_display_model_unreadable_nonempty_names_failure, c2_display_unsupported_viseme_nonempty_names_failure, c2_display_all_variants_produce_distinct_messages]
attempts: 2
last_failure: ""
---
The avatar subsystem's typed failure surface for the gateable path, with every `Display` arm exercised so none survives mutation (the T-01.10 lesson). Inputs: a failed precondition. Outputs: a distinct, named error. Edge: each failure maps to its own variant. Invariant: every variant's message is non-empty and unique. Done-check: the two criteria.

### T-03.12  Render the VRM avatar
id: T-03.12
phase: 3
status: blocked
depends_on: [T-03.09, T-03.10]
stack: rust
criteria:
  - C1: a Bevy/wgpu application starts a 30fps render loop on an integrated GPU, loads the configured `.vrm` model, applies each `AvatarState`'s `ExpressionPreset` to the model's blendshapes, and drives the mouth blendshape from the live `Viseme`/`VisemeFrame` stream, with idle blink/breathing motion.
not_doing:
  - The pure expression/viseme/state logic (T-03.01..T-03.10) is already gateable and must not be reimplemented here.
  - No 2D fallback path here — that is the gateable `FallbackFrame`.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: needs an integrated GPU + display/windowing (Bevy/wgpu render loop) and a `.vrm` model on disk; cannot run in the headless frozen-test harness; tracked, not attempted by the loop."
---
The real embodied avatar: a Bevy/wgpu render loop that loads the VRM model and applies the (already-gateable) expression presets and viseme stream as blendshapes at 30fps, with idle life. Blocked-on-human: requires an integrated GPU, a display/windowing stack, and a `.vrm` asset — none available in the headless deterministic harness. Done-check: the one criterion, measured on target hardware with a real model.

### T-04.01  Define the SkillManifest type
id: T-04.01
phase: 4
status: done
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_skills::SkillManifest` is a struct with public fields `name: String`, `version: String`, `entry: String`, `capabilities: Vec<String>`, and `allowed_roots: Vec<String>`, deriving `Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize`.
  - C2: A repo-root integration test `tests/skill_manifest_type.rs` constructs a `SkillManifest` with all five fields populated and asserts each field reads back the stored value.
  - C3: `SkillManifest` round-trips through `serde_json` (serialize then deserialize) to an equal value, pinned by the test.
not_doing:
  - No TOML/JSON parsing-from-text helper here (that is the next task).
  - No signing, scanning, or gate logic here — type definition only.
test_files: [tests/skill_manifest_type.rs]
criteria_map:
  C1: [c1_skill_manifest_fields_and_derives]
  C2: [c2_skill_manifest_field_readback, c2_skill_manifest_empty_vecs_are_legal]
  C3: [c3_skill_manifest_serde_json_round_trip]
attempts: 2
last_failure: ""
---
The data record every later safety check reads. Inputs: the five field values. Outputs: a serde-stable manifest struct. Edge: an empty capabilities or allowed_roots vec is legal here (default-deny is enforced downstream, not at construction). Invariant: the struct is the single source for a skill's declared name, version, entry, capabilities, and path roots. Done-check: the three criteria, including the serde_json round-trip.

### T-04.02  Parse the manifest
id: T-04.02
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::parse_manifest_toml(text: &str) -> Result<SkillManifest, ManifestError>` deserializes a well-formed TOML manifest into a `SkillManifest`, pinned by a test on a valid fixture.
  - C2: `zira_skills::parse_manifest_json(text: &str) -> Result<SkillManifest, ManifestError>` deserializes a well-formed JSON manifest into the same `SkillManifest`, and a test asserts the TOML and JSON forms of one manifest parse to equal values.
  - C3: Malformed input (invalid TOML and invalid JSON) returns `Err(ManifestError::Parse(..))`, not a panic, pinned by the test for both formats.
not_doing:
  - No reading from disk — both parsers take an in-memory `&str`.
  - No semantic validation of capabilities here (that is the gate task).
test_files: [tests/parse_manifest.rs]
criteria_map:
  C1: [c1_parse_manifest_toml_valid_fixture]
  C2: [c2_parse_manifest_json_valid_fixture, c2_toml_and_json_parse_to_equal_values]
  C3: [c3_malformed_toml_returns_parse_error, c3_malformed_json_returns_parse_error]
attempts: 1
last_failure: ""
---
Turns serialized manifest text into the typed record. Inputs: a TOML or JSON `&str`. Outputs: a `SkillManifest` or a typed `ManifestError`. Edge: malformed text is a recoverable `Parse` error, never a panic; the two formats must agree on a shared fixture. Invariant: parsing is total over arbitrary input. Done-check: valid-parse for both formats, format-equality, and malformed-rejection for both.

### T-04.03  Define the ManifestError type
id: T-04.03
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::ManifestError` is a `thiserror`-derived enum with variants `Parse(String)`, `MissingField(String)`, and `Io(String)`, each carrying context.
  - C2: A repo-root integration test `tests/manifest_error_type.rs` formats each of the three variants via `Display` and asserts each message is non-empty and distinct from the others, exercising every variant's `Display`.
  - C3: `ManifestError` implements `std::error::Error` (asserted by binding a constructed value to `&dyn std::error::Error`).
not_doing:
  - No new error variants beyond the three the manifest paths require.
  - No conversion `From` impls beyond what `thiserror` derives.
test_files: [tests/manifest_error_type.rs]
criteria_map:
  C1: [c1_manifest_error_all_three_variants_exist, c1_manifest_error_is_thiserror_derived]
  C2: [c2_display_messages_are_non_empty, c2_display_messages_are_distinct]
  C3: [c3_manifest_error_implements_std_error]
attempts: 1
last_failure: ""
---
The typed failure surface for manifest handling. Inputs: a constructed variant. Outputs: a distinct, non-empty `Display` string per variant and an `Error` impl. Edge: every variant's `Display` must be exercised so no format operator survives mutation. Invariant: each manifest failure maps to exactly one variant. Done-check: per-variant Display distinctness plus the `Error` trait object binding.

### T-04.04  Define the Signature type
id: T-04.04
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::Signature` is a struct wrapping the raw HMAC bytes (`Vec<u8>`) with a `Signature::to_hex(&self) -> String` accessor and a `Signature::from_hex(s: &str) -> Result<Signature, ManifestError>` constructor.
  - C2: A repo-root integration test `tests/signature_type.rs` round-trips a `Signature` through `to_hex` then `from_hex` to an equal value.
  - C3: `Signature::from_hex` on a non-hex string returns `Err`, pinned by the test, rather than panicking.
not_doing:
  - No HMAC computation here — this is the carrier type only.
  - No base64 form; hex is the single serialized representation.
test_files: [tests/signature_type.rs]
criteria_map:
  C1: [c1_signature_struct_exists_with_to_hex, c1_from_hex_returns_manifest_error_on_failure]
  C2: [c2_signature_hex_round_trip]
  C3: [c3_from_hex_non_hex_string_returns_err]
attempts: 1
last_failure: ""
---
The serialized form of an HMAC tag, shared by sign and verify. Inputs: raw bytes or a hex string. Outputs: a `Signature` plus a hex string. Edge: a non-hex string is a recoverable error. Invariant: `from_hex(to_hex(s)) == s` for any signature. Done-check: the hex round-trip and the non-hex rejection.

### T-04.05  Sign the manifest
id: T-04.05
phase: 4
status: done
depends_on: [T-04.01, T-04.04]
stack: rust
criteria:
  - C1: `zira_skills::sign_manifest(key: &[u8], m: &SkillManifest) -> Signature` computes an HMAC-SHA256 over a deterministic byte serialization of `m` using the real `hmac` + `sha2` crates.
  - C2: A repo-root integration test `tests/sign_manifest.rs` asserts signing the same manifest with the same key twice yields equal signatures (determinism).
  - C3: The test asserts that signing the same manifest with two different keys yields different signatures, and that mutating one manifest field changes the signature.
not_doing:
  - No verification here (the next task).
  - No key management/derivation — the caller supplies the key bytes.
test_files: [tests/sign_manifest.rs]
criteria_map:
  C1: [test_sign_manifest_is_deterministic, test_sign_manifest_output_is_hmac_sha256_length]
  C2: [test_sign_manifest_is_deterministic]
  C3: [test_sign_manifest_key_sensitivity, test_sign_manifest_content_sensitivity]
attempts: 1
last_failure: ""
---
Produces the authenticity tag over a manifest. Inputs: a key and a manifest. Outputs: a deterministic HMAC-SHA256 `Signature`. Edge: distinct keys and distinct manifest contents must produce distinct signatures so the verify path has signal. Invariant: the byte serialization fed to HMAC is stable across runs. Done-check: same-key determinism, key-sensitivity, and content-sensitivity.

### T-04.06  Verify the signature
id: T-04.06
phase: 4
status: done
depends_on: [T-04.04, T-04.05]
stack: rust
criteria:
  - C1: `zira_skills::verify_manifest(key: &[u8], m: &SkillManifest, sig: &Signature) -> bool` returns `true` for a signature produced by `sign_manifest` with the same key and manifest (the ACCEPT path), pinned by the test.
  - C2: A repo-root integration test `tests/verify_manifest.rs` asserts verification returns `false` when the signature bytes are tampered (a flipped byte) — the tampered-signature REJECT path.
  - C3: The test asserts verification returns `false` when the manifest is altered after signing and when a different key is used — both REJECT paths.
not_doing:
  - No timing-attack hardening beyond using the `hmac` crate's constant-time `verify`.
  - No signature storage format concerns (covered by the Signature type).
test_files: [tests/verify_manifest.rs]
criteria_map:
  C1: [test_verify_manifest_accept]
  C2: [test_verify_manifest_reject_tampered_sig]
  C3: [test_verify_manifest_reject_altered_manifest, test_verify_manifest_reject_wrong_key]
attempts: 1
last_failure: ""
---
The gatekeeper that a manifest is authentic and untampered. Inputs: a key, a manifest, and a candidate signature. Outputs: a boolean accept/reject. Edge: BOTH the valid-signature accept and the tampered-signature/altered-manifest/wrong-key rejects are pinned so no branch survives mutation. Invariant: verify accepts iff the signature equals a fresh sign over the same key and manifest. Done-check: one positive and three negative criteria.

### T-04.07  Scan for injection
id: T-04.07
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::scan_injection(text: &str) -> Vec<Finding>` greps the text against a fixed table of prompt-injection substring patterns (e.g. "ignore previous instructions", "disregard the constitution", "reveal your system prompt") case-insensitively and returns one `Finding` per match.
  - C2: A repo-root integration test `tests/scan_injection.rs` feeds a planted-bad string containing a known injection phrase and asserts the returned `Vec<Finding>` is non-empty and names the matched pattern.
  - C3: The test feeds a clean skill description and asserts `scan_injection` returns an empty `Vec` (the clean case), pinning both the hit and no-hit branches.
not_doing:
  - No ML/embedding-based detection — substring patterns only, like Ratchet's checker.
  - No mutation of the scanned text; the scanner is read-only and reports.
test_files: [tests/scan_injection.rs]
criteria_map:
  C1: [test_scan_injection_detects_known_phrase, test_scan_injection_case_insensitive, test_scan_injection_clean_text_returns_empty]
  C2: [test_scan_injection_detects_known_phrase, test_scan_injection_detects_disregard_constitution, test_scan_injection_detects_reveal_system_prompt]
  C3: [test_scan_injection_clean_text_returns_empty]
attempts: 1
last_failure: ""
---
Zira's prompt-injection grep over externally-sourced skill text. Inputs: a `&str` of skill prompt/description. Outputs: a `Vec<Finding>`, one per matched danger pattern. Edge: a planted-bad string yields findings; a clean string yields an empty vec. Invariant: the pattern table is the single source of injection signatures and matching is case-insensitive. Done-check: the planted-bad and clean cases both pinned.

### T-04.08  Construct a finding
id: T-04.08
phase: 4
status: done
depends_on: [T-04.07]
stack: rust
criteria:
  - C1: `zira_skills::Finding::new(pattern: impl Into<String>) -> Finding` builds a `Finding` whose `pattern` field equals the argument, asserted by readback in `tests/finding_type.rs`.
  - C2: `Finding` implements `std::fmt::Display`, rendering a non-empty string that contains the finding's `pattern`, exercised by the test.
  - C3: two `Finding`s built from the same pattern compare equal and two from different patterns compare unequal, exercising the `PartialEq` derive.
not_doing:
  - No severity scoring — a finding stays a flat record keyed by its matched pattern.
  - No change to the `scan_injection` danger table or its return type.
test_files: [tests/finding_type.rs]
criteria_map:
  C1: [test_finding_new_stores_pattern, test_finding_new_accepts_owned_string]
  C2: [test_finding_display_contains_pattern, test_finding_display_names_pattern_exactly]
  C3: [test_finding_eq_same_pattern, test_finding_neq_different_pattern]
attempts: 1
last_failure: ""
---
BUILD NOTE: `Finding` already exists from T-04.07 (its consumer was authored first — an ordering inversion), so this task does NOT redefine the struct; it ADDS the ergonomic `Finding::new` constructor and a `Display` impl that `tests/finding_type.rs` exercises. RED therefore fails cleanly on the missing `new`/`Display` symbols, not on a struct field. Inputs: a matched pattern. Outputs: a constructed, printable finding. Invariant: Display always names the pattern (C2 pins it so no format arm survives mutation). Done-check: the three criteria.

### T-04.09  Gate capabilities against the constitution
id: T-04.09
phase: 4
status: done
depends_on: [T-00.12, T-04.10]
stack: rust
criteria:
  - C1: `zira_skills::gate_capabilities(c: &zira_config::Constitution, m: &SkillManifest) -> GateDecision` returns `GateDecision::Allow` when every declared capability is sanctioned by the constitution rules, pinned by `tests/gate_capabilities.rs` (the ALLOW path).
  - C2: a manifest declaring a constitution-forbidden capability returns `GateDecision::Deny { capability, .. }` naming the offending capability — the DENY path.
  - C3: a manifest declaring an unknown capability matched by no rule returns `GateDecision::Deny { .. }` (default-deny), never `Allow`.
not_doing:
  - No mutation of the constitution — it is read-only via `rules()`.
  - No path/sandbox checks here (the capability sandbox is a separate task).
test_files: [tests/gate_capabilities.rs]
criteria_map:
  C1: [test_allow_when_all_capabilities_sanctioned]
  C2: [test_deny_names_forbidden_capability]
  C3: [test_deny_unknown_capability_default_deny]
attempts: 1
last_failure: ""
---
BUILD NOTE: `GateDecision` is the canonical TWO-field type from T-04.10 (`Deny { capability: String, reason: String }`); tests MUST destructure it as `Deny { capability, .. }`. This task was reset because its original frozen test (written before T-04.10 existed) assumed a one-field `Deny` and was edited afterwards — an ordering inversion now corrected by depending on T-04.10. The `gate_capabilities` function was removed so RED fails cleanly on the missing function, not on a struct field. Default-deny: a capability is sanctioned only when a non-prohibitive constitution rule names it. Done-check: one allow + two deny criteria.

### T-04.10  Define the GateDecision type
id: T-04.10
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::GateDecision` is an enum with variants `Allow` and `Deny { capability: String, reason: String }`, deriving `Debug, Clone, PartialEq`, and exposes an `is_allowed(&self) -> bool` accessor.
  - C2: A repo-root integration test `tests/gate_decision_type.rs` asserts `Allow.is_allowed()` is `true` and a `Deny { .. }` value's `is_allowed()` is `false`, and that a `Deny` carries the offending capability and reason.
  - C3: The test formats both variants via a `Display` impl and asserts each message is non-empty and distinct, exercising both variants' `Display`.
not_doing:
  - No multi-finding aggregation — a decision reports the first denial.
  - No serde derive required on the decision.
test_files: [tests/gate_decision_type.rs]
criteria_map:
  C1: [test_allow_is_allowed, test_deny_is_not_allowed, test_clone_and_partial_eq]
  C2: [test_allow_is_allowed, test_deny_is_not_allowed, test_deny_carries_capability_and_reason]
  C3: [test_display_non_empty_and_distinct, test_deny_display_contains_fields]
attempts: 1
last_failure: ""
---
The verdict type the constitution gate returns. Inputs: a constructed variant. Outputs: an `is_allowed` boolean plus a distinct `Display` per variant. Edge: every variant's `Display` is exercised so no format operator survives mutation. Invariant: `is_allowed` is true iff `Allow`. Done-check: the boolean per-variant plus the distinct-Display check.

### T-04.11  Check a path against the sandbox
id: T-04.11
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::path_allowed(m: &SkillManifest, candidate: &std::path::Path) -> bool` returns `true` iff `candidate`, after normalization, lies under at least one of the manifest's declared `allowed_roots` — pinned by a test where the candidate is inside a declared root (the ALLOW path).
  - C2: A repo-root integration test `tests/path_sandbox.rs` asserts a candidate outside every declared root returns `false` — the DENY path.
  - C3: The test asserts a traversal escape (e.g. a declared root joined with `../` climbing above it) returns `false`, so `..` cannot smuggle a path out of the sandbox.
not_doing:
  - No filesystem access — the check is purely lexical over normalized path components.
  - No symlink resolution (declared as out of scope; lexical containment only).
test_files: [tests/path_sandbox.rs]
criteria_map:
  C1: [c1_path_inside_declared_root_is_allowed, c1_path_equal_to_declared_root_is_allowed, c1_path_under_second_of_two_roots_is_allowed]
  C2: [c2_path_outside_all_roots_is_denied, c2_path_with_matching_prefix_but_not_child_is_denied, c2_empty_roots_denies_any_candidate]
  C3: [c3_traversal_escape_is_denied, c3_traversal_escape_at_root_boundary_is_denied]
attempts: 1
last_failure: ""
---
The capability sandbox restricting a skill to its declared path roots. Inputs: a manifest and a candidate path. Outputs: a boolean allowed/denied. Edge: an in-root path is allowed; an out-of-root path AND a `../` traversal escape are both denied. Invariant: a path is allowed only if it lexically resolves under a declared root with no upward escape. Done-check: the in-root allow plus the out-of-root and traversal denies.

### T-04.12  Define the audit entry
id: T-04.12
phase: 4
status: done
depends_on: [T-04.04]
stack: rust
criteria:
  - C1: `zira_skills::AuditEntry` is a struct with public fields `skill_name: String`, `action: String`, `prev_hash: String`, and `entry_hash: String`, deriving `Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize`.
  - C2: A repo-root integration test `tests/audit_entry_type.rs` constructs an `AuditEntry`, asserts each field reads back, and round-trips it through `serde_json` to an equal value.
  - C3: `zira_skills::compute_entry_hash(key: &[u8], skill_name: &str, action: &str, prev_hash: &str) -> String` is deterministic (same inputs yield the same hash) and changes when any input changes, pinned by the test.
not_doing:
  - No chain append/verify logic here (the next two tasks).
  - No on-disk persistence of entries.
test_files: [tests/audit_entry_type.rs]
criteria_map:
  C1: [c1_audit_entry_struct_fields, c1_audit_entry_derives_debug_clone_partialeq, c1_audit_entry_derives_serialize_deserialize]
  C2: [c2_construct_and_read_fields, c2_serde_json_round_trip]
  C3: [c3_hash_deterministic, c3_hash_is_nonempty_hex, c3_hash_sensitive_to_skill_name, c3_hash_sensitive_to_action, c3_hash_sensitive_to_prev_hash, c3_hash_sensitive_to_key]
attempts: 1
last_failure: ""
---
The link record of the HMAC audit chain. Inputs: the entry fields, plus key/name/action/prev for the hash helper. Outputs: a serde-stable entry and a deterministic, input-sensitive `entry_hash`. Edge: changing any hashed input changes the hash so tampering is detectable downstream. Invariant: an entry binds its content to the previous hash. Done-check: field read-back, serde round-trip, and hash determinism/sensitivity.

### T-04.13  Append an audit entry
id: T-04.13
phase: 4
status: done
depends_on: [T-04.12]
stack: rust
criteria:
  - C1: `zira_skills::append_audit(key: &[u8], chain: &[AuditEntry], skill_name: &str, action: &str) -> AuditEntry` produces a new entry whose `prev_hash` equals the last entry's `entry_hash` (or a fixed genesis constant when the chain is empty), pinned by the test.
  - C2: A repo-root integration test `tests/append_audit.rs` appends to an empty chain and asserts the first entry's `prev_hash` equals the genesis constant.
  - C3: The test appends two entries in sequence and asserts the second entry's `prev_hash` equals the first entry's `entry_hash`, so the chain links.
not_doing:
  - No tamper detection here (the verify task).
  - No I/O — the chain is an in-memory slice.
test_files: [tests/append_audit.rs]
criteria_map:
  C1: [c1_genesis_constant_is_64_lowercase_hex_zeros, c1_append_to_nonempty_chain_links_to_last_entry_hash]
  C2: [c2_empty_chain_prev_hash_equals_genesis]
  C3: [c3_two_entries_second_prev_hash_equals_first_entry_hash]
attempts: 1
last_failure: ""
---
Extends the audit chain by one HMAC-linked entry. Inputs: the key, the existing chain, and the skill name + action. Outputs: a new `AuditEntry` chained to the prior hash. Edge: an empty chain links to a fixed genesis constant rather than panicking. Invariant: each appended entry's `prev_hash` is the predecessor's `entry_hash`. Done-check: the genesis link and the two-entry chaining check.

### T-04.14  Verify the chain
id: T-04.14
phase: 4
status: done
depends_on: [T-04.13]
stack: rust
criteria:
  - C1: `zira_skills::verify_chain(key: &[u8], chain: &[AuditEntry]) -> bool` returns `true` for a chain built entirely by `append_audit` — the intact-chain ACCEPT path, pinned by the test.
  - C2: A repo-root integration test `tests/verify_chain.rs` mutates one entry's `action` field after the chain is built and asserts `verify_chain` returns `false` — the tampered-content REJECT path.
  - C3: The test asserts that removing/reordering an entry (breaking a `prev_hash` link) returns `false`, the broken-link REJECT path.
not_doing:
  - No automatic repair — verify reports a tampered chain, it does not fix it.
  - No I/O; verification runs over an in-memory slice.
test_files: [tests/verify_chain.rs]
criteria_map:
  C1: [c1_intact_chain_built_by_append_audit_returns_true]
  C2: [c2_tampered_action_field_returns_false]
  C3: [c3_removed_entry_breaks_link_returns_false, c3_reordered_entries_break_link_returns_false]
attempts: 1
last_failure: ""
---
Proves the audit log was not tampered with. Inputs: the key and a chain slice. Outputs: a boolean intact/broken. Edge: BOTH an intact chain (accept) and a content-tampered AND a link-broken chain (reject) are pinned so no branch survives mutation. Invariant: a chain verifies iff every entry's recomputed hash matches and each `prev_hash` equals its predecessor's `entry_hash`. Done-check: one accept and two reject criteria.

### T-04.15  Register a skill
id: T-04.15
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::SkillRegistry` supports `register(&mut self, m: SkillManifest)`, `lookup(&self, name: &str) -> Option<&SkillManifest>`, `list(&self) -> Vec<&SkillManifest>`, and `remove(&mut self, name: &str) -> bool`; a test registers a manifest and asserts `lookup` returns it and `list` includes it.
  - C2: A repo-root integration test `tests/skill_registry.rs` asserts `remove` of a registered name returns `true` and a subsequent `lookup` returns `None`, while `remove` of an absent name returns `false`.
  - C3: The test asserts registering a second manifest with an existing name replaces (not duplicates) the entry, keeping `list` length at one for that name.
not_doing:
  - No persistence to disk — an in-memory registry only.
  - No gate invocation inside register; the gate is applied by the caller before registration.
test_files: [tests/skill_registry.rs]
criteria_map:
  C1: [c1_register_then_lookup_returns_manifest, c1_register_then_list_includes_manifest]
  C2: [c2_remove_present_returns_true_and_clears_lookup, c2_remove_absent_returns_false]
  C3: [c3_reregister_same_name_replaces_not_duplicates]
attempts: 1
last_failure: ""
---
The in-memory catalog of admitted skills. Inputs: manifests by name. Outputs: register/list/lookup/remove over them. Edge: removing an absent name is a benign `false`; re-registering a name replaces rather than duplicates. Invariant: a name maps to at most one manifest. Done-check: register+lookup+list, remove semantics for present and absent names, and the replace-on-duplicate-name check.

### T-04.16  Scaffold the MCP config
id: T-04.16
phase: 4
status: done
depends_on: [T-04.01]
stack: rust
criteria:
  - C1: `zira_skills::mcp_config_from_manifest(m: &SkillManifest) -> serde_json::Value` produces an `.mcp.json`-shaped object whose `mcpServers` table contains an entry keyed by the manifest `name` with `command` set from the manifest `entry`.
  - C2: A repo-root integration test `tests/mcp_factory.rs` calls the factory on a manifest and asserts the generated JSON parses, contains the `mcpServers` key, and the server entry's name and command match the manifest.
  - C3: The test asserts the generated config serializes to a string and re-parses to an equal `serde_json::Value` (a stable, valid MCP skeleton).
not_doing:
  - No spawning or running of the MCP server — generation of the config skeleton only.
  - No writing the file to disk; the factory returns the JSON value.
test_files: [tests/mcp_factory.rs]
criteria_map:
  C1: [c1_mcpservers_key_and_command_from_manifest]
  C2: [c2_factory_json_parses_and_keys_match_manifest]
  C3: [c3_generated_config_round_trips]
attempts: 1
last_failure: ""
---
The MCP factory that turns an admitted skill manifest into an MCP server config skeleton. Inputs: a `SkillManifest`. Outputs: an `.mcp.json`-shaped `serde_json::Value`. Edge: the generated config must round-trip through string serialization unchanged so it is a valid, stable skeleton. Invariant: the server is keyed by the manifest name with its command taken from the manifest entry. Done-check: the shape/key checks plus the serialize-reparse equality.

### T-05.01  Define the plan decision
id: T-05.01
phase: 5
status: done
depends_on: [T-00.08]
stack: rust
criteria:
  - C1: `zira_core::PlanDecision` is an enum with exactly the two variants `Accept` and `Reject`, deriving `Debug` + `Clone` + `Copy` + `PartialEq`.
  - C2: a repo-root integration test `tests/plan_decision.rs` constructs both variants and asserts `PlanDecision::Accept != PlanDecision::Reject` (the two are distinguishable).
not_doing:
  - No transition logic here — that is the next task.
  - No serde derive — the decision is an in-process value, never persisted.
test_files: [tests/plan_decision.rs]
criteria_map:
  C1: [test_plan_decision_derives]
  C2: [test_plan_decision_distinguishable]
attempts: 1
last_failure: ""
---
The user's verdict on a narrated plan. Inputs: a caller in PlanReview UX. Outputs: a tiny two-variant `Copy` enum `PlanDecision{Accept,Reject}` in the `zira-core` LIBRARY crate (never a lib.rs on the `zira` binary). Edge: only two outcomes — there is no third 'defer'. Invariant: the decision type is the single vocabulary plan-review logic switches on. Done-check: the two criteria — both variants exist and compare unequal.

### T-05.02  Map a plan decision to an event
id: T-05.02
phase: 5
status: done
depends_on: [T-05.01]
stack: rust
criteria:
  - C1: `zira_core::review_plan(&PlanSummary, PlanDecision) -> Event` returns `Event::TurnStarted` for `PlanDecision::Accept`, pinned by `tests/review_plan.rs`.
  - C2: `zira_core::review_plan` returns an `Event::Error(_)` for `PlanDecision::Reject`, pinned by the same test.
  - C3: `tests/review_plan.rs` asserts the returned event is independent of `PlanSummary` content (the same decision over two different `PlanSummary` values yields the same `Event` variant).
not_doing:
  - No state mutation here — `review_plan` is a pure function returning the event the caller feeds to the bus.
  - No re-implementation of the transition table — that lives in `next_state`.
test_files: [tests/review_plan.rs]
criteria_map:
  C1: [accept_returns_turn_started]
  C2: [reject_returns_error]
  C3: [decision_independent_of_plan_content]
attempts: 1
last_failure: ""
---
Pure plan-review logic over the existing state machine. Inputs: the narrated `PlanSummary` and a `PlanDecision`. Outputs: `Accept -> Event::TurnStarted` (which drives PlanReview->Thinking) and `Reject -> Event::Error(..)` (which drives PlanReview->Idle), in the `zira-core` library crate. Edge: the plan body never changes the mapping — only the decision does. Invariant: `review_plan` is side-effect-free and total over the two decisions. Done-check: the three criteria.

### T-05.03  Verify the plan-review transition
id: T-05.03
phase: 5
status: done
depends_on: [T-05.02]
stack: rust
criteria:
  - C1: `tests/plan_review_transition.rs` asserts `next_state(State::PlanReview, &review_plan(&plan, PlanDecision::Accept)) == Some(State::Thinking)`.
  - C2: the same test asserts `next_state(State::PlanReview, &review_plan(&plan, PlanDecision::Reject)) == Some(State::Idle)`.
not_doing:
  - No new transition rows — this reuses the frozen `next_state` table from Phase 0.
  - No orchestrator run-loop changes.
test_files: [tests/plan_review_transition.rs]
criteria_map:
  C1: [test_accept_lands_in_thinking]
  C2: [test_reject_lands_in_idle]
attempts: 3
last_failure: ""
---
Wires the decision mapping to the real state machine end-to-end. Inputs: `State::PlanReview` plus the `Event` from `review_plan`. Outputs: an Accept lands in `Thinking` and a Reject lands in `Idle`, proving `review_plan`'s event choice is the correct key into `next_state`. Edge: any other base state is out of scope. Invariant: plan-review never invents a transition the table does not already define. Done-check: the two criteria.

### T-05.04  Resolve the default emotion
id: T-05.04
phase: 5
status: done
depends_on: [T-00.05, T-00.09]
stack: rust
criteria:
  - C1: `zira_config::resolve_default_emotion(&EmotionConfig) -> zira_proto::Emotion` maps a known tag (e.g. `"happy"`, case-insensitively) to the matching `Emotion` variant, pinned by `tests/default_emotion.rs`.
  - C2: `zira_config::resolve_default_emotion` maps an empty or unknown `default_emotion` string to `Emotion::Neutral`, pinned by the same test.
not_doing:
  - No new emotion variants — the vocabulary stays the ten in `zira_proto::Emotion`.
  - No mutation of the config — the resolver borrows it read-only.
test_files: [tests/default_emotion.rs]
criteria_map:
  C1: [c1_known_tag_maps_to_matching_emotion_variant, c1_known_tag_is_case_insensitive]
  C2: [c2_empty_string_maps_to_neutral, c2_unknown_tag_maps_to_neutral]
attempts: 1
last_failure: ""
---
Turns the configured `EmotionConfig.default_emotion` string into a typed `Emotion`. Inputs: an `&EmotionConfig`. Outputs: the resolved `Emotion`, delegating to the existing `Emotion::from_tag` case-insensitive mapping, in the `zira-config` library crate. Edge: empty and unknown tags both fall back to `Neutral`, never an error. Invariant: the configured default is always a valid in-vocabulary `Emotion`. Done-check: the two criteria.

### T-05.05  Define the vocabulary error
id: T-05.05
phase: 5
status: done
depends_on: [T-00.05]
stack: rust
criteria:
  - C1: `zira_config::VocabError` is a `thiserror` enum with at least the variant `UnknownTag { tag: String }`, deriving `Debug`.
  - C2: `tests/vocab_error.rs` formats every `VocabError` variant via `Display` (`to_string()`) and asserts the message contains the offending `tag`.
not_doing:
  - No validation logic here — that is the next task; this defines only the error type its failures use.
  - No `panic!`-based reporting — failures are typed `Result` errors.
test_files: [tests/vocab_error.rs]
criteria_map:
  C1: [c1_unknown_tag_variant_exists_and_derives_debug]
  C2: [c2_display_contains_offending_tag]
attempts: 1
last_failure: ""
---
The typed failure for emotion-vocabulary review. Inputs: an offending tag string. Outputs: a `VocabError` enum (in `zira-config`) whose `UnknownTag` variant carries the rejected tag. Edge: the `Display` of EVERY variant is exercised by a test criterion so no message operator survives mutation. Invariant: an unknown tag is a typed error, never a silent coercion in this strict path. Done-check: the two criteria — the variant exists and its Display is asserted to name the tag.

### T-05.06  Validate the emotion vocabulary
id: T-05.06
phase: 5
status: done
depends_on: [T-05.05]
stack: rust
criteria:
  - C1: `zira_config::validate_vocab(&[String]) -> Result<Vec<zira_proto::Emotion>, VocabError>` returns `Ok` with one resolved `Emotion` per input tag when every tag (case-insensitively) names one of the ten variants, pinned by `tests/validate_vocab.rs`.
  - C2: `validate_vocab` returns `Err(VocabError::UnknownTag { tag })` naming the first tag that matches no variant, pinned by the same test.
  - C3: `tests/validate_vocab.rs` asserts an empty slice yields `Ok` of an empty `Vec` (the empty vocabulary is valid).
not_doing:
  - No normalization of casing in the OUTPUT — output is the typed `Emotion`, not a re-cased string.
  - No fallback-to-Neutral here — this STRICT path rejects unknown tags (unlike `from_tag`).
test_files: [tests/validate_vocab.rs]
criteria_map:
  C1: [c1_all_ten_variants_resolve, c1_case_insensitive]
  C2: [c2_first_unknown_tag_rejected, c2_stops_at_first_unknown]
  C3: [c3_empty_slice_is_ok]
attempts: 1
last_failure: ""
---
Validates and normalizes a configured tag list against the ten `Emotion` variants. Inputs: a slice of tag strings. Outputs: the resolved `Emotion` vector on success, or `VocabError::UnknownTag` for the first unrecognized tag. Edge: empty input is valid; matching is case-insensitive; unknown is a hard error (distinct from the lenient `from_tag`). Invariant: a returned `Ok` vector contains only in-vocabulary emotions. Done-check: the three criteria.

### T-05.07  Detect the first run
id: T-05.07
phase: 5
status: done
depends_on: [T-00.10]
stack: rust
criteria:
  - C1: `zira_config::is_first_run(&std::path::Path) -> bool` returns `true` when no file exists at the given config path and `false` when one exists, pinned by `tests/first_run_detect.rs` using a temp path.
  - C2: `tests/first_run_detect.rs` asserts the function does not create, modify, or delete anything at the path (detection is read-only).
not_doing:
  - No directory or file creation here — detection only; creation is the next task.
  - No reliance on the live XDG home — the test passes an explicit temp path.
test_files: [tests/first_run_detect.rs]
criteria_map:
  C1: [c1_absent_path_returns_true, c1_present_file_returns_false, c1_present_but_empty_file_returns_false]
  C2: [c2_detection_does_not_create_missing_path, c2_detection_does_not_modify_existing_file, c2_detection_does_not_delete_existing_file]
attempts: 1
last_failure: ""
---
Decides whether first-run setup is needed. Inputs: a config-file path. Outputs: a `bool` — `true` exactly when the config file is absent, in the `zira-config` library crate. Edge: a present-but-empty file counts as 'exists' (not first run); detection mutates nothing. Invariant: calling the detector is always safe and side-effect-free. Done-check: the two criteria.

### T-05.08  Write the default config
id: T-05.08
phase: 5
status: done
depends_on: [T-05.07]
stack: rust
criteria:
  - C1: `zira_config::write_default_config(&std::path::Path) -> Result<(), ConfigError>` creates the parent directories and writes a TOML file that `zira_config::load_from` re-reads into a value equal to `ZiraConfig::default()`, pinned by `tests/write_default_config.rs` over a temp path.
  - C2: `tests/write_default_config.rs` asserts a second call succeeds and leaves the loaded config unchanged (the write is idempotent).
not_doing:
  - No interactive prompting — the default file is written non-interactively.
  - No overwrite of a user-edited config beyond re-emitting the serializable defaults.
test_files: [tests/write_default_config.rs]
criteria_map:
  C1: [c1_creates_parent_dirs_and_round_trips_default, c1_file_exists_after_write, c1_loaded_config_equals_default]
  C2: [c2_second_call_succeeds, c2_second_call_leaves_config_unchanged]
attempts: 1
last_failure: ""
---
Performs first-run setup by materializing a default config. Inputs: the target config path. Outputs: parent dirs created and a default `config.toml` written such that `load_from` round-trips it to `ZiraConfig::default()`, reusing the existing `ConfigError`. Edge: running twice is safe (idempotent); a path that cannot be created surfaces a typed `ConfigError`. Invariant: after setup, the config path loads to a complete, valid config. Done-check: the two criteria.

### T-05.09  Define the budget error
id: T-05.09
phase: 5
status: done
depends_on: [T-00.09]
stack: rust
criteria:
  - C1: `zira_config::BudgetError` is a `thiserror` enum with the variants `EpisodesTooHigh { value: usize, max: usize }` and `EpisodesZero`, deriving `Debug`.
  - C2: `tests/budget_error.rs` formats BOTH `BudgetError` variants via `Display` (`to_string()`) and asserts each message names its variant's distinguishing data (the over-limit `value` for one, and the zero condition for the other).
not_doing:
  - No bounds-checking logic here — that is the next task; this defines only the error.
  - No reuse of `ConfigError` — the budget audit owns a distinct typed error.
test_files: [tests/budget_error.rs]
criteria_map:
  C1: [c1_episodes_too_high_variant_exists_and_derives_debug, c1_episodes_zero_variant_exists_and_derives_debug]
  C2: [c2_display_episodes_too_high_contains_value, c2_display_episodes_zero_indicates_zero_condition]
attempts: 1
last_failure: ""
---
The typed failure for the resource-budget audit. Inputs: an out-of-bounds budget value. Outputs: a `BudgetError` enum (in `zira-config`) distinguishing 'too high' from 'zero'. Edge: EVERY variant's `Display` is exercised by a test criterion so no message survives mutation. Invariant: a budget violation is a typed error, never a silent clamp. Done-check: the two criteria — both variants exist and both Displays are asserted.

### T-05.10  Audit the memory budget
id: T-05.10
phase: 5
status: done
depends_on: [T-05.09]
stack: rust
criteria:
  - C1: `zira_config::audit_memory_budget(&MemoryConfig, usize) -> Result<(), BudgetError>` returns `Ok` when `max_episodes` is non-zero and at most the supplied ceiling, pinned by `tests/audit_budget.rs`.
  - C2: `audit_memory_budget` returns `Err(BudgetError::EpisodesTooHigh { value, max })` when `max_episodes` exceeds the ceiling, pinned by the same test.
  - C3: `audit_memory_budget` returns `Err(BudgetError::EpisodesZero)` when `max_episodes` is zero, pinned by the same test.
not_doing:
  - No I/O — the audit is a pure check over the already-loaded config.
  - No auto-repair — the audit reports a violation, it does not rewrite the value.
test_files: [tests/audit_budget.rs]
criteria_map:
  C1: [c1_ok_when_episodes_at_ceiling, c1_ok_when_episodes_below_ceiling]
  C2: [c2_err_too_high_when_episodes_exceed_ceiling, c2_err_too_high_carries_value_and_max]
  C3: [c3_err_zero_when_episodes_is_zero]
attempts: 1
last_failure: ""
---
Checks a configured resource budget against sane bounds. Inputs: a `&MemoryConfig` and a ceiling. Outputs: `Ok(())` within bounds, else the matching `BudgetError`, in the `zira-config` library crate. Edge: zero episodes and over-ceiling are distinct typed failures; the boundary `value == max` is allowed. Invariant: a passing audit guarantees `0 < max_episodes <= ceiling`. Done-check: the three criteria, one per outcome.

### T-05.11  Expose the build version
id: T-05.11
phase: 5
status: done
depends_on: [T-00.01]
stack: rust
criteria:
  - C1: `zira_config::build_version() -> &'static str` returns the crate's `CARGO_PKG_VERSION` and is non-empty, pinned by `tests/build_version.rs`.
  - C2: `tests/build_version.rs` asserts the returned string parses as a dotted semver-shaped value (at least `major.minor.patch`, i.e. three dot-separated numeric components).
not_doing:
  - No git-hash embedding here — the release manifest task may add metadata; this exposes only the package version.
  - No runtime configurability — the version is compiled in.
test_files: [tests/build_version.rs]
criteria_map:
  C1: [build_version_is_non_empty, build_version_is_static_str]
  C2: [build_version_is_semver_shaped]
attempts: 1
last_failure: ""
---
Embeds the build version for packaging. Inputs: none (compile-time `CARGO_PKG_VERSION`). Outputs: a non-empty `&'static str` version, in the `zira-config` library crate. Edge: the string is always present and shaped `X.Y.Z`. Invariant: the reported version tracks the crate manifest with no drift. Done-check: the two criteria — non-empty and semver-shaped.

### T-05.12  Generate the install manifest
id: T-05.12
phase: 5
status: done
depends_on: [T-05.11]
stack: rust
criteria:
  - C1: `zira_config::install_manifest() -> String` returns a manifest string that contains the `build_version()` value, pinned by `tests/install_manifest.rs`.
  - C2: `tests/install_manifest.rs` asserts the manifest lists the application directory name `zira` (the install target Zira owns under the XDG bases).
not_doing:
  - No filesystem writes — the generator returns the manifest text; persisting it is a caller concern.
  - No package-format coupling — the manifest is plain text, not tied to one distro packager.
test_files: [tests/install_manifest.rs]
criteria_map:
  C1: [install_manifest_contains_build_version, install_manifest_is_non_empty]
  C2: [install_manifest_contains_app_dir_name]
attempts: 1
last_failure: ""
---
Produces a release/install manifest for packaging. Inputs: none (reads `build_version` and the owned app dir). Outputs: a `String` manifest naming the version and the `zira` install directory, in the `zira-config` library crate. Edge: the manifest is deterministic for a given build — same version in, same text out. Invariant: the manifest always reflects the real `build_version()`. Done-check: the two criteria.

### T-05.13  Tune the barge-in threshold
id: T-05.13
phase: 5
status: blocked
depends_on: [T-00.20]
stack: rust
criteria:
  - C1: the barge-in interrupt threshold (the speech-energy/latency margin that triggers `Event::BargeIn` while Zira is Speaking) is tuned so interruptions fire promptly without false triggers, measured against live microphone audio on target hardware.
not_doing:
  - The state-machine barge-in transitions themselves — those are pure and already defined in `next_state`.
  - Mock-driven barge-in — already covered by the Phase-0 orchestrator tests.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: barge-in threshold tuning needs real audio-input latency measured on target hardware; tracked, not attempted by the loop."
---
Real-world barge-in responsiveness. Inputs: live microphone audio during Speaking. Outputs: a tuned interrupt threshold that balances responsiveness against false triggers. Edge: too sensitive self-interrupts on TTS bleed, too dull ignores the user. Invariant: tuning never alters the frozen `next_state` table — only the detection margin feeding `Event::BargeIn`. Blocked-on-human: requires audio-latency measurement on target hardware. Done-check: the one criterion, measured on target hardware.

### T-05.14  Soak-test the runtime
id: T-05.14
phase: 5
status: blocked
depends_on: [T-00.20]
stack: rust
criteria:
  - C1: the full Zira runtime sustains a multi-hour soak run on target hardware without memory growth, deadlock, or state-machine wedging, exercising repeated Idle->...->Idle conversation cycles with the real voice stack.
not_doing:
  - Short mock-driven cycle tests — those exist from Phase 0 and run in the loop.
  - Micro-benchmarks of individual pure functions — out of scope for the soak test.
test_files: []
criteria_map: {}
attempts: 0
last_failure: "blocked-on-human: a long-running soak test must run on target hardware against the real audio/model stack; tracked, not attempted by the loop."
---
Long-running stability under real load. Inputs: a multi-hour live session on target hardware. Outputs: evidence of stable memory, no deadlocks, and a state machine that always returns to Idle. Edge: slow leaks or rare wedges only surface over hours. Invariant: the runtime is steady-state stable across many turns. Blocked-on-human: needs the real audio/model stack on target hardware over hours. Done-check: the one criterion, observed on target hardware.
