# Cliprove 开发指南

## 环境要求

- macOS 13+（Apple Silicon 推荐）
- Node.js 20+
- Rust stable（`rustup default stable`）
- Python 3.11+
- FFmpeg（`brew install ffmpeg`）
- Git（含 submodule 支持）

## 首次克隆

```bash
git clone https://github.com/ingeniousfrog/Cliprove.git
cd Cliprove
git submodule update --init --recursive
```

## 日常开发

推荐使用一键脚本：

```bash
chmod +x scripts/dev.sh
./scripts/dev.sh
```

或手动步骤：

```bash
npm install
python3 -m venv sidecar/.venv
sidecar/.venv/bin/pip install -r sidecar/requirements.txt
sidecar/.venv/bin/pip install -r engines/douyin-downloader/requirements.txt
npm run tauri dev
```

## 项目结构

| 目录 | 说明 |
|------|------|
| `src/` | React 前端 |
| `src-tauri/` | Tauri + Rust 核心（数据库、任务、命令） |
| `sidecar/` | Python FastAPI 引擎服务 |
| `engines/douyin-downloader` | 抖音上游引擎（git submodule） |
| `scripts/` | 开发/打包脚本 |

## Sidecar 开发

Sidecar 默认监听 `127.0.0.1:18765`，开发时由 Rust 自动拉起 `sidecar/.venv/bin/python3 sidecar/app.py`。

单独调试：

```bash
cd sidecar
../sidecar/.venv/bin/python3 app.py --port 18765
curl http://127.0.0.1:18765/health
```

## Rust 与前端

```bash
# 前端类型检查 + 构建
npm run build

# Rust 检查
cd src-tauri && cargo check

# 集成测试
cd src-tauri && cargo test
```

## 本地数据位置

| 类型 | 默认路径 |
|------|----------|
| SQLite 数据库 | `~/Library/Application Support/Cliprove/cliprove.db` |
| 应用日志 | `~/Library/Application Support/Cliprove/logs/` |
| 下载目录 | 设置页可配置，默认 `~/Downloads/Cliprove Library` |

## 规划文档

`GOAL.md` 与 `IMPLEMENTATION_PLAN.md` 为本地规划文件，未纳入 Git 仓库。
