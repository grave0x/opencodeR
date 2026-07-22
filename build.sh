#!/usr/bin/env bash
set -euo pipefail

# opencodeR build script
# Builds all binaries for the current platform

APP="opencodeR"
CARGO="cargo"

echo "==> Building $APP binaries (release)"
$CARGO build --release --bin "$APP" --bin "$APP-server" --bin "$APP-client"

echo ""
echo "==> Binaries:"
ls -lh target/release/"$APP" target/release/"$APP-server" target/release/"$APP-client"

echo ""
echo "==> Done. Binary sizes:"
du -sh target/release/"$APP" target/release/"$APP-server" target/release/"$APP-client"
