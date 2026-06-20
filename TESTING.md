# Testing Zira on another machine

This guide is for verifying Zira's **device-bound layers** — the parts that need real
models or hardware and therefore can't be checked by CI alone. None of this needs a
powerful GPU; the model inference is **CPU-only**. A modest/integrated GPU only matters for
the (not-yet-built) 3D avatar renderer.

Everything that can be verified purely in software is already green in CI. What's worth
running on a real machine: the **CPU model engines** (embedder, STT, TTS), the **wake-word**
build, and — if you have a GPU — eventually the **avatar render**.

---

## 1. Prerequisites

- **Rust** (stable): https://rustup.rs
- **espeak-ng** — the phonemizer Piper TTS uses. Install for your OS:
  - Debian/Ubuntu: `sudo apt install espeak-ng`
  - Arch: `sudo pacman -S espeak-ng`
  - Fedora: `sudo dnf install espeak-ng`
  - macOS: `brew install espeak-ng`
- **curl** + **git** (for cloning + fetching models).
- **Disk**: ~15 GB free. The dev profile builds *optimized* (`opt-level = 3`, needed so the
  CPU whisper test runs in seconds instead of ~20 min), so the first build is slow (~30–40
  min, mostly compiling `candle` + the ONNX runtime) and the `target/` dir grows to a few GB.

> No `cmake`, no CUDA, no GPU drivers are required for the model tests — STT is pure-Rust
> Candle whisper, and TTS uses the `ort` crate which auto-downloads a CPU ONNX runtime.

---

## 2. Setup

```bash
git clone https://github.com/voldiguarddevelopment/Zira.git
cd Zira

# Fetch the CPU model assets (~310 MB) into ~/.cache/zira/models — the default paths the
# tests look for, so no env vars are needed afterward.
bash scripts/fetch-models.sh

# Build everything (first build is slow — see Prerequisites).
cargo build --workspace
```

---

## 3. Run the tests

```bash
cargo test --workspace
```

Tests split into three groups:

| Group | Runs when… | Examples |
|-------|-----------|----------|
| **Pure-Rust** (always) | always | the state machine, emotion parser, bridge, memory, skills, avatar logic, **barge-in**, **soak**, **VAD** |
| **Model-gated** | the model asset is on disk | **embedder**, **STT**, **TTS** |
| **Feature-gated** | you pass the feature flag | **wake** |

The model-gated tests **skip cleanly** if a model is missing (so CI stays green), and **run
for real** once `scripts/fetch-models.sh` has placed the assets. To confirm they actually
ran (not skipped), run them individually with output:

```bash
# Embedder — loads all-MiniLM-L6-v2, produces a real 384-d vector.
cargo test --test candle_embedder -- --nocapture

# STT — transcribes the bundled jfk.wav; the test asserts it contains "country"/"americans".
cargo test --test whisper_stt -- --nocapture

# TTS — synthesizes a phrase to 22 kHz PCM (asserts real, non-silent audio) via espeak-ng + ort.
cargo test --test piper_tts -- --nocapture

# VAD — pure-Rust, uses committed fixtures (no model needed); speech vs silence.
cargo test --test earshot_vad
```

A green `test result: ok.` on each means that engine works on your hardware.

> The whisper STT test takes ~45 s (CPU inference). The embedder ~couple minutes the first
> time. TTS is fast (one ONNX forward pass).

---

## 4. The two still-blocked tasks

### Wake-word (T-01.15) — `cargo test -p zira-voice --features wake`

The wake detector (`RustpotterWake`, `crates/zira-voice/src/wake.rs`) is **written and ready**
but gated behind a `wake` cargo feature, because the `rustpotter` crate was unbuildable on the
original dev box (its 2.x line is yanked on crates.io and 3.x failed to compile due to a
`candle-gemm`/`rand` conflict).

**Please try it on your toolchain** — it may well build:

```bash
cargo test -p zira-voice --features wake
```

The repo ships a stand-in wake model + audio (`tests/fixtures/wake/alexa.rpw` + `alexa.wav`).
If it builds and the detection test passes, wake works and the only thing left is training a
real **"Zira"** model from voice recordings (a one-time human step with the rustpotter trainer).
If `rustpotter` still won't build, note the error — we may need to pin a working version or
swap the wake library.

### 3D avatar render (T-03.12) — needs a GPU

This one is **not implemented yet** — it needs a GPU + display that the original box lacked
(no `/dev/dri`). Everything it *consumes* is already built and gate-tested: the
emotion→expression presets, viseme timing, the 2D-fallback projection, and the pure
`AvatarDriver` state machine (`crates/zira-avatar`).

If your machine has a working GPU (integrated is fine — an iGPU with Vulkan/`/dev/dri` on
Linux, or Metal on macOS), it's the right place to build + test the Bevy/`wgpu` renderer:
load a `.vrm` model, drive its blendshapes from `AvatarState`, and run the 2D fallback when no
GPU is present. Confirm the GPU is visible first:

```bash
# Linux: should list a render node and a Vulkan device
ls /dev/dri/                 # expect e.g. card0, renderD128
vulkaninfo --summary 2>/dev/null | grep deviceName
```

---

## 5. Reporting back

For each engine, note: did the test **run** (not skip) and **pass**, on what CPU/OS, and how
long it took. For wake, note whether `--features wake` built. For the avatar, note your GPU +
whether a `wgpu`/Bevy hello-triangle runs at all. That tells us which device-bound tasks are
verified on real hardware and which still need work.
