# Zira — PLAN.md

A voice-driven coding helper and general assistant built **on top of Claude Code**, implemented as a **pure-Rust** harness targeting **lightweight, CPU-first hardware**. Zira does not reimplement the agent loop — it wraps the official `claude` binary as its brain and bolts voice (with emotion), a VRM avatar, on-disk memory, and a self-extension layer onto it.

This single document covers both the architecture (§1–§8) and the task-level execution breakdown (§9–§10).

> **Ratchet note.** This `PLAN.md` is the human design doc / roadmap. The Ratchet build
> loop is driven by the derived task trio `plan.md` / `spec.md` / `list.md` (canonical
> `T-<phase>.<seq>` records). The plan's `P<phase>-<area>-NN` ids below are the
> human-readable roadmap; they are re-expressed as `T-00.NN` records in the trio.

---

## 1. Design thesis

> Zira is a **shell around Claude Code**, not a reimplementation of it.

The official `claude` binary already provides the full agent loop: tool use, file editing, bash, web, subagents, MCP servers, skills, hooks, and permission modes (including plan mode). Zira keeps the brain as-is and adds what Claude Code lacks:

1. **Voice I/O with emotion** — wakeword → STT in; emotion-tagged TTS out.
2. **An embodied avatar** — a VRM model with lip-sync, idle motion, and emotion-driven expression.
3. **Custom on-disk memory** — layered, retrieval-augmented, self-consolidating.
4. **Self-extension** — authoring new Skills and MCP servers behind a safety gate.

Everything routes through one long-lived Claude Code session over its stream-json protocol.

---

## 2. Hard constraints (decided)

| Constraint | Decision |
|---|---|
| Language | **Pure Rust** harness. The only non-Rust artifact is the official `claude` binary (the universal SDK substrate). |
| Brain | The **official `claude` CLI binary**, driven over `--input-format stream-json --output-format stream-json`. |
| Plan support | Required and confirmed via the `plan` permission mode (read-only planning, dynamically switchable). |
| Core model | A formal **conversation state machine** with **barge-in**. |
| Memory | Custom, **on disk**, layered, retrieval-augmented. |
| Emotion | TTS and avatar driven by inline `[Emotion]` tags emitted by the model. |
| **Target hardware** | **Lightweight / CPU-first. No discrete GPU.** Voice + memory run on CPU. The 3D avatar needs an *integrated* GPU; a fully GPU-less box falls back to a 2D face. |

The hardware constraint is the dominant design force: every model is sized to run in real time on a CPU (tiny/quantized), and the avatar is the one subsystem with a GPU floor.

---

## 3. The Claude Code bridge (`zira-bridge`)

The load-bearing crate. There is **no official Rust Agent SDK** (Anthropic ships TS + Python only), so the bridge is a thin, purpose-built Rust port of the SDK protocol:

- Spawns the authenticated local `claude` binary as a child process.
- Speaks **stream-json over stdio** — bidirectional, long-lived, interruptible.
- Manages session lifecycle: start, resume, interrupt.
- Sets **permission modes**: `Default`, `Plan`, `AcceptEdits`, `BypassPermissions`. New "do X for me" tasks start in `Plan`; Zira narrates the plan via TTS and switches to `AcceptEdits` only on confirmation.
- Registers **MCP servers** (`.mcp.json`) and **Skills** so the self-extension layer can add capabilities at runtime.
- Exposes **hooks** (PreToolUse / PostToolUse) — the integration point for the safety gate (§7).

> Build-vs-borrow: community crates already wrap the CLI. Recommended path is to **own the stream-json handling** rather than depend on someone else's parity effort.

---

## 4. Workspace layout

```
zira/
├── Cargo.toml                # workspace
├── crates/
│   ├── zira                  # binary: wires everything, owns the runtime
│   ├── zira-core             # event bus + orchestrator + state machine
│   ├── zira-bridge           # Claude Code stream-json driver (§3)
│   ├── zira-voice            # wake / vad / stt / tts pipeline (§6)
│   ├── zira-emotion          # emotion-tag parser + Emotion→{prosody,expression} maps (§6.5)
│   ├── zira-avatar           # Bevy VRM renderer + viseme/expression driver (§6.2)
│   ├── zira-memory           # on-disk layered memory (§6.3)
│   ├── zira-skills           # skill/MCP factory + safety gate (§7)
│   ├── zira-config           # config, paths, the constitution
│   └── zira-proto            # shared types + the Event enum + Emotion enum
└── assets/
    ├── zira.vrm              # the avatar model
    ├── wakeword/             # trained rustpotter "Zira" model
    └── models/               # quantized whisper + piper voices
```

