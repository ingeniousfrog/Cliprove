#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f "$ROOT/scripts/fetch-ffmpeg.sh" ]]; then
  chmod +x "$ROOT/scripts/fetch-ffmpeg.sh"
  "$ROOT/scripts/fetch-ffmpeg.sh" || echo "Warning: bundled FFmpeg fetch failed; release build may require system FFmpeg"
fi

"$ROOT/scripts/build-sidecar.sh"
npm install
npm run tauri build

echo "Build complete. Check src-tauri/target/release/bundle/"
