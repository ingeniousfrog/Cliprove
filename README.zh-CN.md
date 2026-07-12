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

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri v2、Rust |
| 界面 | React、TypeScript、Vite、Tailwind CSS |
| 存储 | SQLite（WAL） |
| 下载引擎 | Python Sidecar（FastAPI）、平台适配器 |

抖音下载基于 [douyin-downloader](https://github.com/jiji262/douyin-downloader) 引擎（git submodule）；Bilibili 使用 `yt-dlp` 与 `bilibili-api-python`；YouTube 使用 `yt-dlp`。

## 环境要求

| 工具 | 版本 | 说明 |
|------|------|------|
| macOS | 13+ | 推荐 Apple Silicon |
| Node.js | 20+ | 前端工具链 |
| Rust | stable | 通过 [rustup](https://rustup.rs) 安装 |
| Python | 3.11+ | 仅开发时需要；发布包已内置 |
| FFmpeg | 最新 | `brew install ffmpeg` — Bilibili/YouTube 合并及部分高清流需要 |

## 快速开始

```bash
git clone https://github.com/ingeniousfrog/Cliprove.git
cd Cliprove
git submodule update --init --recursive
chmod +x scripts/dev.sh
./scripts/dev.sh
```

详细说明见 [docs/development.md](./docs/development.md)。

## 打包发布

本地 macOS 打包：

```bash
chmod +x scripts/build.sh
./scripts/build.sh
```

**GitHub 自动发版**（推荐）：

```bash
# 1. 更新 src-tauri/tauri.conf.json 的 version
# 2. 提交后打 tag 并推送（推 main 不会发版，只会跑 CI）
git tag v0.1.3
git push origin v0.1.3
```

Release 工作流会自动构建 **macOS `.dmg`** 与 **Windows `.exe` / `.msi`**，并发布到 [GitHub Releases](https://github.com/ingeniousfrog/Cliprove/releases)。

详见 [docs/packaging.md](./docs/packaging.md)。

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
