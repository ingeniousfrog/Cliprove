use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::errors::{AppError, AppResult};
use crate::mock;
use crate::models::{
    AppSettings, AuthStatus, BatchEnqueueItemResult, BatchEnqueueResult, Collection,
    DownloadOptions, DownloadSpec, DownloadTask, FfmpegStatus, LibraryFilter, LibraryItem,
    MediaItem, ParsedMedia, PlatformLoginSession, SearchPage, SidecarHealth, Tag,
};
use crate::shell;
use crate::tasks;

fn is_douyin_url(url: &str) -> bool {
    let lowered = url.to_lowercase();
    ["douyin.com", "iesdouyin.com", "v.douyin.com"]
        .iter()
        .any(|token| lowered.contains(token))
}

fn is_bilibili_url(url: &str) -> bool {
    let lowered = url.to_lowercase();
    lowered.contains("bilibili.com")
        || lowered.contains("b23.tv")
        || lowered.starts_with("bv")
        || lowered.starts_with("av")
}

fn is_youtube_url(url: &str) -> bool {
    let lowered = url.to_lowercase();
    ["youtube.com", "youtu.be", "youtube-nocookie.com"]
        .iter()
        .any(|token| lowered.contains(token))
}

fn ensure_sidecar(state: &AppState, platform: &str) -> AppResult<()> {
    if platform == "douyin" || platform == "bilibili" || platform == "youtube" {
        state.sidecar.start()?;
    }
    Ok(())
}

fn output_dir_for_item(settings: &AppSettings, item: &MediaItem) -> String {
    format!(
        "{}/{}/{}/{}",
        settings.download_directory,
        item.platform,
        item.author.id,
        item.platform_item_id
    )
}

enum EnqueueOutcome {
    Enqueued(String),
    Skipped(String),
}

fn enqueue_one(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    item: MediaItem,
    options: &DownloadOptions,
    skip_if_in_library: bool,
    skip_if_pending: bool,
    ensure_engine: bool,
) -> AppResult<EnqueueOutcome> {
    if skip_if_in_library
        && !options.force_replace.unwrap_or(false)
        && state
            .db
            .library()
            .exists(&item.platform, &item.platform_item_id)?
    {
        return Ok(EnqueueOutcome::Skipped("已在本地库中".to_string()));
    }

    if skip_if_pending
        && state
            .db
            .tasks()
            .has_active(&item.platform, &item.platform_item_id)?
    {
        return Ok(EnqueueOutcome::Skipped("已有进行中的下载任务".to_string()));
    }

    let settings = state.db.settings().get_all()?;
    let output_dir = output_dir_for_item(&settings, &item);
    let task = state.db.tasks().insert(&item, options, &output_dir)?;

    if ensure_engine {
        ensure_sidecar(state, &item.platform)?;
    }

    tasks::spawn_task(
        app.clone(),
        Arc::clone(state),
        task.id.clone(),
        item,
        output_dir,
        options.clone(),
    );

    Ok(EnqueueOutcome::Enqueued(task.id))
}

#[tauri::command]
pub fn parse_link(state: State<Arc<AppState>>, url: String) -> Result<ParsedMedia, String> {
    run(|| {
        if is_douyin_url(&url) || is_bilibili_url(&url) || is_youtube_url(&url) {
            let settings = state.db.settings().get_all()?;
            state.sidecar.start()?;
            return state.sidecar.client()?.parse_link(&url, &settings);
        }
        mock::parse_link(&url)
    })
}

#[tauri::command]
pub fn search_media(
    state: State<Arc<AppState>>,
    platform: String,
    query: crate::models::SearchQuery,
    cursor: Option<String>,
) -> Result<SearchPage, String> {
    run(|| {
        if platform == "douyin" || platform == "bilibili" || platform == "youtube" {
            let settings = state.db.settings().get_all()?;
            ensure_sidecar(&state, &platform)?;
            return state
                .sidecar
                .client()?
                .search_media(&platform, &query, cursor.as_deref(), &settings);
        }
        mock::search(&platform, &query, cursor.as_deref())
    })
}

