#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

"$ROOT/scripts/build-sidecar.sh"
npm install
npm run tauri build

echo "Build complete. Check src-tauri/target/release/bundle/"