---

## 5. Conversation state machine

The orchestrator (`zira-core`) is an explicit state machine on a tokio event bus. **Barge-in** is first-class.

```
[*] --> Idle
Idle --> Listening: wakeword "Zira"
Listening --> Transcribing: VAD end-of-speech
Listening --> Idle: silence timeout
Transcribing --> Thinking: transcript ready
Thinking --> Speaking: response tokens
Thinking --> PlanReview: plan produced
PlanReview --> Thinking: user approves -> AcceptEdits
PlanReview --> Idle: user rejects
Speaking --> Idle: utterance complete
Speaking --> Listening: BARGE-IN (wakeword/speech)
Thinking --> Listening: BARGE-IN (interrupt turn)
```

In **Idle** only the wakeword detector is hot (cheap CPU). Heavy stages (STT, TTS) load their models once at startup and stay resident.

---

## 6. Subsystem specs

### 6.1 `zira-voice` (lightweight, CPU)

| Stage | Crate / approach | Purity |
|---|---|---|
| Wakeword "Zira" | **rustpotter** (custom-trained), CPU | pure |
| VAD | **earshot** (pure-Rust WebRTC VAD) | pure |
| STT | **whisper.cpp** tiny/base (quantized) via `whisper-rs`, CPU — or Candle whisper-tiny for purity | FFI (or pure, slower) |
| TTS | **`ort`** running **Piper** (CPU, low-resource) + emotion modulation | FFI |

STT and TTS are sized for CPU: whisper **tiny/base** (not large), Piper (designed for Raspberry-Pi-class devices). Streaming is required end-to-end — TTS synthesizes per emotion-segment so Zira starts speaking before the full response lands, and emits **viseme timing** for lip-sync.

### 6.2 `zira-avatar`

Pure-Rust real-time render of a **VRM** model.

- Engine: **Bevy** (wgpu). Targets an **integrated GPU** at 30fps, low settings, low-poly model.
- VRM loading: **`bevy_vrm`**, or Bevy's glTF loader + manual humanoid/blendshape handling if `bevy_vrm` is too immature. **Maturity risk.**
- Lip-sync: TTS viseme stream → mouth blendshapes (A/I/U/E/O), amplitude fallback.
- Expression: emotion tags (§6.5) select a VRM expression preset (happy/sad/angry/…), smoothly blended.
- Idle life: blink, breathing sway, saccades; a "thinking" pose during `Thinking`.
- Driven over in-process channels (no webview).

> **GPU floor:** one VRM model needs *some* GPU; an integrated Intel/AMD iGPU suffices at 30fps. A truly GPU-less box cannot realtime-render the 3D avatar — on such hardware fall back to a 2D/static expression face.

### 6.3 `zira-memory`

On-disk, layered, retrieval-augmented, under the XDG data dir.

- **Episodic** — append-only JSONL per conversation.
- **Semantic** — distilled facts. Store: **redb** (pure Rust) or `rusqlite`.
- **Index** — embeddings via **Candle** (small quantized model, **CPU**), vector search via **`hnsw_rs`** / **`instant-distance`** (pure Rust).
- **Retrieval** — each turn: embed query, pull top-k, inject as context into the Claude turn.
- **Consolidation** — a scheduled **stateless pass** distills episodic logs into semantic facts and prunes.

`CLAUDE.md` becomes *one output target* of this system, not the system itself.

### 6.4 `zira-core`

Owns the tokio runtime, event bus, and state machine. Routes `Wake → Transcript → MemoryContext → BridgeTurn → EmotionParse → {PlanReview | Speak+Express} → AvatarVisemes`. Handles barge-in via bridge `interrupt()` + TTS flush. All cross-crate messages are `zira-proto::Event` variants.

### 6.5 `zira-emotion` (emotion tags)

The model emits inline `[Emotion]` tags in spoken text; one parser strips them and drives **both** TTS prosody and avatar expression from a single stream.

