# Cliprove

Local-first desktop app for searching, collecting, and managing publicly accessible video content from multiple platforms.

## Stack

- **Desktop**: Tauri v2 + Rust
- **UI**: React + TypeScript + Vite + Tailwind CSS
- **Data**: SQLite (application database)
- **Engine**: Python sidecar (FastAPI) — Phase 0 health check only

## Phase 2 status

Douyin keyword search is wired to the real engine:

- `/v1/search` with pagination (`cursor`) and filters (`sort`, `publish_time`)
- Search page supports grid/table views, virtualized lists, multi-select, and batch enqueue
- `searchKeyword` is preserved on library items discovered via search

## Phase 1 status (completed)

Douyin link parse/download via sidecar and `douyin-downloader` engine.

## Prerequisites

- Node.js 20+
- Rust (stable)
- Python 3.11+ (for sidecar)

## Development

```bash
# Install frontend dependencies
npm install

# Optional: sidecar dependencies
pip install -r sidecar/requirements.txt

# Run desktop app
npm run tauri dev
```

## Project layout

```
src/           React UI
src-tauri/     Rust core (DB, tasks, commands)
sidecar/       Python engine service
engines/       Upstream engines (Phase 1+)
```

## Documentation

- [GOAL.md](./GOAL.md) — product requirements
- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) — architecture & phases
