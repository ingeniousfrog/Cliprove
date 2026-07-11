# Cliprove MVP 实施计划

> 基于 [GOAL.md](./GOAL.md) 与上游仓库调研（2026-07-11）编写。  
> 当前仓库状态：仅含 Initial commit，尚无应用代码。

---

## 1. 项目现状

| 项目 | 状态 |
|------|------|
| Cliprove 仓库 | 空壳，仅 README / LICENSE / GOAL.md |
| 目标平台 | macOS Apple Silicon（首期），架构预留 Windows |
| 首期平台 | 抖音（Douyin）、Bilibili |
| 非目标 | LLM、云账户、移动端、大规模定时爬取 |

---

## 2. 上游仓库调研摘要

### 2.1 抖音引擎：[jiji262/douyin-downloader](https://github.com/jiji262/douyin-downloader)

| 维度 | 结论 |
|------|------|
| 许可证 | MIT，可复用与改编 |
| 语言 / 运行时 | Python 3.8+，async（aiohttp） |
| 最近活跃 | 2026-07-03 仍有推送，v2.0 稳定可用 |
| 模块结构 | `core/`（解析、下载、API）、`auth/`（Cookie）、`control/`（重试/并发/限速）、`storage/`（文件与 SQLite 去重）、`server/`（REST） |
| MVP 可用能力 | 短链解析、视频/图文、关键词搜索（`--search`）、无水印源优选、封面/音频/元数据、重试、Cookie、去重 |
| REST API（现有） | `POST /api/v1/download`、`GET /api/v1/jobs/{id}`、`GET /api/v1/jobs`、`GET /api/v1/health` |
| REST API 缺口 | **无** parse/preview、**无** search、**无** 细粒度进度事件（仅有 job 级计数） |
| 搜索实现 | `core/discovery.py` → `api_client.search_aweme()`，CLI 导出 JSONL |
| 去重数据库 | 引擎自有 `dy_downloader.db`，**与 Cliprove 应用库分离** |
| 作者桌面版 | Douzy（内测），**不复制其 UI**，仅复用引擎 |

**集成策略**：以 **Git Submodule** 或 **vendor 目录** 引入 `engines/douyin-downloader`，在其之上构建 **Cliprove Python Sidecar**，扩展 REST/JSON-RPC 接口以补齐 parse、search、preview、结构化进度事件。不直接依赖其 CLI 或 Douzy 界面。

### 2.2 Bilibili 方案调研

| 方案 | 优势 | 风险 / 注意 |
|------|------|-------------|
| **yt-dlp**（推荐用于下载） | 活跃维护（2026.07）、原生支持 BV/AV、多清晰度、字幕、分 P 合并（FFmpeg） | GPL/Unlicense 生态，需捆绑二进制；搜索能力弱 |
| **bilibili-api-python**（推荐用于搜索） | 异步 API、关键词搜索、分 P 元数据、字幕列表 | **GPL-3.0**，分发需合规；反爬策略变化快 |
| 自研解析 | 完全可控 | MVP 成本过高，违背 GOAL 原则 |

**集成策略**：下载走 **yt-dlp Python API 或 CLI**；搜索与元数据预览走 **bilibili-api-python**。两者均在同一 Python Sidecar 的 `bilibili` 模块中实现，由 Rust 统一调度。

### 2.3 FFmpeg

- Bilibili 多分 P / 音视频分离流需 FFmpeg 合并。
- 抖音 HLS 直播源（MVP 非目标）亦需 FFmpeg；普通视频通常不需要。
- 设置页提供路径检测与版本校验；打包时可选内置静态 FFmpeg（macOS arm64）。

---

## 3. 架构决策（ADR 摘要）

### ADR-001：四层分离架构