#[tauri::command]
pub fn create_download_spec(
    _state: State<Arc<AppState>>,
    item: MediaItem,
    options: DownloadOptions,
) -> Result<DownloadSpec, String> {
    run(|| mock::create_download_spec(&item, &options))
}

#[tauri::command]
pub fn enqueue_download(
    app: tauri::AppHandle,
    state: State<Arc<AppState>>,
    item: MediaItem,
    options: DownloadOptions,
) -> Result<String, String> {
    run(|| {
        match enqueue_one(
            &app,
            state.inner(),
            item,
            &options,
            true,
            true,
            true,
        )? {
            EnqueueOutcome::Enqueued(task_id) => Ok(task_id),
            EnqueueOutcome::Skipped(message) => Err(AppError::structured(
                "content_unavailable",
                message,
                Some("如需重新下载，请使用重试并覆盖（后续将提供显式覆盖选项）".to_string()),
            )),
        }
    })
}

#[tauri::command]
pub fn enqueue_download_batch(
    app: tauri::AppHandle,
    state: State<Arc<AppState>>,
    items: Vec<MediaItem>,
    options: DownloadOptions,
) -> Result<BatchEnqueueResult, String> {
    run(|| {
        if items.is_empty() {
            return Ok(BatchEnqueueResult {
                results: Vec::new(),
                enqueued_count: 0,
            });
        }

        if let Some(first) = items.first() {
            ensure_sidecar(&state, &first.platform)?;
        }

        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let platform_item_id = item.platform_item_id.clone();
            let outcome = enqueue_one(
                &app,
                state.inner(),
                item,
                &options,
                true,
                true,
                false,
            )?;

            let result = match outcome {
                EnqueueOutcome::Enqueued(task_id) => BatchEnqueueItemResult {
                    platform_item_id,
                    status: "enqueued".to_string(),
                    task_id: Some(task_id),
                    message: None,
                },
                EnqueueOutcome::Skipped(message) => BatchEnqueueItemResult {
                    platform_item_id,
                    status: "skipped".to_string(),
                    task_id: None,
                    message: Some(message),
                },
            };
            results.push(result);
        }

        let enqueued_count = results
            .iter()
            .filter(|result| result.status == "enqueued")
            .count();

        Ok(BatchEnqueueResult {
            results,
            enqueued_count,
        })
    })
}

#[tauri::command]
pub fn list_tasks(state: State<Arc<AppState>>) -> Result<Vec<DownloadTask>, String> {
    run(|| state.db.tasks().list())
}

#[tauri::command]
pub fn task_action(
    app: tauri::AppHandle,
    state: State<Arc<AppState>>,
    task_id: String,
    action: String,
) -> Result<(), String> {
    run(|| {
        match action.as_str() {
            "cancel" => state.db.tasks().mark_cancelled(&task_id)?,
            "delete" => state.db.tasks().delete(&task_id)?,
            "retry" | "resume" => {
                let payload = state
                    .db
                    .tasks()
                    .get_payload(&task_id)?
                    .ok_or_else(|| AppError::Message("任务不存在".to_string()))?;

                let item: MediaItem = payload
                    .item_json
                    .as_deref()
                    .and_then(|value| serde_json::from_str(value).ok())
                    .ok_or_else(|| AppError::Message("任务缺少媒体信息，无法恢复".to_string()))?;

                let options: DownloadOptions = payload
                    .options_json
                    .as_deref()
                    .and_then(|value| serde_json::from_str(value).ok())
                    .unwrap_or(DownloadOptions {
                        assets: vec![
                            "video".to_string(),
                            "cover".to_string(),
                            "metadata".to_string(),
                        ],
                        quality_id: None,
                        save_cover: Some(true),
                        save_audio: None,
                        save_metadata: Some(true),
                        save_subtitles: None,
                        force_replace: Some(true),
                    });

                let settings = state.db.settings().get_all()?;
                let output_dir = payload.output_dir.clone().unwrap_or_else(|| {
                    format!(
                        "{}/{}/{}/{}",
                        settings.download_directory,
                        item.platform,
                        item.author.id,
                        item.platform_item_id
                    )
                });

                if action == "retry" {
                    state.db.tasks().mark_retry(&task_id)?;
                } else {
                    state.db.tasks().mark_resume(&task_id)?;
                }

                ensure_sidecar(&state, &item.platform)?;
                tasks::spawn_task(
                    app,
                    Arc::clone(state.inner()),
                    task_id,
                    item,
                    output_dir,
                    options,
                );
            }
            "pause" => {}
            _ => {
                return Err(AppError::Message(format!("unsupported action: {action}")));
            }
        }
        Ok(())
    })
}

