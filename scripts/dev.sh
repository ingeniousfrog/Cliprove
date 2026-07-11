#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -d sidecar/.venv ]]; then
  python3 -m venv sidecar/.venv
  sidecar/.venv/bin/pip install -r sidecar/requirements.txt
fi

npm install
npm run tauri dev