```
┌─────────────────────────────────────────────────────────────┐
│  Presentation   React + TypeScript + Vite                   │
│  (Home / Search / Tasks / Library / Settings)                 │
└──────────────────────────┬──────────────────────────────────┘
                           │ Tauri invoke + events
┌──────────────────────────▼──────────────────────────────────┐
│  Application    Rust (Tauri v2)                             │
│  - 任务队列与状态机                                          │
│  - SQLite (sqlx / rusqlite)                                 │
│  - 文件布局与去重                                            │
│  - Sidecar 生命周期                                          │
│  - FFmpeg 子进程                                             │
│  - 结构化错误映射                                            │
└──────────────────────────┬──────────────────────────────────┘
                           │ localhost HTTP (JSON)
┌──────────────────────────▼──────────────────────────────────┐
│  Engine Sidecar Python 3.11+ (FastAPI + uvicorn)            │
│  - /douyin/*  → douyin-downloader core                      │
│  - /bilibili/* → yt-dlp + bilibili-api-python               │
│  - 归一化为共享 DTO                                          │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│  Upstream     douyin-downloader / yt-dlp / bilibili-api     │
└─────────────────────────────────────────────────────────────┘
```

**理由**：GOAL 明确要求 Tauri + React + Rust + Python sidecar；平台逻辑隔离在引擎层，UI 不接触原始 API 响应。

---

### ADR-002：平台适配器双端契约

| 层级 | 职责 |
|------|------|
| **TS `PlatformAdapter`**（`src/adapters/`） | UI 侧：平台检测、展示过滤器能力、调用 Tauri command |
| **Rust `PlatformAdapter` trait**（`src-tauri/src/adapters/`） | 编排：调用 Sidecar、映射错误、写入 DB、发射进度事件 |
| **Python 平台模块**（`sidecar/platforms/`） | 引擎：解析、搜索、生成 `DownloadSpec` |

共享模型（TS / Rust / Python 各一份，字段对齐）：

- `Platform`, `MediaItem`, `Author`, `SearchQuery`, `SearchPage`
- `ParsedMedia`, `DownloadTask`, `DownloadAsset`, `DownloadProgress`
- `LibraryItem`, `AuthenticationProfile`, `StructuredError`

**理由**：GOAL 指定适配器接口；双端契约保证 UI 与 Rust 核心解耦，Python 仅负责引擎能力。

---

### ADR-003：Cliprove 独立数据库，不复用引擎 SQLite

- 应用库：`~/Library/Application Support/Cliprove/cliprove.db`（macOS）
- 引擎临时配置：Sidecar 工作目录下的 `config.yml` + `.cookies.json`
- **禁用** douyin-downloader REST 模式下的 `database`（避免双去重源）
- 去重语义键：`(platform, platform_item_id)`，由 Rust 在入队前查询

**理由**：GOAL 要求独立应用数据模型，不绑定任何下载引擎 schema。

---

### ADR-004：任务状态机与持久化

```
pending → parsing → queued → downloading → post_processing → completed
                    ↘ failed (可 retry → queued)
                    ↘ cancelled
```

- 所有状态转换写入 `download_tasks` 表（含 `stage`, `progress`, `speed_bps`, `retry_count`, `error_json`）
- 应用崩溃后：启动时扫描 `downloading` / `post_processing` → 标记为 `interrupted`，用户可一键重试
- 任务执行在 Rust 后台 async runtime，**不绑定 UI 页面生命周期**
- 进度通过 Tauri `emit` 推送到前端；前端用 Zustand 或 TanStack Query 缓存

---

### ADR-005：Sidecar 通信协议

**传输**：`127.0.0.1:{dynamic_port}` HTTP/JSON（开发期固定 18765，生产由 Rust 分配空闲端口）。

**核心端点（Cliprove Sidecar，扩展上游）**：

| Method | Path | 说明 |
|--------|------|------|
| GET | `/health` | 健康检查 |
| POST | `/v1/parse` | `{ platform?, url }` → `ParsedMedia` |
| POST | `/v1/search` | `{ platform, query, cursor? }` → `SearchPage` |
| POST | `/v1/download/plan` | `{ item, options }` → `DownloadSpec`（URL 列表 + 后处理指令） |
| POST | `/v1/download/execute` | 流式或轮询式执行单个 asset 下载（供 Rust 拉取字节或落盘到指定路径） |
| POST | `/v1/auth/validate` | 校验 Cookie 有效性 |
| POST | `/v1/auth/cookies` | 更新平台 Cookie（仅本地文件） |

