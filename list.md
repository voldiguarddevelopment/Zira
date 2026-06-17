# Zira — list.md (live state: status + frozen-test mapping)

The authoritative live document. `status` drives the loop; `criteria_map` + `test_files`
are filled by the RED phase and frozen.

### T-00.01  Scaffold the Cargo workspace
id: T-00.01
phase: 0
status: pending
depends_on: []
stack: rust
criteria:
  - C1: the root `Cargo.toml` declares BOTH a `[package]` and a `[workspace]` table with `resolver = "2"`, so `cargo test` at the root runs repo-root `tests/`; `cargo build` at the root exits 0.
  - C2: the workspace `members` list the ten member crates under `crates/`: zira, zira-core, zira-bridge, zira-voice, zira-emotion, zira-avatar, zira-memory, zira-skills, zira-config, zira-proto.
  - C3: `cargo test` exits 0 across the workspace with zero tests defined; `zira` is a binary target and the other nine are library targets.
not_doing:
  - No crate internals beyond an empty lib/bin target each.
  - No dependency wiring beyond what empty crates need to build.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The root surface every other task attaches to. Inputs: Cargo manifests only. Outputs: a compiling ten-crate workspace whose root is also a package (so repo-root `tests/` run) and a green empty test run. Errors/edges: a manifest that fails to parse is the only failure, surfaced by cargo. Invariant: the workspace compiles from here forward. Done-check: the three cargo-observable criteria.

### T-00.02  Declare the shared dependencies
id: T-00.02
phase: 0
status: pending
depends_on: [T-00.01]
stack: rust
criteria:
  - C1: the root `[workspace.dependencies]` table declares `tokio`, `serde`, `serde_json`, `thiserror`, `anyhow`, and `tracing` with pinned versions.
  - C2: at least one member crate consumes a shared dependency via `{ workspace = true }` and `cargo build` resolves it (proving the workspace-inheritance wiring works).
  - C3: `cargo metadata` exits 0 and the six shared deps appear exactly once in `[workspace.dependencies]` (no per-crate version drift for these).
not_doing:
  - No feature-flag tuning beyond what the crates need to compile.
  - No FFI / audio / GPU dependencies — those belong to later, blocked tasks.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The shared dependency surface. Inputs: the root manifest. Outputs: `[workspace.dependencies]` with the six core deps and a proven inheritance into a member. Errors/edges: a version that fails to resolve fails `cargo build`. Invariant: core deps are declared once at the root. Done-check: the three criteria.

### T-00.03  Configure the lint policy
id: T-00.03
phase: 0
status: pending
depends_on: [T-00.01]
stack: rust
criteria:
  - C1: a `rust-toolchain.toml` pins `channel = "stable"` and includes the `rustfmt` and `clippy` components.
  - C2: a `rustfmt.toml` exists and `cargo fmt --all --check` exits 0 on the scaffolded tree.
  - C3: the workspace configures clippy to deny warnings (a `[workspace.lints]` table or equivalent) and `cargo clippy --workspace` exits 0 on the scaffolded tree.
not_doing:
  - No custom lint authoring beyond enabling the standard rustfmt + clippy gates.
  - No CI changes — the GitHub Actions workflow is maintained outside the loop.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The style + lint floor. Inputs: the workspace root. Outputs: toolchain + fmt + clippy config that pass clean on the scaffold. Errors/edges: a malformed toml fails the respective tool. Invariant: fmt and clippy are green from here. Done-check: the three tool-observable criteria.

### T-00.04  Initialize structured logging
id: T-00.04
phase: 0
status: pending
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: the `zira` binary initializes `tracing-subscriber` with an `EnvFilter` honoring the `RUST_LOG` environment variable.
  - C2: a repo-root integration test `tests/logging_init.rs` asserts the init function is idempotent (a second call does not panic or double-install) and returns a typed result.
  - C3: with `RUST_LOG` unset the subscriber installs at a sane default level (info) rather than silent or trace.
not_doing:
  - No log routing to files or external sinks — stdout/stderr only.
  - No per-crate log configuration beyond the global env filter.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
