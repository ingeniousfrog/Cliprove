# -*- mode: python ; coding: utf-8 -*-
from pathlib import Path

from PyInstaller.utils.hooks import collect_all

SPEC_PATH = Path(SPECPATH).resolve()
SIDECAR_DIR = SPEC_PATH.parent if SPEC_PATH.suffix == ".spec" else SPEC_PATH
REPO_ROOT = SIDECAR_DIR.parent
ENGINE_SRC = REPO_ROOT / "engines" / "douyin-downloader"

if not ENGINE_SRC.is_dir():
    raise SystemExit(f"douyin-downloader engine not found: {ENGINE_SRC}")

# hiddenimports alone is not enough for aiohttp and friends; collect_all pulls
# submodules plus any package data required at runtime (e.g. douyin-downloader).
_collected_packages = (
    "aiohttp",
    "aiofiles",
    "aiosqlite",
    "httpx",
    "certifi",
    "multidict",
    "yarl",
    "frozenlist",
    "aiosignal",
    "attrs",
    "gmssl",
    "rich",
    "yt_dlp",
    "bilibili_api",
)
_extra_datas: list = [(str(ENGINE_SRC), "engines/douyin-downloader")]
_extra_binaries: list = []
_extra_hiddenimports: list = []
for _package in _collected_packages:
    _datas, _binaries, _hiddenimports = collect_all(_package)
    _extra_datas += _datas
    _extra_binaries += _binaries
    _extra_hiddenimports += _hiddenimports

a = Analysis(
    ["app.py"],
    pathex=[str(SIDECAR_DIR)],
    binaries=_extra_binaries,
    datas=_extra_datas,
    hiddenimports=_extra_hiddenimports + [
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
        "platforms.youtube.service",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[str(SIDECAR_DIR / "hooks" / "pyi_rth_ssl_certifi.py")],
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