**理由**：上游 REST 仅支持整 URL 下载 job，无法满足「先预览再选择资源」与「搜索分页」；扩展 Sidecar 比 fork 上游 server 更清晰。

---

### ADR-006：存储布局

```
{download_root}/
├── douyin/{author_id}/{media_id}/
│   ├── video.mp4
│   ├── cover.jpg
│   ├── audio.mp3          # 可选
│   └── metadata.json
├── bilibili/{author_id}/{media_id}/
│   ├── video.mp4          # 或 part-001.mp4, part-002.mp4
│   ├── cover.jpg
│   ├── subtitles/
│   └── metadata.json
├── thumbnails/            # 列表缩略图缓存
├── metadata/              # 导出用
└── exports/
```

- 目录名使用 **平台稳定 ID**，不用标题
- 文件名模板在设置中可配置（如 `{platform}_{author}_{title}_{id}`），渲染时做文件名消毒

---

### ADR-007：结构化错误体系

错误码枚举（Rust `enum` + TS 常量 + Python 字符串对齐）：

`unsupported_link` · `unsupported_content_type` · `auth_required` · `auth_expired` · `verification_required` · `rate_limited` · `platform_changed` · `content_unavailable` · `private_content` · `region_restricted` · `media_url_expired` · `network_timeout` · `download_incomplete` · `ffmpeg_unavailable` · `disk_full` · `permission_denied` · `engine_failure` · `unknown`

- 用户可见：`message` + `suggestion`（可操作文案）
- 日志保留：`technical_detail`、`engine_trace`（不直接展示原始 traceback）

---

### ADR-008：UI 技术选型

| 类别 | 选择 | 理由 |
|------|------|------|
| 路由 | React Router v7 | 五页面清晰分区 |
| 状态 | Zustand（UI）+ TanStack Query（服务端状态） | 轻量，适合桌面 |
| 组件 | shadcn/ui + Tailwind CSS | 浅色、紧凑、可定制；避免厚重 UI 库 |
| 虚拟列表 | `@tanstack/react-virtual` | 搜索结果大数据集 |
| 表格 | TanStack Table | 库/任务列表 |
| 图标 | Lucide | 与 shadcn 生态一致 |

设计原则：浅色、信息密度适中、可调整面板、原生 macOS 菜单/快捷键；**不做**暗色赛博风、不做 AI 装饰、不做移动端拉伸布局。

---

### ADR-009：依赖引入方式

| 依赖 | 方式 |
|------|------|
| douyin-downloader | `engines/douyin-downloader` git submodule，sidecar `requirements.txt` 以 editable install 或 `PYTHONPATH` 引用 |
| yt-dlp | PyPI 固定版本 + 可选捆绑 `yt-dlp_macos` 二进制 |
| bilibili-api-python | PyPI 固定版本；**注意 GPL-3.0**，在 LICENSE/NOTICE 中声明 |
| FFmpeg | 系统 PATH 优先；设置页手动指定；打包可选 vendor |
| Python 运行时 | 开发期系统 Python 3.11+；打包用 PyInstaller 或 `python-build-standalone` 嵌入 |

---

### ADR-010：安全与隐私

- Cookie / Token 仅存本地：`Application Support/Cliprove/credentials/`（文件权限 600）
- Sidecar 仅监听 `127.0.0.1`
- 无遥测、无云同步、无账户系统
- 日志脱敏：输出前剥离 Cookie 字段

---

## 4. 仓库目录结构（目标）

```
Cliprove/
├── GOAL.md
├── IMPLEMENTATION_PLAN.md
├── README.md
├── src/                          # React 前端
│   ├── adapters/                 # TS 适配器接口 + mock
│   ├── components/
│   ├── pages/                    # home, search, tasks, library, settings
│   ├── stores/
│   ├── types/                    # 共享模型
│   └── lib/
├── src-tauri/                    # Rust 核心
│   ├── src/
│   │   ├── adapters/
│   │   ├── commands/
│   │   ├── db/                   # migrations, models, repos
│   │   ├── tasks/                # 队列、执行器、恢复
│   │   ├── sidecar/              # 进程管理、HTTP 客户端
│   │   ├── errors/
│   │   └── ffmpeg/
│   ├── migrations/
│   └── tauri.conf.json
├── sidecar/                      # Python 引擎服务
│   ├── app.py
│   ├── platforms/
│   │   ├── douyin/
│   │   └── bilibili/
│   ├── models/                   # Pydantic DTO
│   └── requirements.txt
├── engines/
│   └── douyin-downloader/        # git submodule
├── docs/
│   ├── development.md
│   ├── packaging.md
│   └── troubleshooting.md
└── scripts/
    ├── dev.sh
    └── bundle-python.sh
```

