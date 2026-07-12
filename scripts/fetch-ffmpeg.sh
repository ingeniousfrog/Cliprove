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
    OUTPUT="$DEST_DIR/ffmpeg-${TARGET}"
    ARCHIVE_EXT="zip"
    ;;
  x86_64-apple-darwin)
    URL="https://www.osxexperts.net/ffmpeg${FFMPEG_VERSION}64.zip"
    OUTPUT="$DEST_DIR/ffmpeg-${TARGET}"
    ARCHIVE_EXT="zip"
    ;;
  x86_64-pc-windows-msvc|x86_64-pc-windows-gnu)
    URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
    OUTPUT="$DEST_DIR/ffmpeg-${TARGET}.exe"
    ARCHIVE_EXT="zip"
    ;;
  *)
    echo "Unsupported target for bundled FFmpeg: $TARGET"
    exit 1
    ;;
esac

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading FFmpeg for $TARGET ..."
curl -fsSL "$URL" -o "$TMP_DIR/ffmpeg-archive.${ARCHIVE_EXT}"
unzip -q "$TMP_DIR/ffmpeg-archive.${ARCHIVE_EXT}" -d "$TMP_DIR"

if [[ "$OUTPUT" == *.exe ]]; then
  FFMPEG_BIN="$(find "$TMP_DIR" -iname ffmpeg.exe -type f | head -n 1)"
else
  FFMPEG_BIN="$(find "$TMP_DIR" -name ffmpeg -type f | head -n 1)"
fi

if [[ -z "$FFMPEG_BIN" ]]; then
  echo "Failed to locate ffmpeg binary in archive"
  exit 1
fi

cp "$FFMPEG_BIN" "$OUTPUT"
chmod +x "$OUTPUT" 2>/dev/null || true

echo "Bundled FFmpeg ready: $OUTPUT"
