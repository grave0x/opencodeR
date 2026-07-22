#!/usr/bin/env bash
set -euo pipefail

# opencodeR build script
# Builds all three binaries for the current platform

APP="opencodeR"
CARGO_FLAGS="${CARGO_FLAGS:-"--release"}"

echo "==> Building $APP binaries"
echo "    Flags: $CARGO_FLAGS"

# shellcheck disable=SC2086
cargo build $CARGO_FLAGS --bin "$APP" --bin "$APP-server" --bin "$APP-client"

echo ""
echo "==> Binaries:"
ls -lh target/release/"$APP" target/release/"$APP-server" target/release/"$APP-client"

echo ""
echo "==> Done."
