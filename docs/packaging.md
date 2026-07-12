# Cliprove 打包指南

## 概述

发布构建包含三步：

1. 用 PyInstaller 将 Python sidecar 打成独立可执行文件
2. 将 sidecar 二进制放入 `src-tauri/binaries/`（Tauri `externalBin`）
3. 运行 `tauri build` 生成 macOS `.dmg`

## 一键打包

本地：

```bash
chmod +x scripts/build.sh scripts/build-sidecar.sh
./scripts/build.sh
```

GitHub Actions（推荐发布）：

1. 将 `src-tauri/tauri.conf.json` 中的 `version` 更新为目标版本
2. 提交并打 tag，例如 `v0.1.0`
3. 推送 tag：`git push origin v0.1.0`
4. [Release 工作流](https://github.com/ingeniousfrog/Cliprove/actions/workflows/release.yml) 会自动构建 macOS `.dmg` 并创建 GitHub Release

也可在 Actions 页手动触发 **Release** 工作流，仅上传构建产物（不创建 Release）。

CI 工作流会在 push / PR 时运行前端构建、Rust 测试与 Sidecar 测试。

产物目录：

```
src-tauri/target/release/bundle/dmg/Cliprove_0.1.0_aarch64.dmg
```

## 分步打包

### 1. 构建 Sidecar

```bash
./scripts/build-sidecar.sh
```

脚本会：

- 初始化 `engines/douyin-downloader` submodule
- 创建/更新 `sidecar/.venv`
- 安装 sidecar 与引擎依赖
- 运行 PyInstaller（`sidecar/cliprove-sidecar.spec`）
- 输出到 `src-tauri/binaries/cliprove-sidecar-<target-triple>`

### 2. 构建 Tauri 应用

```bash
npm install
npm run tauri build
```

`tauri.conf.json` 已将 `binaries/cliprove-sidecar` 配置为 `externalBin`，打包时会与主程序一并放入 `.app/Contents/MacOS/`。

## FFmpeg

发布构建会通过 `scripts/fetch-ffmpeg.sh` 下载 macOS static FFmpeg 并打入 `.app/Contents/Resources/ffmpeg/`。

检测优先级：

1. 用户在设置中手动指定的路径
2. 系统 PATH / Homebrew 常见路径
3. App 内置 FFmpeg（fallback）

开发环境仍优先使用本机已安装的 FFmpeg；若未执行 fetch 脚本，行为与旧版一致。

```bash
chmod +x scripts/fetch-ffmpeg.sh
./scripts/fetch-ffmpeg.sh
```

也可手动安装系统 FFmpeg：

```bash
brew install ffmpeg
```

Bilibili 音视频合并、部分高清流依赖 FFmpeg。内置 FFmpeg 基于 LGPL 许可，分发时需保留许可说明；源码可从 [FFmpeg 官网](https://ffmpeg.org/download.html) 获取。

打包时 `codesign` 需覆盖 `Resources/ffmpeg/` 下的二进制（与 sidecar 同级处理）。

## 引擎更新

### 更新 douyin-downloader submodule

```bash
cd engines/douyin-downloader
git fetch origin
git checkout <tag-or-commit>
cd ../..
git add engines/douyin-downloader
```

更新后需重新运行 `./scripts/build-sidecar.sh` 再打包应用。

### 更新 Python 依赖

修改 `sidecar/requirements.txt` 后重新构建 sidecar。

## 签名与公证（可选）

当前仓库未内置 Apple 签名配置。若要分发到公网，需：

1. 配置 Apple Developer 证书
2. 在 `tauri.conf.json` 添加签名身份
3. 对 `.app` 与 sidecar 二进制执行 `codesign`
4. 提交公证（`notarytool`）

详见 [Tauri macOS 代码签名文档](https://v2.tauri.app/distribute/sign/macos/)。

## 常见问题

- **Sidecar 启动失败**：确认 `src-tauri/binaries/cliprove-sidecar-*` 存在且可执行
- **抖音功能不可用**：确认 submodule 已初始化且已打入 PyInstaller `datas`
- **DMG 体积较大**：PyInstaller onefile 会包含 Python 运行时与依赖，属正常现象
