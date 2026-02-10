#!/bin/sh
set -eu

CARGO_BIN="$HOME/.cargo/bin"
PATH_LINE="export PATH=\"\$HOME/.cargo/bin:\$PATH\""

usage() {
    echo "Usage: $0 [--no-path]"
    echo "  Install bashers via cargo and optionally add ~/.cargo/bin to your shell config."
    echo "  --no-path   Skip adding PATH to profile (only run cargo install)."
    exit 0
}

NO_PATH=false
for arg in "$@"; do
    case "$arg" in
        --no-path) NO_PATH=true ;;
        -h|--help) usage ;;
    esac
done

echo "==> Installing bashers..."
cargo install bashers --force

if [ "$NO_PATH" = true ]; then
    echo "==> Done. Ensure $CARGO_BIN is in your PATH."
    exit 0
fi

if echo ":$PATH:" | grep -q ":$CARGO_BIN:"; then
    echo "==> Done. $CARGO_BIN is already in your PATH."
    exit 0
fi

pick_profile() {
    if [ -n "${ZSH_VERSION:-}" ] && [ -f "$HOME/.zshrc" ]; then
        echo "$HOME/.zshrc"
        return
    fi
    if [ -n "${BASH_VERSION:-}" ] && [ -f "$HOME/.bashrc" ]; then
        echo "$HOME/.bashrc"
        return
    fi
    if [ -f "$HOME/.profile" ]; then
        echo "$HOME/.profile"
        return
    fi
    if [ -n "${ZSH_VERSION:-}" ]; then
        echo "$HOME/.zshrc"
        return
    fi
    if [ -n "${BASH_VERSION:-}" ]; then
        echo "$HOME/.bashrc"
        return
    fi
    echo "$HOME/.profile"
}

PROFILE="$(pick_profile)"

if grep -q '\.cargo/bin' "$PROFILE" 2>/dev/null; then
    echo "==> Done. $CARGO_BIN is already configured in $PROFILE."
    exit 0
fi

echo "" >> "$PROFILE"
echo "# cargo install path (added by bashers install.sh)" >> "$PROFILE"
echo "$PATH_LINE" >> "$PROFILE"
echo "==> Done. Added $CARGO_BIN to $PROFILE. Run \`source $PROFILE\` or start a new shell."