---

## 5. 数据库 Schema（MVP）

### `library_items`

| 列 | 类型 | 说明 |
|----|------|------|
| id | TEXT PK | UUID |
| platform | TEXT | douyin / bilibili |
| platform_item_id | TEXT | aweme_id / bvid |
| original_url | TEXT | |
| canonical_url | TEXT | |
| title | TEXT | |
| description | TEXT | |
| author_id | TEXT | |
| author_name | TEXT | |
| published_at | INTEGER | Unix ms |
| media_type | TEXT | video / image_post / audio / multipart |
| duration_sec | INTEGER | |
| cover_path | TEXT | |
| media_paths | TEXT | JSON 数组 |
| metadata_path | TEXT | |
| subtitle_paths | TEXT | JSON 数组 |
| file_size | INTEGER | |
| checksum | TEXT | SHA256，可选 |
| search_keyword | TEXT | 发现来源关键词 |
| created_at | INTEGER | |
| updated_at | INTEGER | |

**唯一索引**：`(platform, platform_item_id)`

### `download_tasks`

| 列 | 类型 | 说明 |
|----|------|------|
| id | TEXT PK | |
| library_item_id | TEXT FK nullable | 完成后关联 |
| platform | TEXT | |
| platform_item_id | TEXT | |
| status | TEXT | 状态机 |
| stage | TEXT | 当前阶段文案 |
| progress | REAL | 0–1 |
| speed_bps | INTEGER | |
| retry_count | INTEGER | |
| options_json | TEXT | 用户选择的资源项 |
| error_json | TEXT | StructuredError |
| output_dir | TEXT | |
| created_at / updated_at / completed_at | INTEGER | |

### `tags`, `collections`, `collection_items`, `settings`

按 GOAL 库管理需求在 Phase 4 完善；Phase 0 仅 `settings` key-value 表。

---

## 6. 分阶段实施计划

### Phase 0：仓库基础（预计 3–5 天）

**目标**：可启动的空壳应用 + Mock 流程跑通。

| # | 任务 | 产出 |
|---|------|------|
| 0.1 | `npm create tauri-app` 初始化 Tauri v2 + React + TS + Vite | 可编译运行 |
| 0.2 | 定义 `src/types/` 共享模型与 `PlatformAdapter` 接口 | 类型文件 |
| 0.3 | Rust：`db` 模块 + SQLite migrations | `cliprove.db` 初始化 |
| 0.4 | Rust：settings CRUD commands | 设置读写 |
| 0.5 | Rust：结构化错误 + tracing 日志 | 错误枚举、日志文件 |
| 0.6 | Mock 适配器（douyin/bilibili） | 固定假数据 |
| 0.7 | 五页面骨架 + 侧边导航 | 路由可切换 |
| 0.8 | Sidecar 空壳 FastAPI `/health` | Rust 可拉起进程 |

**验收**：应用启动；Mock 解析链接 → 预览 → 入队 → 库列表可见假数据；设置可保存；重启后设置仍在。

---

### Phase 1：抖音链接工作流（预计 5–7 天）

| # | 任务 | 产出 |
|---|------|------|
| 1.1 | 添加 `engines/douyin-downloader` submodule | 引擎源码 |
| 1.2 | Sidecar `/v1/parse`：短链 + 视频 + 图文 | `ParsedMedia` |
| 1.3 | Sidecar `/v1/download/plan` + asset 下载 | 落盘到 Cliprove 目录结构 |
| 1.4 | Rust 任务执行器 + 进度事件 | Tasks 页实时更新 |
| 1.5 | 去重：入队前查 `(platform, id)` | 重复提示 / 强制覆盖确认 |
| 1.6 | Home 页：粘贴链接、预览、资源勾选、下载 | 端到端可用 |
| 1.7 | Cookie 设置与 `validateAuth` | Settings 页 |
| 1.8 | 重试与失败展示 | 结构化错误 |