Observability from first boot. Inputs: the `RUST_LOG` env var. Outputs: an installed tracing subscriber + an idempotent init. Errors/edges: a malformed filter falls back to the default level, never a panic. Invariant: logging is safe to initialize once. Done-check: the three criteria.

### T-00.05  Define the Emotion type
id: T-00.05
phase: 0
status: pending
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_proto::Emotion` is an enum with exactly the ten variants `Neutral, Happy, Sad, Angry, Excited, Calm, Curious, Concerned, Playful, Tired`, derives `Serialize`/`Deserialize`, and `Default` returns `Neutral`.
  - C2: a repo-root integration test `tests/emotion_type.rs` round-trips every variant through serde JSON and back to the same value.
  - C3: parsing an unknown or malformed tag string maps to `Emotion::Neutral` (case-insensitive match on the known names), pinned by the test.
not_doing:
  - No prosody or expression tables here (those are `zira-emotion`, a later task).
  - No streaming tag parser here (that is `zira-emotion`).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The fixed emotion vocabulary shared across TTS and the avatar. Inputs: a variant or a tag string. Outputs: a serde-stable enum defaulting to Neutral, with unknown->Neutral parsing. Errors/edges: an unknown name is Neutral, never an error. Invariant: the ten-name vocabulary is the single source. Done-check: the three criteria.

### T-00.06  Define the State type
id: T-00.06
phase: 0
status: pending
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_proto::State` is an enum with exactly `Idle, Listening, Transcribing, Thinking, PlanReview, Speaking`, derives `Serialize`/`Deserialize` + `Copy` + `PartialEq`, and `Default` returns `Idle`.
  - C2: a repo-root integration test `tests/state_type.rs` round-trips every variant through serde and asserts `State::default() == State::Idle`.
not_doing:
  - No transition logic here (that is the state machine in `zira-core`).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The conversation-state alphabet. Inputs: a variant. Outputs: a serde-stable, copyable enum defaulting to Idle. Errors/edges: none beyond serde. Invariant: these six states are the only states. Done-check: the two criteria.

### T-00.07  Define the payload types
id: T-00.07
phase: 0
status: pending
depends_on: [T-00.02, T-00.05]
stack: rust
criteria:
  - C1: `zira_proto` defines structs `Transcript`, `AudioChunk`, `Segment`, `VisemeFrame`, `PlanSummary`, and `Usage`, each deriving `Serialize`/`Deserialize` + `Clone`.
  - C2: `Segment` carries an `Emotion` and the spoken `text`, so an emotion-tagged segment is representable.
  - C3: a repo-root integration test `tests/payload_types.rs` round-trips a populated instance of each of the six structs through serde JSON unchanged.
not_doing:
  - No event wrapping here (the `Event` enum is the next task).
  - No audio decoding — `AudioChunk` is a typed PCM container only.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The data carried between stages. Inputs: stage-produced values. Outputs: six serde-stable payload structs, with `Segment` carrying an `Emotion`. Errors/edges: none beyond serde. Invariant: cross-stage data is typed, not ad-hoc maps. Done-check: the three criteria.

### T-00.08  Define the Event type
id: T-00.08
phase: 0
status: pending
depends_on: [T-00.06, T-00.07]
stack: rust
criteria:
  - C1: `zira_proto::Event` is an enum covering the runtime vocabulary: `WakeDetected, SpeechStarted, SpeechEnded, AudioChunk, TranscriptReady, TurnStarted, TextDelta, EmotionSegment, PlanReady, SpeakRequest, VisemeFrame, ExpressionChange, BargeIn, TurnComplete, Error`.
  - C2: the payload-bearing variants carry the matching `zira_proto` payload types (`TranscriptReady(Transcript)`, `EmotionSegment(Segment)`, `VisemeFrame(VisemeFrame)`, `TurnComplete(Usage)`, `Error(String)`), and `Event` derives `Clone` + `Serialize`/`Deserialize`.
  - C3: a repo-root integration test `tests/event_type.rs` round-trips a representative payload-bearing variant and a unit variant through serde unchanged.
not_doing:
  - No bus or dispatch here (that is `zira-core`).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The single message type on the bus. Inputs: a stage emitting an event. Outputs: a serde-stable enum whose payload variants wrap the typed payloads. Errors/edges: none beyond serde. Invariant: every cross-crate message is an `Event`. Done-check: the three criteria.

