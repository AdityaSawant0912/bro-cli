#!/usr/bin/env bash
# Install bro for bash/zsh on WSL (or any Linux/macOS).
# Run once from the repo root:  bash install.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
BRO_BIN="$BIN_DIR/bro"

step()  { printf '\e[36m==> %s\e[0m\n' "$*"; }
ok()    { printf '\e[32m ok  %s\e[0m\n' "$*"; }
skip()  { printf '\e[33mskip %s\e[0m\n' "$*"; }
die()   { printf '\e[31merr  %s\e[0m\n' "$*" >&2; exit 1; }

# ── 1. Cargo ─────────────────────────────────────────────────────────────────
step "Checking for cargo"
command -v cargo &>/dev/null || die "cargo not found. Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
ok "$(cargo --version)"

# ── 2. Build ─────────────────────────────────────────────────────────────────
step "Building release binary (Linux)"
# Note: building from /mnt/* in WSL works but is slower than a native path.
# For best speed: clone the repo to ~/bro-cli inside WSL.
(cd "$REPO_ROOT" && cargo build --release)
RELEASE_BIN="$REPO_ROOT/target/release/bro"
[[ -f "$RELEASE_BIN" ]] || die "build produced no binary at $RELEASE_BIN"
ok "built $RELEASE_BIN"

# ── 3. Install dir ───────────────────────────────────────────────────────────
step "Ensuring $BIN_DIR exists"
mkdir -p "$BIN_DIR"
ok "$BIN_DIR ready"

# ── 4. Copy binary ───────────────────────────────────────────────────────────
step "Installing bro → $BRO_BIN"
cp -f "$RELEASE_BIN" "$BRO_BIN"
chmod +x "$BRO_BIN"
ok "copied"

# ── 5. PATH check ────────────────────────────────────────────────────────────
step "Checking PATH"
if echo "$PATH" | grep -q "$BIN_DIR"; then
    skip "$BIN_DIR already in PATH"
else
    ok "$BIN_DIR will be in PATH after sourcing your rc file (most distros include it automatically)"
fi

# ── 6. Shell wrapper ─────────────────────────────────────────────────────────
step "Detecting shell"

DETECTED_SHELL="$(basename "${SHELL:-bash}")"
case "$DETECTED_SHELL" in
    zsh)  RC_FILE="$HOME/.zshrc";  SHELL_ARG="zsh"  ;;
    fish) RC_FILE="$HOME/.config/fish/config.fish"; SHELL_ARG="fish" ;;
    *)    RC_FILE="$HOME/.bashrc"; SHELL_ARG="bash" ;;
esac

ok "shell: $DETECTED_SHELL → rc: $RC_FILE"

step "Installing $SHELL_ARG wrapper to $RC_FILE"

MARKER="# bro wrapper"

if grep -qF "$MARKER" "$RC_FILE" 2>/dev/null; then
    skip "wrapper already in $RC_FILE"
else
    touch "$RC_FILE"
    if [[ "$SHELL_ARG" == "fish" ]]; then
        printf '\n%s\nbro init fish | source\n' "$MARKER" >> "$RC_FILE"
    else
        printf '\n%s\neval "$(%s init %s)"\n' "$MARKER" "$BRO_BIN" "$SHELL_ARG" >> "$RC_FILE"
    fi
    ok "wrapper added to $RC_FILE"
fi

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
echo -e "\e[32mDone! Reload your shell:\e[0m"
echo "  source $RC_FILE"
echo ""
echo "Then try:  bro add gs \"git status\"  &&  bro gs"
