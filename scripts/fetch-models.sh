#!/usr/bin/env bash
# fetch-models.sh — download the model assets Zira's device-bound tests need.
#
# Everything lands under ~/.cache/zira/models (the default paths the tests look for, so
# no env vars are needed afterwards). Re-runnable: existing files are re-fetched. Total
# download is ~310 MB. No GPU required for any of this — these are all CPU models.
set -euo pipefail

ROOT="${ZIRA_MODEL_ROOT:-$HOME/.cache/zira/models}"
echo "Fetching Zira model assets into: $ROOT"
mkdir -p "$ROOT"

dl() { # dl <url> <dest>
  local url="$1" dest="$2"
  mkdir -p "$(dirname "$dest")"
  echo "  -> $(basename "$dest")"
  curl -fL --retry 3 -s -o "$dest" "$url"
}

# 1) Embedding model — all-MiniLM-L6-v2 (384-d BERT, ~90 MB) for zira-memory's CandleEmbedder.
EMB="$ROOT/all-MiniLM-L6-v2"
HF_EMB="https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main"
echo "[1/3] embedding model (all-MiniLM-L6-v2)"
dl "$HF_EMB/config.json"          "$EMB/config.json"
dl "$HF_EMB/tokenizer.json"       "$EMB/tokenizer.json"
dl "$HF_EMB/model.safetensors"    "$EMB/model.safetensors"

# 2) STT — whisper-tiny.en (~150 MB) + mel filters + a 16 kHz speech fixture, for WhisperStt.
STT="$ROOT/whisper-tiny.en"
HF_STT="https://huggingface.co/openai/whisper-tiny.en/resolve/main"
echo "[2/3] STT model (whisper-tiny.en)"
dl "$HF_STT/config.json"          "$STT/config.json"
dl "$HF_STT/tokenizer.json"       "$STT/tokenizer.json"
dl "$HF_STT/model.safetensors"    "$STT/model.safetensors"
dl "https://github.com/huggingface/candle/raw/main/candle-examples/examples/whisper/melfilters.bytes" "$STT/melfilters.bytes"
dl "https://raw.githubusercontent.com/ggerganov/whisper.cpp/master/samples/jfk.wav"                   "$STT/jfk.wav"

# 3) TTS — Piper en_US-lessac-medium voice (~63 MB) for PiperTts.
TTS="$ROOT/piper/en_US-lessac-medium"
HF_TTS="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium"
echo "[3/3] TTS voice (en_US-lessac-medium)"
dl "$HF_TTS/en_US-lessac-medium.onnx"      "$TTS/en_US-lessac-medium.onnx"
dl "$HF_TTS/en_US-lessac-medium.onnx.json" "$TTS/en_US-lessac-medium.onnx.json"

echo
echo "Done. Verify sizes (model.safetensors should be ~150M for whisper, ~90M for MiniLM):"
du -h "$EMB/model.safetensors" "$STT/model.safetensors" "$TTS/en_US-lessac-medium.onnx" 2>/dev/null || true
echo
echo "Now run:  cargo test --workspace"
echo "(the model-gated tests run when these assets are present; without them they skip cleanly.)"
