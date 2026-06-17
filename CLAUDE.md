# CLAUDE.md — Zira project constitution (read fully before any task)

You are building **Zira**, a voice-driven coding helper + general assistant built **on
top of Claude Code**, as a **pure-Rust** harness for **lightweight, CPU-first** hardware.
Zira does NOT reimplement the agent loop — it wraps the official `claude` binary as its
brain and adds voice (with emotion), a VRM avatar, on-disk memory, and a self-extension
layer. This file is the standing law. `PLAN.md` is the full design + roadmap;
`plan.md` / `spec.md` / `list.md` are the derived task trio the loop executes.

Context is thrown away every pass and re-derived from disk. **Disk and git history are
the only memory.** Re-read the relevant files at the start of work; write your
conclusions to disk, not just into your reply. No in-context state survives a pass, so
corner-cutting in one pass cannot poison the next.

---

## Non-negotiable rules

- **No stubs, no simplified implementations, no fake passes.** If you cannot implement
  the real thing, log a blocker and stop — a green that isn't real is the single worst
  outcome in this system. Never weaken a detector, a test, or a gate to get past it.
- **Tests freeze at red-pass.** Never edit a frozen file (test files, `criteria_map`,
  the detectors in `.ratchet/detectors/`) during a green phase.
- **Judgment is deterministic:** the compiler, the frozen tests, the frozen checker, and
  mutation decide — not your opinion. "It sounds right" is never a verdict.
- **One task per worktree; tasks are small by design.** If your context fills up, the
  task should have been split.
- **Fix documents before code:** reconcile plan.md, spec.md, list.md before building.
- **Log all failures with raw tool output,** not paraphrases.
- **IDs are immutable.** Splits add suffixes (`T-00.07a`); nothing is renumbered or deleted.

---

## The canonical format

Every task, in every file, is one fenced record: rigid fields (`id`, `phase`,
`depends_on`, `stack`, `criteria`, `not_doing` — plus in `list.md`: `status`,
`test_files`, `criteria_map`, `attempts`, `last_failure`) that MUST parse, then a `---`,
then free prose. IDs are `T-<phase>.<seq>` and the phase in the id MUST equal the `phase`
field. Titles are imperative, single verb + object, **no "and"**. `criteria` are
falsifiable assertions (each becomes ≥1 test), not descriptions. A non-conforming record
is rejected by the conformance gate and regenerated — wasting a pass.

## Where frozen tests live

The gate detects frozen tests by `path.starts_with("tests/")` relative to the **repo
root**. Frozen integration tests MUST live at **repo-root `tests/*.rs`**, NOT under
`crates/*/tests/`. The root `Cargo.toml` is BOTH `[package]` and `[workspace]` so
`cargo test` at the root runs the repo-root `tests/`. A test written under a member
crate's `tests/` dir is invisible to the gate and the task will stall in RED.

---

## THE BUILD SCOPE — what the loop builds vs. what is blocked-on-human

Zira spans pure-Rust logic AND subsystems that need audio hardware, FFI model runtimes,
a GPU, or a trained model. The latter are **NOT expressible as a frozen-test + mutation
gate** — the only way to "pass" them without the mic / speaker / GPU / trained weights is
to fake a green, the one outcome this system exists to prevent. The loop must NEVER
attempt those. They are marked **`status: blocked`** in `list.md` with a human blocker.
**If handed a blocked task: do not implement it, do not fake a result, do not stub a
device. Re-confirm the blocker, log it, and stop.**

**The loop BUILDS (deterministic, frozen-test-gateable) — the pure-Rust substrate:**

- **Phase 0 (entire):** the Cargo workspace, the shared `zira-proto` types (Emotion,
  State, Event, payloads + serde round-trips), `zira-config` (schema, TOML load, XDG
  paths, the immutable constitution, validation), and the `zira-core` conversation state
  machine (transition table, event bus, the select-loop, silence timeout, the stage
  traits + mock impls, the full mocked Idle→…→Idle integration cycle).
