#!/bin/sh
set -eu

# Set up repo so ./scripts/local.sh works: check, build, test.
# Usage: ./scripts/setup-local.sh

cd "$(dirname "$0")/.."

echo "==> Checking..."
cargo check

echo ""
echo "==> Building..."
cargo build

echo ""
echo "==> Testing..."
cargo test

echo ""
echo "==> Done. Run locally: ./scripts/local.sh <command>"