#[tauri::command]
pub fn list_library(
    state: State<Arc<AppState>>,
    filter: Option<LibraryFilter>,
) -> Result<Vec<LibraryItem>, String> {
    run(|| state.db.library().list(&filter.unwrap_or_default()))
}

#[tauri::command]
pub fn get_library_item(state: State<Arc<AppState>>, id: String) -> Result<LibraryItem, String> {
    run(|| {
        state
            .db
            .library()
            .get(&id)?
            .ok_or_else(|| AppError::Message("库条目不存在".to_string()))
    })
}

#[tauri::command]
pub fn delete_library_item(
    state: State<Arc<AppState>>,
    id: String,
    delete_files: bool,
) -> Result<(), String> {
    run(|| state.db.library().delete(&id, delete_files))
}

#[tauri::command]
pub fn list_tags(state: State<Arc<AppState>>) -> Result<Vec<Tag>, String> {
    run(|| state.db.tags().list())
}

#[tauri::command]
pub fn create_tag(state: State<Arc<AppState>>, name: String) -> Result<Tag, String> {
    run(|| state.db.tags().create(&name))
}

#[tauri::command]
pub fn delete_tag(state: State<Arc<AppState>>, id: String) -> Result<(), String> {
    run(|| state.db.tags().delete(&id))
}

#[tauri::command]
pub fn set_library_tags(
    state: State<Arc<AppState>>,
    library_item_id: String,
    tag_ids: Vec<String>,
) -> Result<Vec<String>, String> {
    run(|| {
        let tags = state.db.tags().set_for_item(&library_item_id, &tag_ids)?;
        state.db.library().refresh_fts_tags(&library_item_id)?;
        Ok(tags.into_iter().map(|tag| tag.name).collect())
    })
}

#[tauri::command]
pub fn list_collections(state: State<Arc<AppState>>) -> Result<Vec<Collection>, String> {
    run(|| state.db.collections().list())
}

#[tauri::command]
pub fn create_collection(state: State<Arc<AppState>>, name: String) -> Result<Collection, String> {
    run(|| state.db.collections().create(&name))
}

#[tauri::command]
pub fn rename_collection(
    state: State<Arc<AppState>>,
    id: String,
    name: String,
) -> Result<Collection, String> {
    run(|| state.db.collections().rename(&id, &name))
}

#[tauri::command]
pub fn delete_collection(state: State<Arc<AppState>>, id: String) -> Result<(), String> {
    run(|| state.db.collections().delete(&id))
}

#[tauri::command]
pub fn add_to_collection(
    state: State<Arc<AppState>>,
    collection_id: String,
    library_item_id: String,
) -> Result<(), String> {
    run(|| {
        state
            .db
            .collections()
            .add_item(&collection_id, &library_item_id)
    })
}

#[tauri::command]
pub fn remove_from_collection(
    state: State<Arc<AppState>>,
    collection_id: String,
    library_item_id: String,
) -> Result<(), String> {
    run(|| {
        state
            .db
            .collections()
            .remove_item(&collection_id, &library_item_id)
    })
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    run(|| shell::reveal_in_finder(&path))
}

#[tauri::command]
pub fn open_local_file(app: tauri::AppHandle, path: String) -> Result<(), String> {
    run(|| tasks::open_path(&app, &path))
}

#[tauri::command]
pub fn read_local_file(path: String) -> Result<String, String> {
    run(|| shell::read_text_file(&path, 512 * 1024))
}

#[tauri::command]
pub fn validate_ffmpeg(path: String) -> Result<FfmpegStatus, String> {
    run(|| {
        let (valid, message, resolved_path) = shell::validate_ffmpeg(&path)?;
        Ok(FfmpegStatus {
            valid,
            message,
            resolved_path,
        })
    })
}

