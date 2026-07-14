# Cliprove

[English](./README.md)

Cliprove 是一款本地优先的桌面应用，用于发现、下载和管理多平台公开视频内容。数据、凭证与下载内容均保存在本机，无需云账户或远程后端。

**已支持平台：** 关键词搜索支持 Bilibili 与 YouTube；链接下载支持抖音、Bilibili 与 YouTube。

## 核心能力

- **链接下载** — 粘贴分享链接，预览元数据，选择资源后加入下载队列
- **关键词搜索** — 支持筛选、多选与批量下载
- **任务中心** — 进度、重试、结构化错误展示，中断后可恢复
- **本地库** — 全文搜索、标签、收藏夹，支持 Finder 定位与文件打开
- **隐私优先** — SQLite 数据库与 Cookie 仅存储在本地

## 下载安装

预编译安装包发布于 **[GitHub Releases](https://github.com/ingeniousfrog/Cliprove/releases)**，请根据系统选择对应资源：

| 平台 | 安装包 | 系统要求 |
|------|--------|----------|
| macOS | `.dmg` | macOS 13（Ventura）及以上，**Apple Silicon（arm64）** |
| Windows | `.exe`（NSIS）或 `.msi` | Windows 10/11，**x64** |

> **最新版本：** [v0.1.5](https://github.com/ingeniousfrog/Cliprove/releases/tag/v0.1.5)

## 安装须知

安装或向他人分发前，建议先阅读本节。

### macOS

1. 打开 `.dmg`，将 **Cliprove** 拖入 **应用程序** 文件夹。
2. 从启动台或 Spotlight 启动应用。
3. **Gatekeeper（未公证构建）：** 当前 Release 未做 Apple 公证。首次启动若被系统拦截，请 **右键 → 打开**，或在 **系统设置 → 隐私与安全性** 中点击 **仍要打开**。
4. **FFmpeg：** 应用已内置静态 FFmpeg，一般无需额外安装。可在 **设置** 中手动指定路径。Bilibili、YouTube 的音视频合并依赖 FFmpeg。

### Windows

1. 运行 `.exe` 安装程序，或通过 `.msi` 安装。
2. **SmartScreen：** 未签名构建可能触发 Windows Defender SmartScreen 提示。若安装包来自官方 [Releases](https://github.com/ingeniousfrog/Cliprove/releases) 页面，可选择 **更多信息 → 仍要运行**。
3. **FFmpeg：** 已随应用打包，首次启动后可在 **设置** 中确认状态。

### 平台登录与 Cookie

| 平台 | 是否需要登录 | 说明 |
|------|-------------|------|
| 抖音 | 需要（Cookie） | 多数分享链接需有效会话，请在 **设置 → 平台认证** 中配置。 |
| Bilibili | 建议配置 | 公开预览可不登录；高清或大会员画质需要 `SESSDATA`。 |
| YouTube | 公开内容无需登录 | 基于 `yt-dlp`；部分视频可能受 **地区限制** 或 YouTube 年龄验证影响。 |

Cookie 仅保存在本机，不会上传至 Cliprove 运营的服务器。

### 数据存储位置

| 系统 | 路径 |
|------|------|
| macOS | `~/Library/Application Support/com.heqk.cliprove/` |
| Windows | `%APPDATA%\com.heqk.cliprove\` |

该目录包含 SQLite 数据库、下载设置、平台 Cookie 与日志。卸载应用不会自动删除此目录。

### 网络与权限

- 应用仅通过 `127.0.0.1` 与本机 Python Sidecar 通信；外网请求直接发往目标平台（抖音、Bilibili、YouTube）。
- 请为所选下载目录授予读写权限。
- 剪贴板自动检测为可选功能，需在系统中授予相应权限。

### 使用前建议

- 预留足够磁盘空间存放视频与合并后的文件。
- 建议使用稳定网络；大文件下载与 Bilibili 合并对网络中断较敏感。
- 安装后遇到问题，请参阅 [故障排查](./docs/troubleshooting.md)。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri v2、Rust |
| 界面 | React、TypeScript、Vite、Tailwind CSS |
| 存储 | SQLite（WAL） |
| 下载引擎 | Python Sidecar（FastAPI）、平台适配器 |

抖音下载基于 [douyin-downloader](https://github.com/jiji262/douyin-downloader) 引擎（git submodule）；Bilibili 使用 `yt-dlp` 与 `bilibili-api-python`；YouTube 使用 `yt-dlp`。

## 开发

### 环境要求

| 工具 | 版本 | 说明 |
|------|------|------|
| macOS | 13+ | 推荐 Apple Silicon |
| Node.js | 20+ | 前端工具链 |
| Rust | stable | 通过 [rustup](https://rustup.rs) 安装 |
| Python | 3.11+ | 仅开发时需要；发布包已内置 |
| FFmpeg | 最新 | `brew install ffmpeg` — Bilibili/YouTube 合并及部分高清流需要 |

### 快速开始

```bash
git clone https://github.com/ingeniousfrog/Cliprove.git
cd Cliprove
git submodule update --init --recursive
chmod +x scripts/dev.sh
./scripts/dev.sh
```

详细说明见 [docs/development.md](./docs/development.md)。

### 本地打包

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

安装包输出至 `src-tauri/target/release/bundle/dmg/`。推送 `v*` 标签后，GitHub Actions 会自动构建 macOS 与 Windows 产物并发布到 Releases。详见 [docs/packaging.md](./docs/packaging.md)。

## 目录结构

```
src/           React 前端
src-tauri/     Rust 核心 — 数据库、任务队列、Tauri 命令
sidecar/       Python 引擎服务
engines/       平台引擎（git submodule）
scripts/       开发与打包脚本
docs/          开发、打包与故障排查文档
```

## 文档

- [开发指南](./docs/development.md)
- [打包指南](./docs/packaging.md)
- [故障排查](./docs/troubleshooting.md)

## 许可证

MIT — 见 [LICENSE](./LICENSE)。