### T-00.09  Define the config schema
id: T-00.09
phase: 0
status: pending
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_config::ZiraConfig` is a serde struct with the sub-sections `paths`, `model`, `wakeword`, `vad`, `stt`, `tts`, `emotion`, `memory`, and `avatar`, each a typed sub-struct.
  - C2: every field has a serde default so a fully-empty TOML document deserializes to a complete `ZiraConfig`.
  - C3: a repo-root integration test `tests/config_schema.rs` deserializes `""` (empty doc) into `ZiraConfig` and asserts the defaults match `ZiraConfig::default()`.
not_doing:
  - No file IO here (loading is the next task).
  - No validation logic here (a later task).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The typed configuration surface. Inputs: a TOML document (possibly empty). Outputs: a fully-defaulted `ZiraConfig`. Errors/edges: an absent field uses its serde default. Invariant: config is always complete after deserialization. Done-check: the three criteria.

### T-00.10  Load the config file
id: T-00.10
phase: 0
status: pending
depends_on: [T-00.09]
stack: rust
criteria:
  - C1: `zira_config::load_from(path)` reads a TOML file into `ZiraConfig`, applying serde defaults for absent fields.
  - C2: a missing file returns `ZiraConfig::default()` (not an error), and a present-but-partial file overlays only its set fields.
  - C3: a repo-root integration test `tests/config_load.rs` writes a partial TOML fixture to a temp dir, loads it, and asserts the set field overrides while unset fields keep their defaults; a missing path yields the default config.
not_doing:
  - No XDG path resolution here (the next task); the loader takes an explicit path.
  - No environment-variable overlay.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
Turning a file into config. Inputs: a filesystem path. Outputs: a `ZiraConfig` with file values over defaults; default on absent file. Errors/edges: a malformed TOML is a typed error; an absent file is the default, not an error. Invariant: loading never panics. Done-check: the three criteria.

### T-00.11  Resolve the data paths
id: T-00.11
phase: 0
status: pending
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_config` exposes helpers for the config, data, memory, and skills directories rooted under the XDG base dirs (honoring `XDG_CONFIG_HOME`/`XDG_DATA_HOME` when set).
  - C2: a `ensure_dirs()` helper creates any missing directory and is idempotent (a second call succeeds).
  - C3: a repo-root integration test `tests/config_paths.rs` points the XDG env vars at a temp dir, calls the helpers, and asserts the four directories resolve under it and are created.
not_doing:
  - No file content management — directory resolution + creation only.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
Where Zira keeps its state on disk. Inputs: the XDG environment. Outputs: four resolved, created directories. Errors/edges: an un-creatable path is a typed error; an existing dir is fine. Invariant: paths honor XDG and are create-if-missing. Done-check: the three criteria.

### T-00.12  Embed the constitution
id: T-00.12
phase: 0
status: pending
depends_on: [T-00.02]
stack: rust
criteria:
  - C1: `zira_config::Constitution` is loaded from an embedded default via `include_str!` (compiled into the binary), so the baseline constitution is always present without a file on disk.
  - C2: the loaded `Constitution` exposes its rules through read-only accessors with no public mutator (immutable after load).
  - C3: a repo-root integration test `tests/constitution.rs` loads the embedded constitution, asserts it is non-empty, and confirms there is no public API to mutate a loaded rule set.
not_doing:
  - No enforcement logic here (that is the `zira-skills` safety gate, a later phase).
  - No on-disk override format yet.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The immutable baseline policy compiled into Zira. Inputs: the embedded default text. Outputs: a read-only `Constitution`. Errors/edges: a malformed embedded default fails at parse, loudly. Invariant: a loaded constitution cannot be mutated. Done-check: the three criteria.

### T-00.13  Validate the config
id: T-00.13
phase: 0
status: pending
depends_on: [T-00.09]
stack: rust
criteria:
  - C1: `ZiraConfig::validate()` returns `Result<(), ConfigError>` where `ConfigError` is a typed `thiserror` enum naming the offending field and reason.
  - C2: validation rejects at least: a non-positive sample rate, an empty model/binary path where one is required, and an out-of-range threshold — each as a distinct `ConfigError` variant.
  - C3: a repo-root integration test `tests/config_validate.rs` asserts a default config validates Ok and that each invalid fixture yields the specific expected `ConfigError`.
