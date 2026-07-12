# Cliprove

[中文文档](./README.zh-CN.md)

Cliprove is a local-first desktop application for discovering, downloading, and organizing publicly accessible video content from multiple platforms. All data, credentials, and downloaded media stay on your device—no cloud account or remote backend required.

**Supported platforms:** Bilibili and YouTube in keyword search; Douyin, Bilibili, and YouTube in the link workflow.

## Highlights

- **Link workflow** — Paste a share URL, preview metadata, choose assets, and enqueue downloads
- **Keyword search** — Search with filters, multi-select results, and batch download
- **Task center** — Progress, retries, structured errors, and recovery after interruption
- **Local library** — Full-text search, tags, collections, and Finder integration
- **Privacy by design** — SQLite database and cookies stored locally only

## Download

Pre-built installers are published on **[GitHub Releases](https://github.com/ingeniousfrog/Cliprove/releases)**. Download the asset that matches your platform:

| Platform | Installer | System requirement |
|----------|-----------|-------------------|
| macOS | `.dmg` | macOS 13 (Ventura) or later, **Apple Silicon (arm64)** |
| Windows | `.exe` (NSIS) or `.msi` | Windows 10/11, **x64** |

> **Latest release:** [v0.1.4](https://github.com/ingeniousfrog/Cliprove/releases/tag/v0.1.4)

## Installation Notes

Read this section before installing or distributing Cliprove to end users.

### macOS

1. Open the `.dmg`, then drag **Cliprove** into **Applications**.
2. Launch from Applications or Spotlight.
3. **Gatekeeper (unsigned builds):** Release artifacts are not Apple-notarized by default. On first launch, macOS may block the app. Use **Right-click → Open**, or go to **System Settings → Privacy & Security → Open Anyway**.
4. **FFmpeg:** A static FFmpeg binary is bundled inside the app. You can override the path in **Settings** if needed. Bilibili and YouTube merges require a working FFmpeg.

### Windows

1. Run the `.exe` installer or install via `.msi`.
2. **SmartScreen:** Unsigned builds may trigger Windows Defender SmartScreen. Choose **More info → Run anyway** if you obtained the installer from the official [Releases](https://github.com/ingeniousfrog/Cliprove/releases) page.
3. **FFmpeg:** Bundled with the application; verify status under **Settings** after first launch.

### Platform authentication

| Platform | Login required? | Notes |
|----------|-------------------|-------|
| Douyin | Yes (cookie) | Most share links need a valid session. Configure under **Settings → Platform auth**. |
| Bilibili | Recommended | Public previews work without login; HD or member-only streams need `SESSDATA`. |
| YouTube | No (public content) | Uses `yt-dlp`. Some videos may be **region-restricted** or age-gated by YouTube. |

Cookies are stored **only on your machine** and are never sent to a Cliprove-operated server.

### Data storage

| OS | Location |
|----|----------|
| macOS | `~/Library/Application Support/com.heqk.cliprove/` |
| Windows | `%APPDATA%\com.heqk.cliprove\` |

The directory contains the SQLite database, download settings, platform cookies, and logs. Uninstalling the app does not automatically remove this folder.

### Network & permissions

- Cliprove talks to its local Python sidecar on `127.0.0.1` only; outbound traffic goes directly to the target platforms (Douyin, Bilibili, YouTube).
- Grant read/write access to your chosen download directory.
- Optional clipboard monitoring requires the corresponding system permission.

### Before you start

- Reserve enough disk space for video files and merged outputs.
- Use a stable network connection; large files and Bilibili merges are sensitive to interruption.
- For issues after install, see [Troubleshooting](./docs/troubleshooting.md).

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri v2, Rust |
| UI | React, TypeScript, Vite, Tailwind CSS |
| Storage | SQLite (WAL) |
| Download engine | Python sidecar (FastAPI), platform adapters |

Douyin downloads are powered by the [douyin-downloader](https://github.com/jiji262/douyin-downloader) engine (git submodule). Bilibili uses `yt-dlp` and `bilibili-api-python`; YouTube uses `yt-dlp`.

## Development

### Requirements

| Tool | Version | Notes |
|------|---------|-------|
| macOS | 13+ | Apple Silicon recommended |
| Node.js | 20+ | Frontend toolchain |
| Rust | stable | Via [rustup](https://rustup.rs) |
| Python | 3.11+ | Development only; bundled in release builds |
| FFmpeg | latest | `brew install ffmpeg` — required for Bilibili/YouTube merge & some streams |

### Quick Start

```bash
git clone https://github.com/ingeniousfrog/Cliprove.git
cd Cliprove
git submodule update --init --recursive
chmod +x scripts/dev.sh
./scripts/dev.sh
```

For a step-by-step guide, see [docs/development.md](./docs/development.md).

### Build Release (local)

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

The installer is written to `src-tauri/target/release/bundle/dmg/`. CI publishes multi-platform artifacts via GitHub Actions when a `v*` tag is pushed. See [docs/packaging.md](./docs/packaging.md) for sidecar rebuilds, engine updates, and the release workflow.

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
