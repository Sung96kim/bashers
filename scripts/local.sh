#!/bin/sh
cd "$(dirname "$0")/.."
exec cargo run --bin bashers -- "$@"