not_doing:
  - No auto-repair — validation reports, it does not silently fix.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
Catching bad config loudly. Inputs: a `ZiraConfig`. Outputs: Ok or a field-specific `ConfigError`. Errors/edges: each invalid field maps to a distinct typed error. Invariant: an invalid config never reaches the runtime silently. Done-check: the three criteria.

### T-00.14  Define the Orchestrator
id: T-00.14
phase: 0
status: pending
depends_on: [T-00.06, T-00.08]
stack: rust
criteria:
  - C1: `zira_core::Orchestrator` holds the current `State` (starting `Idle`) and the channel handles for the command + event buses.
  - C2: a constructor builds an `Orchestrator` in `Idle` and exposes a read-only `state()` accessor.
  - C3: a repo-root integration test `tests/orchestrator_new.rs` constructs an `Orchestrator` and asserts its initial `state()` is `State::Idle`.
not_doing:
  - No transition or run-loop logic here (later tasks).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The runtime's owner of conversation state. Inputs: channel handles. Outputs: an `Orchestrator` in Idle with a state accessor. Errors/edges: none. Invariant: a fresh orchestrator is Idle. Done-check: the three criteria.

### T-00.15  Build the event bus
id: T-00.15
phase: 0
status: pending
depends_on: [T-00.02, T-00.08]
stack: rust
criteria:
  - C1: `zira_core` constructs an mpsc command channel and a broadcast event channel typed over `zira_proto::Event`, returning the sender/receiver handles.
  - C2: a published `Event` is observed by every subscribed broadcast receiver, and the command channel delivers to its single consumer.
  - C3: a repo-root integration test `tests/event_bus.rs` (tokio) publishes an `Event` to two broadcast subscribers and asserts both receive it, and that a command sent on the mpsc channel is received once.
not_doing:
  - No orchestrator wiring here (that is the run loop task).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The fan-out spine. Inputs: events + commands. Outputs: a broadcast event channel + an mpsc command channel over `Event`. Errors/edges: a lagging subscriber follows tokio broadcast semantics. Invariant: events fan out to all subscribers. Done-check: the three criteria.

### T-00.16  Define the transition table
id: T-00.16
phase: 0
status: pending
depends_on: [T-00.06, T-00.08]
stack: rust
criteria:
  - C1: `zira_core::next_state(current: State, event: &Event) -> Option<State>` implements the PLAN.md §5 table (e.g. `Idle` + `WakeDetected` -> `Listening`; `Speaking` + `BargeIn` -> `Listening`; `Thinking` + `PlanReady` -> `PlanReview`).
  - C2: an event with no defined transition from the current state returns `None` (a no-op), never a panic or a wrong state.
  - C3: a repo-root integration test `tests/transitions.rs` asserts every valid `(state, event)` pair from the table yields the expected next state, and that a sampling of undefined pairs return `None`.
not_doing:
  - No side effects here — `next_state` is a pure function.
  - No timers (the silence timeout is a separate task).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The pure heart of the state machine. Inputs: the current state + an event. Outputs: `Some(next)` for a defined transition, `None` otherwise. Errors/edges: undefined pairs are no-ops. Invariant: transitions are total and pure. Done-check: the three criteria.

### T-00.17  Run the orchestrator loop
id: T-00.17
phase: 0
status: pending
depends_on: [T-00.14, T-00.15, T-00.16]
stack: rust
criteria:
  - C1: `Orchestrator::run()` is an async select-loop that consumes events from the bus, applies `next_state`, and updates the held `State` on each defined transition.
  - C2: an undefined transition leaves the state unchanged and the loop continues; a shutdown command exits the loop cleanly.
  - C3: a repo-root integration test `tests/orchestrator_run.rs` (tokio) feeds a scripted event sequence and asserts the orchestrator's `state()` advances through the expected states, then exits on shutdown.
