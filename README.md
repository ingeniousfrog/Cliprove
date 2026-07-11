# Cliprove

Local-first desktop app for searching, collecting, and managing publicly accessible video content from multiple platforms.

## Stack

- **Desktop**: Tauri v2 + Rust
- **UI**: React + TypeScript + Vite + Tailwind CSS
- **Data**: SQLite (application database)
- **Engine**: Python sidecar (FastAPI) — Douyin + Bilibili

## Features

- Douyin & Bilibili link parse, search, and download
- Local library with FTS search, tags, and collections
- Task queue with recovery and concurrency control
- macOS `.dmg` packaging with bundled Python sidecar

## Prerequisites

- Node.js 20+
- Rust (stable)
- Python 3.11+ (development only; bundled in release builds)
- FFmpeg (`brew install ffmpeg`)

## Development

```bash
git submodule update --init --recursive
chmod +x scripts/dev.sh
./scripts/dev.sh
```

See [docs/development.md](./docs/development.md) for details.

## Packaging

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

Output: `src-tauri/target/release/bundle/dmg/`

See [docs/packaging.md](./docs/packaging.md) for sidecar rebuild and engine updates.

## Project layout

```
src/           React UI
src-tauri/     Rust core (DB, tasks, commands)
sidecar/       Python engine service
engines/       Upstream engines (git submodule)
scripts/       dev.sh, build.sh, build-sidecar.sh
docs/          development, packaging, troubleshooting
```

## Documentation

- [docs/development.md](./docs/development.md) — setup and daily dev workflow
- [docs/packaging.md](./docs/packaging.md) — PyInstaller sidecar + Tauri DMG build
- [docs/troubleshooting.md](./docs/troubleshooting.md) — common issues
