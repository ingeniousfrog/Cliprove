#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

git submodule update --init --recursive

if [[ ! -d sidecar/.venv ]]; then
  python3 -m venv sidecar/.venv
fi

sidecar/.venv/bin/pip install -r sidecar/requirements.txt
sidecar/.venv/bin/pip install -r engines/douyin-downloader/requirements.txt

npm install
npm run tauri dev
