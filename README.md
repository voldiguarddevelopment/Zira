<div align="center">

![Zira](docs/social-card.png)

# Zira

**A voice-driven coding helper & assistant — built _on top of_ Claude Code, in pure Rust.**

Wake it with your voice, talk to it, and watch an expressive avatar answer back —
with emotion, memory, and the ability to extend itself. Zira wraps the official
`claude` binary as its brain and adds everything Claude Code doesn't have: voice,
a face, persistent memory, and self-extension — all sized to run on a **CPU-first**
machine with no discrete GPU.

[![CI](https://github.com/voldiguarddevelopment/Zira/actions/workflows/ci.yml/badge.svg)](https://github.com/voldiguarddevelopment/Zira/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-TBD-lightgrey.svg)](#license)
[![Built with Ratchet](https://img.shields.io/badge/built%20with-Ratchet%20%28TDD%20harness%29-7c5cff.svg)](#how-this-was-built)

</div>

---

## The idea

> **Zira is a shell around Claude Code, not a reimplementation of it.**

The official `claude` binary already provides the full agent loop — tool use, file
editing, bash, web, subagents, MCP servers, skills, hooks, and permission modes
(including plan mode). Zira keeps that brain exactly as-is and bolts on the four things
it lacks:

1. 🎙️ **Voice I/O with emotion** — wakeword → speech-to-text in; emotion-tagged
   text-to-speech out.
2. 🧑‍🎤 **An embodied avatar** — a VRM model with lip-sync, idle motion, and
   emotion-driven expression.
3. 🧠 **Custom on-disk memory** — layered, retrieval-augmented, self-consolidating.
4. 🧩 **Self-extension** — authoring its own Skills and MCP servers behind a safety gate.

Everything routes through one long-lived Claude Code session over its stream-json
protocol.

---

## Highlights

- 🦀 **Pure-Rust harness.** The only non-Rust artifact is the official `claude` binary
  (the universal agent substrate). Everything Zira adds is Rust.
- 💻 **CPU-first, no GPU required.** Wakeword, VAD, STT, TTS, and memory all run on a
  CPU in real time (tiny/quantized models). The 3D avatar is the one part with a GPU
  floor — an *integrated* GPU suffices, and a GPU-less box falls back to a 2D face.
- 🗣️ **Emotion as a first-class signal.** The model emits inline `[Emotion]` tags; one
  parser drives **both** TTS prosody and avatar expression from a single stream.
- ⚡ **Barge-in built in.** A formal conversation state machine lets you interrupt
  mid-sentence — the avatar stops, listens, and re-engages.
- 🔒 **Safe self-extension.** New skills/MCP servers pass an immutable constitution +
  sandbox + prompt-injection scan + signed manifest + HMAC-chained audit before going
  live.
- 🛡️ **Token isolation.** The spawned `claude` agent never sees Zira's secrets.

---

## Architecture

```
 voice in                          the brain                         voice + face out
┌─────────┐   ┌─────┐   ┌─────┐   ┌──────────────┐   ┌─────────┐   ┌─────┐   ┌────────┐
│ wakeword│──▶│ VAD │──▶│ STT │──▶│ Claude Code   │──▶│ emotion │──▶│ TTS │──▶│ speaker│
│ "Zira"  │   └─────┘   └─────┘   │ (stream-json) │   │  parser │   └──┬──┘   └────────┘
└─────────┘                       └──────┬───────┘    └────┬────┘      │ visemes
                                         │ memory ctx        │ expression │
                                  ┌──────▼───────┐    ┌──────▼─────────▼─┐
                                  │ on-disk memory│    │   VRM avatar      │
                                  │ (retrieval)   │    │ (lip-sync + mood) │
                                  └───────────────┘    └───────────────────┘
```

The orchestrator (`zira-core`) is an explicit **state machine** on a Tokio event bus —
`Idle → Listening → Transcribing → Thinking → {PlanReview | Speaking} → Idle` — with
**barge-in** as a first-class transition. In `Idle`, only the cheap wakeword detector is
hot; heavy models load once at startup and stay resident.

---

## Workspace layout

A ten-crate Rust workspace; each crate owns one concern.

| Crate | Responsibility |
|-------|----------------|
| [`zira`](crates/zira) | The binary: wires everything together, owns the runtime |
| [`zira-core`](crates/zira-core) | Event bus + orchestrator + conversation state machine |
| [`zira-bridge`](crates/zira-bridge) | The Claude Code **stream-json driver** (spawn, session, permission modes, interrupt) |
| [`zira-proto`](crates/zira-proto) | Shared types: the `Event`, `State`, and `Emotion` vocabulary + payloads |
| [`zira-config`](crates/zira-config) | Config, XDG paths, and the immutable constitution |
| [`zira-emotion`](crates/zira-emotion) | Streaming `[Emotion]`-tag parser + emotion→{prosody, expression} maps |
| [`zira-voice`](crates/zira-voice) | Wakeword / VAD / STT / TTS pipeline (CPU) |
| [`zira-memory`](crates/zira-memory) | On-disk layered memory: episodic, semantic, vector retrieval |
| [`zira-avatar`](crates/zira-avatar) | Bevy VRM renderer + viseme/expression driver |
| [`zira-skills`](crates/zira-skills) | Skill/MCP factory + the self-extension safety gate |

---

## Build status

Zira is built **test-first behind deterministic gates** (see
[How this was built](#how-this-was-built)) — a task is `done` only when its frozen tests
pass, never by assertion. The work splits cleanly into a **pure-Rust, gateable core**
(which the harness builds autonomously) and **device-bound layers** (voice hardware,
FFI model runtimes, the GPU avatar) that need a human + real devices.

**🟢 Building now — the pure-Rust foundation (Phase 0):**
- The ten-crate workspace + shared dependencies + lint policy.
- `zira-proto`: the `Emotion` vocabulary, the `State` and `Event` types, and the typed
  cross-stage payloads.
- `zira-config`: the config schema, TOML loading, XDG path resolution, the immutable
  constitution, and validation.
- `zira-core`: the conversation state machine — transition table, event bus, the
  select-loop, the silence timeout, the stage traits + mocks, and the full
  mocked `Idle → … → Idle` cycle.

**🟡 Next (gateable, pure-Rust):**
- `zira-bridge` — the Claude Code stream-json driver, gated against a **stub `claude`**
  (no real auth/model needed to test the protocol).
- `zira-emotion` — the streaming tag parser + prosody/expression tables.
- `zira-memory` — episodic JSONL, the facts store (redb), CPU embeddings (Candle), the
  vector index, retrieval + injection, and the consolidation pass.
- `zira-skills` — the staging factories + the constitution/sandbox/injection-scan/
  signed-manifest/HMAC-audit safety gate.

**🔴 Blocked-on-human (audio hardware / FFI / GPU / trained models):**
- Wakeword training + mic capture; the real VAD/STT/TTS engines (whisper, Piper-via-ort)
  + audio I/O; the Bevy/VRM avatar renderer; and any on-device latency measurement.
  These can't be expressed as a frozen-test gate — the only way to "pass" them without
  the device is to fake it, which this build refuses to do.

---

## The pure-Rust stack (CPU-first)

| Component | Approach | Purity |
|-----------|----------|--------|
| Async runtime / bus | Tokio | pure |
| Claude Code bridge | custom, over the `claude` binary | pure harness |
| Wakeword | rustpotter (custom-trained) | pure |
| VAD | earshot (pure-Rust WebRTC VAD) | pure |
| STT | whisper.cpp via `whisper-rs` (CPU) — or Candle whisper-tiny | FFI (or pure, slower) |
| TTS | Piper via `ort` (CPU) + emotion modulation | FFI |
| Emotion | `zira-emotion` parser + maps | pure |
| Embeddings | Candle (small, CPU) | pure |
| Vector index | `hnsw_rs` / `instant-distance` | pure |
| Facts store | redb | pure |
| Avatar | Bevy + `bevy_vrm` (integrated GPU) | pure (GPU floor) |

---

## Getting started

> **Heads up:** the pure-Rust foundation builds and tests today; the end-to-end voice
> loop awaits the device-bound layers above (and a one-time model/wakeword setup).

```bash
# Build the whole workspace
cargo build --workspace

# Run the test suite (frozen tests + properties)
cargo test --workspace

# Explore a crate
cargo run -p zira -- --help
```

**Requirements:** a stable Rust toolchain. The full assistant additionally needs a
microphone + speaker, the quantized voice models, and (for the 3D avatar) an integrated
GPU.

---

## How this was built

Zira is built by **[Ratchet](https://github.com/voldiguarddevelopment/Ratchet)**, a
hardened autonomous TDD harness. Every change runs a strict gate cascade —
integrity → checker → compile → frozen tests → mutation — and the project's three
documents (`plan.md` / `spec.md` / `list.md`) are reconciled against the code on every
pass. The core rule is **no stubs, no simplified implementations, no fake passes**: a
green that isn't real is rejected by construction. That's exactly why the build status
above is precise about what is *proven* versus *device-bound* — the harness will not
mark a task done on belief.

---

## Ethics & consent

A voice assistant that listens and remembers carries real responsibility. Zira keeps
its memory **local and on-disk** (under your XDG data dir, never uploaded), strips its
own secrets from the agent it spawns, and gates self-extension behind an immutable
constitution. The roadmap treats consent and safety as requirements, not features.

---

## License

License TBD. Until a license file is added, all rights reserved by the project owners.

<div align="center">
<sub>Built with 🦀 and <a href="https://github.com/voldiguarddevelopment/Ratchet">Ratchet</a> · voldiguarddevelopment</sub>
</div>
