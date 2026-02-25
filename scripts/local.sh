#!/bin/sh
cd "$(dirname "$0")/.."

FEATURES=""
for arg in "$@"; do
    if [ "$arg" = "--gui" ]; then
        FEATURES="--features gui"
        break
    fi
done

exec cargo run --bin bashers $FEATURES -- "$@"
