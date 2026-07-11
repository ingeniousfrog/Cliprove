#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST_DIR="$ROOT/src-tauri/resources/ffmpeg"
FFMPEG_VERSION="7.1"
TARGET="$(rustc -vV | awk '/host: / {print $2}')"

mkdir -p "$DEST_DIR"

case "$TARGET" in
  aarch64-apple-darwin)
    URL="https://www.osxexperts.net/ffmpeg${FFMPEG_VERSION}arm64.zip"
  ;;
  x86_64-apple-darwin)
    URL="https://www.osxexperts.net/ffmpeg${FFMPEG_VERSION}64.zip"
  ;;
  *)
    echo "Unsupported target for bundled FFmpeg: $TARGET"
    exit 1
  ;;
esac

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading FFmpeg for $TARGET ..."
curl -fsSL "$URL" -o "$TMP_DIR/ffmpeg.zip"
unzip -q "$TMP_DIR/ffmpeg.zip" -d "$TMP_DIR"

FFMPEG_BIN="$(find "$TMP_DIR" -name ffmpeg -type f | head -n 1)"
if [[ -z "$FFMPEG_BIN" ]]; then
  echo "Failed to locate ffmpeg binary in archive"
  exit 1
fi

OUTPUT="$DEST_DIR/ffmpeg-${TARGET}"
cp "$FFMPEG_BIN" "$OUTPUT"
chmod +x "$OUTPUT"

echo "Bundled FFmpeg ready: $OUTPUT"