#[tauri::command]
pub fn get_app_paths(state: State<Arc<AppState>>) -> Result<serde_json::Value, String> {
    run(|| {
        let settings = state.db.settings().get_all()?;
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Cliprove")
            .join("logs");
        Ok(serde_json::json!({
            "databasePath": state.db.path().to_string_lossy(),
            "downloadDirectory": settings.download_directory,
            "logDirectory": log_dir.to_string_lossy(),
        }))
    })
}

#[tauri::command]
pub fn get_settings(state: State<Arc<AppState>>) -> Result<AppSettings, String> {
    run(|| {
        ensure_ffmpeg_for_db(&state)?;
        state.db.settings().get_all()
    })
}

#[tauri::command]
pub fn update_settings(
    state: State<Arc<AppState>>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    run(|| state.db.settings().update(&settings))
}

#[tauri::command]
pub fn validate_platform_auth(
    state: State<Arc<AppState>>,
    platform: String,
) -> Result<AuthStatus, String> {
    run(|| {
        let settings = state.db.settings().get_all()?;
        if platform == "douyin" || platform == "bilibili" || platform == "youtube" {
            ensure_sidecar(&state, &platform)?;
            return state
                .sidecar
                .client()?
                .validate_auth(&platform, &settings);
        }
        let cookies = settings.bilibili_cookies;
        Ok(mock::validate_auth(&platform, &cookies))
    })
}

#[tauri::command]
pub fn start_platform_login(
    state: State<Arc<AppState>>,
    platform: String,
) -> Result<PlatformLoginSession, String> {
    run(|| {
        if platform != "douyin" && platform != "bilibili" {
            return Err(AppError::Message("不支持的平台".to_string()));
        }
        ensure_sidecar(&state, &platform)?;
        state.sidecar.client()?.start_platform_login(&platform)
    })
}

#[tauri::command]
pub fn poll_platform_login(
    state: State<Arc<AppState>>,
    session_id: String,
) -> Result<PlatformLoginSession, String> {
    run(|| {
        ensure_sidecar(&state, "douyin")?;
        state.sidecar.client()?.poll_platform_login(&session_id)
    })
}

#[tauri::command]
pub fn resolve_media_preview(
    state: State<Arc<AppState>>,
    platform: String,
    platform_item_id: String,
) -> Result<Option<String>, String> {
    run(|| {
        if platform == "bilibili" {
            ensure_sidecar(&state, &platform)?;
            return state
                .sidecar
                .client()?
                .resolve_bilibili_preview_url(&platform_item_id);
        }
        Ok(None)
    })
}

#[tauri::command]
pub fn start_sidecar(state: State<Arc<AppState>>) -> Result<SidecarHealth, String> {
    run(|| state.sidecar.start())
}

#[tauri::command]
pub fn sidecar_health(state: State<Arc<AppState>>) -> Result<SidecarHealth, String> {
    run(|| state.sidecar.health())
}

#[tauri::command]
pub fn ensure_ffmpeg(state: State<Arc<AppState>>) -> Result<FfmpegStatus, String> {
    run(|| ensure_ffmpeg_for_db(&state))
}

#[tauri::command]
pub fn count_library(state: State<Arc<AppState>>) -> Result<i64, String> {
    run(|| state.db.library().count())
}

fn ensure_ffmpeg_for_db(state: &AppState) -> AppResult<FfmpegStatus> {
    let settings = state.db.settings().get_all()?;
    if let Some(resolved) = shell::resolve_ffmpeg_path(&settings.ffmpeg_path) {
        let resolved_str = resolved.to_string_lossy().to_string();
        let (valid, message, resolved_path) = shell::validate_ffmpeg(&resolved_str)?;
        if valid && resolved_str != settings.ffmpeg_path {
            let mut partial = settings;
            partial.ffmpeg_path = resolved_str;
            state.db.settings().update(&partial)?;
        }
        return Ok(FfmpegStatus {
            valid,
            message,
            resolved_path,
        });
    }

    Ok(FfmpegStatus {
        valid: false,
        message: "未找到 FFmpeg，请安装后重试".to_string(),
        resolved_path: None,
    })
}

fn run<T>(f: impl FnOnce() -> AppResult<T>) -> Result<T, String> {
    f().map_err(|error| error.to_string())
}
