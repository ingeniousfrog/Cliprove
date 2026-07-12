#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f "$ROOT/scripts/fetch-ffmpeg.sh" ]]; then
  chmod +x "$ROOT/scripts/fetch-ffmpeg.sh"
  "$ROOT/scripts/fetch-ffmpeg.sh" || echo "Warning: bundled FFmpeg fetch failed; release build may require system FFmpeg"
fi

if [[ -f "$ROOT/scripts/fetch-ffmpeg.sh" ]]; then
  chmod +x "$ROOT/scripts/fetch-ffmpeg.sh"
  "$ROOT/scripts/fetch-ffmpeg.sh" || echo "Warning: bundled FFmpeg fetch failed; release build may require system FFmpeg"
fi

"$ROOT/scripts/build-sidecar.sh"

if [[ "${CI:-}" == "true" ]]; then
  npm ci
else
  npm install
fi

TAURI_BUNDLES="${TAURI_BUNDLES:-dmg}"
IFS=',' read -ra BUNDLE_LIST <<< "$TAURI_BUNDLES"
npm run tauri build -- --bundles "${BUNDLE_LIST[@]}"

echo "Build complete. Check src-tauri/target/release/bundle/"
