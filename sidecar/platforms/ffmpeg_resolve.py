"""Resolve FFmpeg binary paths outside a login shell PATH."""

from __future__ import annotations

import os
import shutil
from pathlib import Path

COMMON_FFMPEG_PATHS = (
    "/opt/homebrew/bin/ffmpeg",
    "/usr/local/bin/ffmpeg",
    "/usr/bin/ffmpeg",
)


def resolve_ffmpeg_path(configured: str = "ffmpeg") -> str | None:
    configured = (configured or "ffmpeg").strip()

    def normalize(path: str) -> str:
        resolved = shutil.which(path)
        if resolved:
            return resolved
        candidate = Path(path).expanduser()
        if candidate.is_file():
            return str(candidate)
        return path

    def is_valid(path: str) -> bool:
        if shutil.which(path):
            return True
        return Path(path).expanduser().is_file()

    if configured and configured != "ffmpeg":
        return normalize(configured) if is_valid(configured) else None

    for candidate in ("ffmpeg", *COMMON_FFMPEG_PATHS):
        if is_valid(candidate):
            return normalize(candidate)

    bundled = os.environ.get("CLIPROVE_BUNDLED_FFMPEG", "").strip()
    if bundled and is_valid(bundled):
        return normalize(bundled)

    return None
