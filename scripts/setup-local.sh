#!/bin/sh
set -eu

# Local setup: check, build, test, create release archive, install.
# Usage: ./scripts/setup-local.sh [version]
# Override install dir: BASHERS_INSTALL_DIR=/path ./setup-local.sh

cd "$(dirname "$0")/.."

echo "==> Checking..."
cargo check

echo ""
echo "==> Building..."
cargo build --release

echo ""
echo "==> Testing..."
cargo test

echo ""
echo "==> Creating release archive..."
tar czf bashers-linux-x86_64.tar.gz -C target/release bashers bs

echo ""
echo "==> Installing..."
version="${1:-0.4.9}"
bin_dir="${BASHERS_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$bin_dir"

tar -xzf bashers-linux-x86_64.tar.gz -C /tmp
mv /tmp/bashers "$bin_dir/bashers"
mv /tmp/bs "$bin_dir/bs"
chmod +x "$bin_dir/bashers" "$bin_dir/bs"

echo ""
echo "✓ bashers $version installed to $bin_dir/bashers (alias: bs)"
echo ""

if ! echo "$PATH" | grep -q "$bin_dir"; then
    echo "⚠️  $bin_dir is not in your PATH"
    echo "Add this to your shell configuration:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

echo "==> Done. Try: bashers --help  or  bs --help"