- **`zira-bridge`** (Phase 1): the Claude Code stream-json driver — arg-vector build,
  line-delimited JSON parsing, inbound message structs, text-delta → event mapping,
  result → turn-complete, send-user-message, permission-mode + interrupt control
  requests, plan-mode mapping, typed errors, the respawn watchdog. **Gated against a
  stub `claude` script** emitting canned stream-json (no real auth/model needed).
- **`zira-emotion`** (Phase 1): the streaming `[Emotion]`-tag parser (split across
  deltas), unknown-tag → Neutral, the `EmotionProsody` and `EmotionExpression` tables,
  and the system-prompt fragment. Pure logic, unit-test gated.
- **`zira-memory`** (Phase 2): the on-disk layout, episodic JSONL, the facts store
  (redb), the vector-index insert/search logic, retrieval + context formatting +
  injection, and the stateless consolidation/prune pass. Gateable EXCEPT downloading the
  embedding model (a human prerequisite → that one sub-task is blocked).
- **`zira-skills`** (Phase 4): the skill/MCP staging factories, the constitution check,
  the manifest signing, the HMAC-chained audit log, prompt-injection scanning, and the
  staging→live promotion gate. Pure-Rust and gateable.

**BLOCKED-ON-HUMAN (hardware / FFI / GPU / trained-model / on-device latency):**

- **Wakeword** (`P1-WAKE-*`): mic capture (`cpal`), recording positive clips, training
  the rustpotter model, tuning thresholds on real audio.
- **VAD / STT / TTS** (`P1-VAD-*`, `P1-STT-*`, `P1-TTS-*`): real audio I/O, the whisper
  (FFI/model) and Piper-via-`ort` (FFI/model) engines, model bundling, audio playback,
  and on-device latency measurement. (The pure interfaces/traits these implement live in
  Phase 0; the emotion modulation MATH that does not need a device may be gateable —
  judge per task, and when in doubt mark it blocked.)
- **Avatar** (`P3-AVA-*`): the Bevy/VRM real-time renderer, lip-sync, expression
  blending, and the GPU perf tests. GPU-bound.
- **Any "measure latency / perf on target HW" task,** in any phase.

When the trio later gains these phases, they enter as `status: blocked` with the reason
in `last_failure`. The autonomous path is the pure-Rust substrate above.

---

## The gate sequence (full detail in the Ratchet PLAN)

Gates run in strict order; the first to fail stops the cascade and logs all its failures:
**Reconcile** (re-read docs + code, fix mismatches, split oversized tasks — no code) →
**Red** (write tests from `criteria`; they must compile, fail for the right reason, kill
mutants, cover every criterion; then freeze) → **Green** (implement against frozen tests;
cannot edit frozen files; pass integrity → checker → compile → all tests → mutation) →
**Phase boundary** (full regression green; three files reconcile). If a gate rejects your
work, read the raw failure, fix the actual cause, and retry — never weaken what it checks.

---

## Working conventions

- Idiomatic Rust. Real error handling — no `.unwrap()` walls, no bare `Ok(())` where work
  is owed.
- Pure crates stay pure: keep FFI/audio/GPU dependencies out of the gateable crates
  (`zira-proto`, `zira-config`, `zira-core` logic, `zira-emotion`, the `zira-memory`
  logic, `zira-skills`). The traits in `zira-core` let the real device engines be swapped
  in behind a mock for testing.
- Build artifacts and scratch work stay out of the source tree and out of commits.
- Prefer prose-light, structure-heavy documents: the machine reads the structure, and the
  next agent's context is precious.

## What "done" means

A task is done when its frozen tests pass honestly, the checker is clean, mutation
confirms the tests defend the real code, and every acceptance criterion maps to a passing
test. Nothing is done because you believe it is — it is done because the deterministic
gates say so. When in doubt: re-read from disk, do the smallest honest thing, write the
result down, and let the next pass check you.