**验收**：真实抖音链接 → 预览 → 下载 → 库中可见；重复下载被拦截；进度可见；失败可重试。

---

### Phase 2：抖音关键词搜索（预计 3–4 天）

| # | 任务 | 产出 |
|---|------|------|
| 2.1 | Sidecar `/v1/search` 封装 `discovery.search_and_dump` 逻辑 | 分页 `SearchPage` |
| 2.2 | 归一化 aweme → `MediaItem` | 统一封面/作者/时长 |
| 2.3 | Search 页：平台选择、关键词、网格/表格切换 | UI |
| 2.4 | 虚拟滚动 + 多选 + 批量入队 | 性能与交互 |
| 2.5 | 记录 `search_keyword` 到库项 | 溯源 |

**验收**：关键词搜索 → 选多条 → 批量下载 → 库中带来源关键词。

---

### Phase 3：Bilibili 支持（预计 5–7 天）

| # | 任务 | 产出 |
|---|------|------|
| 3.1 | Sidecar `bilibili` 模块：BV/AV/URL 解析 | `ParsedMedia` |
| 3.2 | bilibili-api 关键词搜索 + 分页 | `SearchPage` |
| 3.3 | yt-dlp 下载 + 清晰度选择 | `DownloadSpec` |
| 3.4 | 多分 P 支持（列表展示 + 选择性下载） | 多 asset |
| 3.5 | 字幕下载（可用时） | subtitle_paths |
| 3.6 | Cookie 配置（高清/会员画质） | Settings |
| 3.7 | FFmpeg 合并（音视频分离流） | post_processing 阶段 |

**验收**：Bilibili 链接与搜索全流程与抖音对等；分 P 与字幕可用；清晰度可选。

---

### Phase 4：库管理与可靠性（预计 4–5 天）

| # | 任务 | 产出 |
|---|------|------|
| 4.1 | Library 全文搜索（SQLite FTS5） | 标题/作者/标签/id |
| 4.2 | 过滤器：平台、类型、日期 | UI |
| 4.3 | 标签与收藏夹 | CRUD |
| 4.4 | 任务恢复：启动扫描 interrupted 任务 | 恢复 UX |
| 4.5 | 打开文件 / Finder 中显示 / 复制链接 | 系统集成 |
| 4.6 | 删除记录 + 可选删文件（二次确认） | 安全删除 |
| 4.7 | 剪贴板链接检测（可选，稳定性评估后） | Home 增强 |
| 4.8 | 集成测试 + E2E（Playwright 或 Tauri WebDriver） | CI |

**验收**：GOAL 库管理条目齐备；杀进程重启后任务状态清晰可恢复。

---

### Phase 5：打包与文档（预计 3–4 天）

| # | 任务 | 产出 |
|---|------|------|
| 5.1 | Python sidecar 打包脚本（PyInstaller / standalone） | 内嵌运行时 |
| 5.2 | FFmpeg 检测与可选捆绑 | 设置页验证 |
| 5.3 | `tauri build` macOS arm64 `.dmg` | 可安装包 |
| 5.4 | `docs/development.md` | 开发指南 |
| 5.5 | `docs/packaging.md` | 打包与引擎更新 |
| 5.6 | `docs/troubleshooting.md` | 常见问题 |

**验收**：全新机器（仅安装 DMG）可完成 GOAL 全部 Acceptance Criteria。

---

## 7. 关键接口草案

### TypeScript `PlatformAdapter`

```typescript
interface PlatformAdapter {
  id: string;
  name: string;
  supportedFilters: SearchFilterKey[];
  canHandle(input: string): boolean;
  parse(input: string): Promise<ParsedMedia>;
  search(query: SearchQuery, cursor?: string): Promise<SearchPage>;
  createDownloadSpec(
    item: MediaItem,
    options: DownloadOptions
  ): Promise<DownloadSpec>;
  validateAuth(): Promise<AuthStatus>;
}
```

