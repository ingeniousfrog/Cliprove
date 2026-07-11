# Cliprove

Local-first desktop app for searching, collecting, and managing publicly accessible video content from multiple platforms.

## Stack

- **Desktop**: Tauri v2 + Rust
- **UI**: React + TypeScript + Vite + Tailwind CSS
- **Data**: SQLite (application database)
- **Engine**: Python sidecar (FastAPI) — Phase 0 health check only

## Phase 1 status

Douyin link workflow uses the real `douyin-downloader` engine via Python sidecar:

- Parse Douyin share links (video / gallery) through `/v1/parse`
- Download selected assets to the Cliprove library layout
- Cookie validation via settings
- Bilibili remains on mock until Phase 3

## Phase 0 status (completed)

Foundation delivered mock end-to-end flows for all five pages, SQLite persistence, and sidecar health checks.

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