not_doing:
  - No real stages here — events are injected directly in the test.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The live driver. Inputs: events from the bus. Outputs: an advancing `State` + clean shutdown. Errors/edges: undefined transitions are ignored; shutdown exits. Invariant: state only changes via `next_state`. Done-check: the three criteria.

### T-00.18  Log the transitions
id: T-00.18
phase: 0
status: pending
depends_on: [T-00.16]
stack: rust
criteria:
  - C1: each applied transition emits a `tracing` event recording `from`, `to`, and the triggering event's discriminant.
  - C2: a no-op (undefined) transition does not emit a state-change log line.
  - C3: a repo-root integration test `tests/transition_log.rs` installs a capturing tracing subscriber, drives one valid and one invalid transition, and asserts exactly one state-change record with the correct from/to was emitted.
not_doing:
  - No metrics or external telemetry — tracing only.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
An auditable trail of the conversation flow. Inputs: applied transitions. Outputs: one structured tracing record per real transition. Errors/edges: no-ops are silent. Invariant: every real state change is logged once. Done-check: the three criteria.

### T-00.19  Add the silence timeout
id: T-00.19
phase: 0
status: pending
depends_on: [T-00.17]
stack: rust
criteria:
  - C1: while in `Listening`, a configurable silence timeout elapsing with no `SpeechStarted`/`SpeechEnded` drives `Listening -> Idle`.
  - C2: the timer is cancelled/reset when speech activity arrives before it fires, so an active utterance is never cut to Idle.
  - C3: a repo-root integration test `tests/silence_timeout.rs` (tokio, with a paused/advanced clock) asserts the timeout fires `Listening -> Idle` on silence and does NOT fire when speech activity arrives first.
not_doing:
  - No VAD here — the test injects activity events directly.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
Returning to rest after silence. Inputs: the Listening state + a clock. Outputs: a `Listening -> Idle` transition on timeout, cancelled by activity. Errors/edges: activity resets the timer. Invariant: only genuine silence returns to Idle. Done-check: the three criteria (deterministic via a controlled clock).

### T-00.20  Define the stage traits
id: T-00.20
phase: 0
status: pending
depends_on: [T-00.08]
stack: rust
criteria:
  - C1: `zira_core` defines the stage traits `WakeSource`, `VadGate`, `SttEngine`, `Brain`, `TtsEngine`, `AvatarSink`, and `MemoryStore`, each with the minimal async method(s) the orchestrator needs.
  - C2: a mock implementation of each trait exists (test-only or feature-gated) that emits scripted events without touching real hardware/FFI.
  - C3: a repo-root integration test `tests/stage_traits.rs` drives each mock through its trait method and asserts it produces the expected scripted `Event`(s).
not_doing:
  - No real engines here — the real STT/TTS/wake/avatar impls are blocked-on-human (hardware/FFI/GPU).
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The seam that lets devices be mocked. Inputs: the orchestrator's needs. Outputs: seven traits + a mock each. Errors/edges: mocks are deterministic. Invariant: the orchestrator depends on traits, never concrete engines. Done-check: the three criteria.

### T-00.21  Integrate the mock cycle
id: T-00.21
phase: 0
status: pending
depends_on: [T-00.17, T-00.20]
stack: rust
criteria:
  - C1: the orchestrator can be assembled from the seven mock stages and run end-to-end on injected events.
  - C2: a repo-root integration test `tests/mock_cycle.rs` (tokio) drives a full `Idle -> Listening -> Transcribing -> Thinking -> Speaking -> Idle` cycle through the mocked stages and asserts the state path is exactly that sequence.
  - C3: a barge-in event during `Speaking` drives the mocked cycle back to `Listening`, asserted by the same test.
not_doing:
  - No real audio/brain — every stage is a mock; this proves the wiring, not the devices.
test_files: []
criteria_map: {}
attempts: 0
last_failure: ""
---
The Phase-0 acceptance: the whole loop cycles on mocks. Inputs: the mock stages + injected events. Outputs: a verified Idle->...->Idle path plus a barge-in path. Errors/edges: barge-in re-enters Listening. Invariant: the state machine + bus + traits compose correctly. Done-check: the three criteria.
