# Cliprove 故障排查

## 应用无法启动

1. 从终端启动查看日志：
   ```bash
   /Applications/Cliprove.app/Contents/MacOS/cliprove
   ```
2. 检查日志目录：`~/Library/Application Support/Cliprove/logs/`
3. 若数据库损坏，可备份后删除：
   ```bash
   rm ~/Library/Application\ Support/Cliprove/cliprove.db
   ```

## Sidecar / 引擎问题

### 症状：设置页 Sidecar 状态非 `ok`

**开发环境**

```bash
sidecar/.venv/bin/pip install -r sidecar/requirements.txt
sidecar/.venv/bin/pip install -r engines/douyin-downloader/requirements.txt
git submodule update --init --recursive
```

**安装包环境**

- 确认 `.app/Contents/MacOS/cliprove-sidecar` 存在
- 重新运行 `./scripts/build-sidecar.sh` 后打包

### 症状：抖音链接解析失败

- 检查 Cookie 是否过期（设置 → 验证抖音）
- 确认 `engines/douyin-downloader` submodule 已初始化
- 查看 sidecar  stderr 或应用日志

### 症状：Bilibili 登录失败（SSL / `CERTIFICATE_VERIFY_FAILED`）

安装包内 Sidecar 访问 `api.bilibili.com` 时若出现 `SSLCertVerificationError` / `unable to get local issuer certificate`：

- 多为打包后的 OpenSSL 未挂上 CA 证书包（`certifi`）导致，请升级至 **v0.1.6+**
- 开发环境一般不受影响（使用本机 Python / Homebrew CA）
- 若自建包仍报错：确认 `sidecar/hooks/pyi_rth_ssl_certifi.py` 已编入，并重新执行 `./scripts/build-sidecar.sh`
- 升级后仍异常：可能是旧版 Sidecar 进程仍占用 `18765` 端口。退出 App 后执行：
  ```bash
  lsof -tiTCP:18765 -sTCP:LISTEN | xargs kill
  ```
  再重新打开 Cliprove

### 症状：Bilibili 下载失败 / 无法合并

- 设置页验证 FFmpeg 路径
- 检查 Bilibili Cookie（`SESSDATA`）
- 高清/会员画质需要有效 Cookie

## 下载与任务

| 错误 | 可能原因 | 处理 |
|------|----------|------|
| 该内容已在本地库中 | 重复下载 | 库中删除记录或使用强制覆盖 |
| 下载完成但缺少结果 | Sidecar 进程中断 | 任务页恢复中断任务 |
| engine_failure | Cookie/网络/引擎异常 | 检查认证与网络，重试任务 |
| ffmpeg_unavailable | FFmpeg 未安装或路径错误 | `brew install ffmpeg` 并验证路径 |
| disk_full | 磁盘空间不足 | 清理磁盘或更换下载目录 |

### 中断任务恢复

应用异常退出后，未完成任务会标记为「已中断」。打开任务中心，点击「恢复下载」。

## 库管理

### 全文搜索无结果

- 确认条目已下载完成并写入库
- 标签变更后会重建 FTS 索引

### 无法打开文件 / Finder 定位失败

- 文件可能已被手动删除
- 检查下载目录是否被移动

## 网络与权限

- 应用仅需本地回环访问 Sidecar（`127.0.0.1:18765`）
- 剪贴板自动检测需在设置中开启，且需授予剪贴板权限
- 下载目录需有读写权限

## 获取帮助

提交 Issue 时请附上：

- macOS 版本与芯片架构（arm64/x64）
- 应用版本（设置 → Sidecar 版本号）
- 相关日志片段（勿包含 Cookie 等敏感信息）
- 复现步骤与平台（抖音/Bilibili）
