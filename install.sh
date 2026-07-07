#!/usr/bin/env bash
# Install or update bro for bash/zsh/fish on Linux/macOS (or WSL).
#   curl -fsSL https://raw.githubusercontent.com/AdityaSawant0912/bro-cli/master/install.sh | bash
#   bash install.sh [--from-source]
set -euo pipefail

REPO="AdityaSawant0912/bro-cli"
BIN_DIR="$HOME/.local/bin"
BRO_BIN="$BIN_DIR/bro"
FROM_SOURCE=0
[[ "${1:-}" == "--from-source" ]] && FROM_SOURCE=1

step()  { printf '\e[36m==> %s\e[0m\n' "$*"; }
ok()    { printf '\e[32m ok  %s\e[0m\n' "$*"; }
skip()  { printf '\e[33mskip %s\e[0m\n' "$*"; }
die()   { printf '\e[31merr  %s\e[0m\n' "$*" >&2; exit 1; }

mkdir -p "$BIN_DIR"

# ── Prebuilt binary ──────────────────────────────────────────────────────────
detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$os" in
        Linux)  echo "x86_64-unknown-linux-gnu" ;;
        Darwin) case "$arch" in
                    arm64)  echo "aarch64-apple-darwin" ;;
                    x86_64) echo "x86_64-apple-darwin" ;;
                    *) return 1 ;;
                esac ;;
        *) return 1 ;;
    esac
}

install_prebuilt() {
    local target url tmp
    target="$(detect_target)" || return 1
    url="https://github.com/$REPO/releases/latest/download/bro-$target.tar.gz"
    tmp="$(mktemp -d)"
    step "Fetching prebuilt binary for $target"
    if ! curl -fsSL "$url" -o "$tmp/bro.tar.gz" 2>/dev/null; then
        rm -rf "$tmp"
        return 1
    fi
    tar -xzf "$tmp/bro.tar.gz" -C "$tmp"
    [[ -f "$tmp/bro" ]] || { rm -rf "$tmp"; return 1; }
    cp -f "$tmp/bro" "$BRO_BIN"
    chmod +x "$BRO_BIN"
    rm -rf "$tmp"
    ok "installed prebuilt binary → $BRO_BIN"
}

# ── Build from source ────────────────────────────────────────────────────────
build_from_source() {
    step "Checking for cargo"
    command -v cargo &>/dev/null || die "cargo not found. Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    ok "$(cargo --version)"

    local repo_root script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [[ -f "$script_dir/Cargo.toml" ]]; then
        repo_root="$script_dir"
    else
        step "Cloning $REPO"
        repo_root="$(mktemp -d)/bro-cli"
        git clone --depth 1 "https://github.com/$REPO.git" "$repo_root"
    fi

    step "Building release binary"
    # Note: building from /mnt/* in WSL works but is slower than a native path.
    (cd "$repo_root" && cargo build --release)
    local release_bin="$repo_root/target/release/bro"
    [[ -f "$release_bin" ]] || die "build produced no binary at $release_bin"
    cp -f "$release_bin" "$BRO_BIN"
    chmod +x "$BRO_BIN"
    ok "built + installed → $BRO_BIN"
}

if [[ "$FROM_SOURCE" -eq 1 ]]; then
    build_from_source
elif ! install_prebuilt; then
    skip "no matching prebuilt binary, falling back to source build"
    build_from_source
fi

# ── PATH check ───────────────────────────────────────────────────────────────
step "Checking PATH"
if echo "$PATH" | grep -q "$BIN_DIR"; then
    skip "$BIN_DIR already in PATH"
else
    ok "$BIN_DIR will be in PATH after sourcing your rc file (most distros include it automatically)"
fi

# ── Shell wrapper ────────────────────────────────────────────────────────────
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
touch "$RC_FILE"

# Always replace — removes stale wrapper from previous installs
if grep -qF "$MARKER" "$RC_FILE" 2>/dev/null; then
    TMP=$(mktemp)
    awk "
        /^# bro wrapper/ { skip=1; next }
        skip && /^\$/ { skip=0; next }
        skip { next }
        { print }
    " "$RC_FILE" > "$TMP" && mv "$TMP" "$RC_FILE"
    ok "removed old wrapper from $RC_FILE"
fi

if [[ "$SHELL_ARG" == "fish" ]]; then
    printf '\n%s\nbro init fish | source\nbro completions fish | source\n' "$MARKER" >> "$RC_FILE"
else
    printf '\n%s\neval "$(%s init %s)"\neval "$(%s completions %s)"\n' \
        "$MARKER" "$BRO_BIN" "$SHELL_ARG" "$BRO_BIN" "$SHELL_ARG" >> "$RC_FILE"
fi
ok "wrapper + completions updated in $RC_FILE"

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
echo -e "\e[32mDone! Reload your shell:\e[0m"
echo "  source $RC_FILE"
echo ""
echo "Then try:  bro add gs \"git status\"  &&  bro gs"
