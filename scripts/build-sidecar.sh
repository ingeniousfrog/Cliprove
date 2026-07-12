#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

git submodule update --init --recursive

if [[ ! -d sidecar/.venv ]]; then
  python3 -m venv sidecar/.venv
fi

PY="sidecar/.venv/bin/python3"
PIP="sidecar/.venv/bin/pip"

"$PIP" install -r sidecar/requirements.txt
"$PIP" install -r engines/douyin-downloader/requirements.txt
"$PIP" install "pyinstaller>=6.10.0"

cd sidecar
rm -rf build dist
"$PY" -m PyInstaller cliprove-sidecar.spec --clean --noconfirm

TARGET="$(rustc -vV | awk '/host: / {print $2}')"
DEST="$ROOT/src-tauri/binaries/cliprove-sidecar-${TARGET}"
mkdir -p "$ROOT/src-tauri/binaries"

SIDECAR_BIN="dist/cliprove-sidecar"
if [[ -f "${SIDECAR_BIN}.exe" ]]; then
  SIDECAR_BIN="${SIDECAR_BIN}.exe"
  DEST="${DEST}.exe"
fi

cp "${SIDECAR_BIN}" "$DEST"
chmod +x "$DEST" 2>/dev/null || true

echo "Sidecar binary ready: $DEST"
