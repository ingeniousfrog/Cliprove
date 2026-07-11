"""Make douyin-downloader importable from the sidecar."""

from __future__ import annotations

import sys
from pathlib import Path


def _engine_root() -> Path:
    if getattr(sys, "frozen", False):
        return Path(sys._MEIPASS) / "engines" / "douyin-downloader"
    return Path(__file__).resolve().parents[3] / "engines" / "douyin-downloader"


ENGINE_ROOT = _engine_root()


def ensure_engine_path() -> Path:
    engine_path = str(ENGINE_ROOT)
    if engine_path not in sys.path:
        sys.path.insert(0, engine_path)
    if not ENGINE_ROOT.exists():
        raise RuntimeError(
            f"douyin-downloader engine not found at {ENGINE_ROOT}. "
            "Run: git submodule update --init --recursive"
        )
    return ENGINE_ROOT
