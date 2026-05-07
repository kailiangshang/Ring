#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "==> Building frontend..."
cd ui
npm ci
npm run build
cd ..

echo "==> Building server (release)..."
cd server
cargo build --release
cd ..

BINARY="server/target/release/ring"
if [ ! -f "$BINARY" ]; then
  BINARY="server/target/release/ring.exe"
fi

echo "==> Done: $BINARY"