- **Vocabulary** (fixed, small, in `zira-proto::Emotion`): `Neutral, Happy, Sad, Angry, Excited, Calm, Curious, Concerned, Playful, Tired`. Kept small so the model uses it reliably; unknown tags → `Neutral`.
- **Production**: a system-prompt fragment instructs Claude Code to prefix clauses with tags, e.g. `[Happy] That compiles now. [Curious] Want me to add tests?`
- **Parsing**: a *streaming* parser (handles tags split across token deltas) yields `Segment { emotion, text }`, stripping the markers from the spoken text.
- **Fan-out**: the same `Emotion` drives (a) **TTS** via an `EmotionProsody` table `{rate, pitch, energy}` (and optional per-emotion Piper voice), and (b) the **avatar** via an `EmotionExpression` table of blendshape presets.
- **Default**: `Neutral` when no tag is present.

---

## 7. Self-extension + safety (`zira-skills`)

Zira authoring its own Skills and MCP servers gets guardrails:

- **Skill factory** writes `SKILL.md` + assets to a staging dir; **MCP factory** scaffolds a server to staging.
- **Safety gate** on the PreToolUse hook + registration path:
  - An **immutable constitution** (`zira-config`) declares what may register without approval.
  - Sandboxed trial execution before going live.
  - Prompt-injection defense on externally-sourced content.
  - Signed manifests; an **HMAC-chained audit log** of every registration.
- Only after the gate passes does an artifact move from staging into the live skills dir / `.mcp.json`.

---

## 8. Pure-Rust stack at a glance (CPU-first)

| Component | Crate | Purity |
|---|---|---|
| Async runtime / bus | tokio | pure |
| Claude Code bridge | custom over `claude` binary | pure harness |
| Wakeword | rustpotter | pure |
| VAD | earshot | pure |
| STT | whisper.cpp via whisper-rs (CPU); Candle tiny = pure alt | FFI (or pure, slower) |
| TTS | ort + Piper (CPU) + emotion modulation | FFI |
| Emotion | zira-emotion (parser + maps) | pure |
| Embeddings | candle (small, CPU) | pure |
| Vector index | hnsw_rs / instant-distance | pure |
| Facts store | redb (or rusqlite) | pure (FFI if rusqlite) |
| Avatar | bevy + bevy_vrm (integrated GPU) | pure (VRM maturity risk) |
| Config | serde + toml | pure |

Caveats: STT (whisper.cpp FFI, or slower pure-Rust Candle), TTS (ort FFI), optional rusqlite, `bevy_vrm` maturity, and the avatar's integrated-GPU floor. Everything else is genuinely pure Rust on CPU.

---

## 9. Build plan (task-level)

Tasks are deliberately small (≈ one function, module, or focused integration each) and carry stable roadmap ids `P<phase>-<area>-NN`. In the Ratchet trio these become `T-<phase>.<seq>` records.

**Areas:** `WS` workspace · `PROTO` shared types · `CFG` config · `SM` state machine · `BR` bridge · `WAKE` · `VAD` · `STT` · `EMO` emotion · `TTS` · `WIRE` integration · `MEM` memory · `AVA` avatar · `SK` self-extension · `POL` polish.

> **What the Ratchet loop builds vs. what is blocked-on-human** is governed by `CLAUDE.md`.
> In short: the pure-Rust, deterministic substrate (workspace, proto, config, state
> machine, the bridge against a stub `claude`, the emotion parser/maps, memory store +
> retrieval logic, the self-extension safety gate) is frozen-test-gateable and the loop
> BUILDS it. The audio-hardware / FFI / GPU / trained-model / on-device-latency tasks
> (mic capture, real STT/TTS engines, wakeword training, the Bevy/VRM avatar, perf
> measurement on target HW) are **blocked-on-human** — exactly like Syrinx's GPU tasks.

### Phase 0 — Foundations
*Goal: workspace compiles; state machine cycles through states on mocked events.*

