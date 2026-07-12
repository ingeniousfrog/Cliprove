# -*- mode: python ; coding: utf-8 -*-
from pathlib import Path

SPEC_PATH = Path(SPECPATH).resolve()
SIDECAR_DIR = SPEC_PATH.parent if SPEC_PATH.suffix == ".spec" else SPEC_PATH
REPO_ROOT = SIDECAR_DIR.parent
ENGINE_SRC = REPO_ROOT / "engines" / "douyin-downloader"

if not ENGINE_SRC.is_dir():
    raise SystemExit(f"douyin-downloader engine not found: {ENGINE_SRC}")

a = Analysis(
    ["app.py"],
    pathex=[str(SIDECAR_DIR)],
    binaries=[],
    datas=[
        (str(ENGINE_SRC), "engines/douyin-downloader"),
    ],
    hiddenimports=[
        "uvicorn.logging",
        "uvicorn.loops",
        "uvicorn.loops.auto",
        "uvicorn.protocols",
        "uvicorn.protocols.http",
        "uvicorn.protocols.http.auto",
        "uvicorn.protocols.websockets",
        "uvicorn.protocols.websockets.auto",
        "uvicorn.lifespan",
        "uvicorn.lifespan.on",
        "engineio.async_drivers.aiohttp",
        "multipart",
        "bilibili_api",
        "yt_dlp",
        "platforms.douyin.service",
        "platforms.douyin.bootstrap",
        "platforms.bilibili.service",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="cliprove-sidecar",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
