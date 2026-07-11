# Cliprove

[中文文档](./README.zh-CN.md)

Cliprove is a local-first desktop application for discovering, downloading, and organizing publicly accessible video content from multiple platforms. All data, credentials, and downloaded media stay on your device—no cloud account or remote backend required.

**Supported platforms:** Douyin, Bilibili

## Highlights

- **Link workflow** — Paste a share URL, preview metadata, choose assets, and enqueue downloads
- **Keyword search** — Search with filters, multi-select results, and batch download
- **Task center** — Progress, retries, structured errors, and recovery after interruption
- **Local library** — Full-text search, tags, collections, and Finder integration
- **Privacy by design** — SQLite database and cookies stored locally only

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri v2, Rust |
| UI | React, TypeScript, Vite, Tailwind CSS |
| Storage | SQLite (WAL) |
| Download engine | Python sidecar (FastAPI), platform adapters |

Douyin downloads are powered by the [douyin-downloader](https://github.com/jiji262/douyin-downloader) engine (git submodule). Bilibili uses `yt-dlp` and `bilibili-api-python`.

## Requirements

| Tool | Version | Notes |
|------|---------|-------|
| macOS | 13+ | Apple Silicon recommended |
| Node.js | 20+ | Frontend toolchain |
| Rust | stable | Via [rustup](https://rustup.rs) |
| Python | 3.11+ | Development only; bundled in release builds |
| FFmpeg | latest | `brew install ffmpeg` — required for Bilibili merge & some streams |

## Quick Start

```bash
git clone https://github.com/ingeniousfrog/Cliprove.git
cd Cliprove
git submodule update --init --recursive
chmod +x scripts/dev.sh
./scripts/dev.sh
```

For a step-by-step guide, see [docs/development.md](./docs/development.md).

## Build Release

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

The installer is written to `src-tauri/target/release/bundle/dmg/`. See [docs/packaging.md](./docs/packaging.md) for sidecar rebuilds and engine updates.

## Repository Layout

```
src/           React UI
src-tauri/     Rust core — database, task queue, Tauri commands
sidecar/       Python engine service
engines/       Platform engines (git submodules)
scripts/       Development and packaging scripts
docs/          Development, packaging, and troubleshooting guides
```

## Documentation

- [Development](./docs/development.md)
- [Packaging](./docs/packaging.md)
- [Troubleshooting](./docs/troubleshooting.md)

## License

MIT — see [LICENSE](./LICENSE).