**Workspace**
- `P0-WS-01` Create workspace root `Cargo.toml` (`[workspace]`, `resolver = "2"`).
- `P0-WS-02` Generate the 10 member crates as empty libs/bin.
- `P0-WS-03` Declare shared deps in `[workspace.dependencies]`.
- `P0-WS-04` Add `rust-toolchain.toml`, `rustfmt.toml`, clippy lints config.
- `P0-WS-05` Add `tracing-subscriber` init + `RUST_LOG` wiring in the `zira` bin.
- `P0-WS-06` GitHub Actions CI gate. *(done outside the loop — see `.github/workflows/ci.yml`.)*
- `P0-WS-07` Root `CLAUDE.md`. *(done outside the loop.)*

**Shared types (`zira-proto`)**
- `P0-PROTO-01` Define `Emotion` enum (10 variants) + `Default = Neutral` + serde.
- `P0-PROTO-02` Define `State` enum (Idle, Listening, Transcribing, Thinking, PlanReview, Speaking).
- `P0-PROTO-03` Define `Event` enum (the runtime event vocabulary).
- `P0-PROTO-04` Define payload structs: `Transcript`, `AudioChunk`, `Segment`, `VisemeFrame`, `PlanSummary`, `Usage`.
- `P0-PROTO-05` Round-trip serde unit tests for all payloads.

**Config (`zira-config`)**
- `P0-CFG-01` Define `ZiraConfig` with the subsystem sections.
- `P0-CFG-02` Implement TOML load from `~/.config/zira/config.toml` with serde defaults.
- `P0-CFG-03` XDG path helpers (config/data/memory/skills dirs); create-if-missing.
- `P0-CFG-04` Define immutable `Constitution` + load from an embedded read-only default.
- `P0-CFG-05` Config validation + typed `ConfigError`.

**State machine (`zira-core`)**
- `P0-SM-01` Define `Orchestrator` struct (current `State` + channel handles).
- `P0-SM-02` Set up bus: mpsc command channel + broadcast event channel.
- `P0-SM-03` Implement `next_state(State, &Event) -> Option<State>` transition table.
- `P0-SM-04` Implement `run()` select-loop: consume events, apply transitions.
- `P0-SM-05` Transition logging (from → to → trigger) via tracing.
- `P0-SM-06` Silence-timeout timer for `Listening → Idle`.
- `P0-SM-07` Define stage traits + mock impls.
- `P0-SM-08` Wire mocks; integration test the full `Idle → … → Idle` cycle.

### Phase 1 — The spine
*Goal: say "Zira", ask a coding task, hear an emotion-inflected spoken answer.*
Bridge (`zira-bridge`) and the emotion parser/maps (`zira-emotion`) are pure-Rust and
gateable (the bridge is tested against a stub `claude` script). Wake/VAD/STT/TTS need
audio hardware + FFI models and are blocked-on-human. (Roadmap ids `P1-BR-*`,
`P1-WAKE-*`, `P1-VAD-*`, `P1-STT-*`, `P1-EMO-*`, `P1-TTS-*`, `P1-WIRE-*`.)

### Phase 2 — Memory
On-disk layered memory: episodic JSONL, a facts store (redb), Candle CPU embeddings,
a pure-Rust vector index, retrieval + injection, and a stateless consolidation pass.
Mostly gateable; the embedding-model download is the one human prerequisite.
(Roadmap ids `P2-MEM-*`.)

### Phase 3 — Avatar
A Bevy/VRM avatar with lip-sync + emotion expression on an integrated GPU, and a 2D
fallback for GPU-less boxes. GPU-bound — blocked-on-human. (Roadmap ids `P3-AVA-*`.)

### Phase 4 — Self-extension
Skill/MCP factories behind the constitution + sandbox + injection-scan + signed-manifest
+ HMAC-audit safety gate. Pure-Rust and gateable. (Roadmap ids `P4-SK-*`.)

### Phase 5 — Polish
Barge-in tuning, plan-review UX, emotion-vocabulary review, first-run setup, resource
budget audit, packaging, soak test. (Roadmap ids `P5-POL-*`.)

---

## 10. Decisions to record as ADRs

1. STT — whisper.cpp (FFI, fast) vs Candle tiny (pure, slower).
2. TTS — emotion via prosody-modulation vs per-emotion Piper voices.
3. Memory store — redb (pure) vs rusqlite (SQL ergonomics).
4. Avatar — `bevy_vrm` vs manual glTF/VRM loading.
5. Avatar — VRM avatar vs 2D fallback on GPU-less hardware.