> 注：实际实现为 **调用 Tauri command**，由 Rust 转发至 Sidecar；TS 适配器是薄封装，不含平台逻辑。

### Tauri Commands（Rust 暴露）

- `parse_link(url)` → `ParsedMedia`
- `search_media(platform, query, cursor?)` → `SearchPage`
- `enqueue_download(item, options)` → `task_id`
- `list_tasks(filter?)` → `DownloadTask[]`
- `task_action(task_id, action)` → `pause | resume | retry | cancel`
- `list_library(query?)` → `LibraryItem[]`
- `get_settings / update_settings`
- `validate_platform_auth(platform)`
- `reveal_in_finder(path)`

### 进度事件

```typescript
// event: "download-progress"
interface DownloadProgressEvent {
  taskId: string;
  stage: string;
  progress: number;      // 0-1
  speedBps?: number;
  retryCount: number;
}
```

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 抖音 API / Cookie 策略变更 | 解析或搜索失败 | 隔离在 sidecar；结构化错误 `platform_changed`；引擎 submodule 可快速跟进上游 |
| 上游 REST 能力不足 | 阻塞预览/搜索 | 自建 Sidecar 扩展端点，直接 import `core/*` |
| bilibili-api GPL-3.0 | 许可证传染 | NOTICE 声明；评估是否仅用 yt-dlp（搜索功能会受限） |
| Python 打包体积大 | DMG 体积 | 最小化依赖；可选首次启动下载引擎包 |
| 剪贴板监听稳定性 | 崩溃 | Phase 4 可选；默认关闭，设置中启用 |
| 平台水印策略误解 | 产品预期偏差 | UI 文案明确「优选无平台水印源」，不做像素级去除 |

---

## 9. 验收清单（对应 GOAL）

- [ ] Apple Silicon macOS 启动成功
- [ ] 抖音：粘贴链接 → 预览 → 下载 → 库
- [ ] 抖音：关键词搜索 → 多选 → 批量下载
- [ ] Bilibili：链接全流程
- [ ] Bilibili：搜索 + 批量下载
- [ ] 任务进度与结构化错误可见
- [ ] 重启保留任务、设置、库
- [ ] 重复下载拦截或确认覆盖
- [ ] 平台代码隔离在适配器/引擎
- [ ] 无 LLM / 云账户 / 远程后端
- [ ] 凭证与内容仅本地
- [ ] 文档齐全

---

## 10. 建议的首次提交顺序

1. Phase 0 脚手架（Tauri + 类型 + DB + Mock + 页面骨架）
2. `engines/douyin-downloader` submodule + sidecar 骨架
3. 按 Phase 1 → 5 迭代，**每阶段验收后再进入下一阶段**

---

## 附录 A：上游 douyin-downloader 模块映射

| 上游模块 | Cliprove 用途 |
|----------|---------------|
| `core/url_parser.py` | `/v1/parse` 路由 |
| `core/api_client.py` | 元数据拉取、搜索 |
| `core/downloader_factory.py` + `*_downloader.py` | `/v1/download/*` |
| `core/discovery.py` | `/v1/search` |
| `core/metadata.py` | aweme → `MediaItem` 字段提取 |
| `auth/CookieManager` | Cookie 持久化 |
| `control/*` | 重试、限速、并发（Sidecar 内复用） |
| `server/app.py` | 参考，不直接依赖；Cliprove 自建 `sidecar/app.py` |

## 附录 B：Bilibili 字段映射（yt-dlp → MediaItem）

| yt-dlp 字段 | MediaItem |
|-------------|-----------|
| `id` | `platformItemId` |
| `title` | `title` |
| `description` | `description` |
| `uploader` / `uploader_id` | `author.name` / `author.id` |
| `duration` | `durationSec` |
| `thumbnails[0].url` | `coverUrl` |
| `webpage_url` | `canonicalUrl` |
| `formats` | `DownloadSpec.qualities` |

---

*文档版本：1.0 · 2026-07-11*
