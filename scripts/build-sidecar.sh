#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

git submodule update --init --recursive

VENV_DIR="$ROOT/sidecar/.venv"
if [[ ! -d "$VENV_DIR" ]]; then
  if command -v python3 >/dev/null 2>&1; then
    python3 -m venv "$VENV_DIR"
  else
    python -m venv "$VENV_DIR"
  fi
fi

if [[ -x "$VENV_DIR/bin/python3" ]]; then
  PY="$VENV_DIR/bin/python3"
  PIP="$VENV_DIR/bin/pip"
elif [[ -x "$VENV_DIR/Scripts/python.exe" ]]; then
  PY="$VENV_DIR/Scripts/python.exe"
  PIP="$VENV_DIR/Scripts/pip.exe"
else
  echo "Python venv not found under $VENV_DIR"
  exit 1
fi

"$PIP" install -r sidecar/requirements.txt
"$PIP" install -r engines/douyin-downloader/requirements.txt
"$PIP" install "pyinstaller>=6.10.0"

cd "$ROOT/sidecar"
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
