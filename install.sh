#!/usr/bin/env bash
#
# Zira installer — builds the workspace, installs the `zira` binary, and sets up the
# XDG config/data directories. CPU-first; the voice + avatar model setup is a later
# step (see "Status" at the end). Idempotent and safe to re-run.
#
#   curl -fsSL .../install.sh | bash      # from a fresh machine (clones the repo)
#   ./install.sh                          # from inside a checkout
#
set -euo pipefail

REPO_URL="${ZIRA_REPO:-https://github.com/voldiguarddevelopment/Zira.git}"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/zira"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/zira"

c_say()  { printf '\033[1;35m▸\033[0m %s\n' "$*"; }
c_ok()   { printf '\033[1;32m✓\033[0m %s\n' "$*"; }
c_warn() { printf '\033[1;33m!\033[0m %s\n' "$*"; }
c_die()  { printf '\033[1;31m✗\033[0m %s\n' "$*" >&2; exit 1; }
have()   { command -v "$1" >/dev/null 2>&1; }

c_say "Zira installer"

# --- prerequisites ---------------------------------------------------------------
have cargo || c_die "Rust toolchain not found — install it from https://rustup.rs"
c_ok "rust:   $(cargo --version | awk '{print $2}')"
have git   || c_die "git not found."
c_ok "git:    $(git --version | awk '{print $3}')"
if have claude; then
  c_ok "claude: $(command -v claude)   (Zira's brain)"
else
  c_warn "the official 'claude' CLI is NOT on PATH — Zira drives it as its brain."
  c_warn "install + authenticate it before running Zira: https://claude.com/claude-code"
fi

# --- locate the source -----------------------------------------------------------
if [ -f Cargo.toml ] && grep -q 'zira-core' Cargo.toml 2>/dev/null; then
  SRC="$(pwd)"
  c_say "using the current checkout: $SRC"
else
  SRC="${ZIRA_SRC:-$HOME/development/zira}"
  if [ -d "$SRC/.git" ]; then
    c_say "updating existing checkout: $SRC"
    git -C "$SRC" pull --ff-only || c_warn "could not fast-forward $SRC; continuing with what's there"
  else
    c_say "cloning Zira into $SRC"
    git clone "$REPO_URL" "$SRC"
  fi
fi
cd "$SRC"

# --- build (defensive: the project is in active development) ----------------------
c_say "building the workspace (release) — first build may take a few minutes…"
if cargo build --release --workspace; then
  c_ok "workspace built"
else
  c_warn "the workspace did not fully build — Zira is mid-development and some"
  c_warn "device-bound crates may not be wired yet. Setup continues; re-run later."
fi

# --- install the binary ----------------------------------------------------------
mkdir -p "$BIN_DIR"
if [ -x target/release/zira ]; then
  install -m 0755 target/release/zira "$BIN_DIR/zira"
  c_ok "installed: $BIN_DIR/zira"
  case ":$PATH:" in
    *":$BIN_DIR:"*) : ;;
    *) c_warn "add $BIN_DIR to your PATH (e.g. in ~/.profile)";;
  esac
else
  c_warn "the 'zira' binary isn't produced yet (runtime mid-build) — skipping binary install"
fi

# --- config + data directories ---------------------------------------------------
mkdir -p "$CONFIG_DIR" "$DATA_DIR" "$DATA_DIR/memory" "$DATA_DIR/skills"
c_ok "config dir: $CONFIG_DIR"
c_ok "data dir:   $DATA_DIR"
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
  cat > "$CONFIG_DIR/config.toml" <<'TOML'
# Zira configuration. A fully-empty file is valid — every field defaults.
# See PLAN.md (§6) for the full schema.

[model]
# binary = "claude"          # the Claude Code CLI Zira drives as its brain

[emotion]
# the model emits inline [Emotion] tags; defaults to Neutral when absent
TOML
  c_ok "wrote default config: $CONFIG_DIR/config.toml"
else
  c_ok "config exists: $CONFIG_DIR/config.toml (left untouched)"
fi

# --- status ----------------------------------------------------------------------
cat <<EOF

$(c_say "Setup complete.")

Working today:  the pure-Rust core — workspace, shared types, config, and the
                conversation state machine — builds and tests.

Not wired yet:  the device-bound layers (in active development) — the voice pipeline
                (wakeword / STT / TTS), the VRM avatar, and on-disk memory. The
                end-to-end voice loop goes live once those land and their models are
                fetched. (Tracked in PLAN.md; built test-first by Ratchet.)

Next steps:
  • authenticate the 'claude' CLI if you haven't.
  • a model-setup step (whisper / piper / wakeword assets) will be added with the
    voice pipeline.
EOF
