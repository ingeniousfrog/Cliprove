use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;
use crate::errors::{AppError, AppResult};
use crate::mock;
use crate::models::{
    AppSettings, AuthStatus, DownloadOptions, DownloadSpec, DownloadTask, LibraryItem,
    MediaItem, ParsedMedia, SearchPage, SidecarHealth,
};
use crate::tasks;

fn is_douyin_url(url: &str) -> bool {
    let lowered = url.to_lowercase();
    ["douyin.com", "iesdouyin.com", "v.douyin.com"]
        .iter()
        .any(|token| lowered.contains(token))
}

#[tauri::command]
pub fn parse_link(state: State<AppState>, url: String) -> Result<ParsedMedia, String> {
    run(|| {
        if is_douyin_url(&url) {
            let settings = state.db.settings().get_all()?;
            state.sidecar.start()?;
            return state.sidecar.client()?.parse_link(&url, &settings);
        }
        mock::parse_link(&url)
    })
}

#[tauri::command]
pub fn search_media(
    _state: State<AppState>,
    platform: String,
    query: crate::models::SearchQuery,
    cursor: Option<String>,
) -> Result<SearchPage, String> {
    run(|| mock::search(&platform, &query, cursor.as_deref()))
}

#[tauri::command]
pub fn create_download_spec(
    _state: State<AppState>,
    item: MediaItem,
    options: DownloadOptions,
) -> Result<DownloadSpec, String> {
    run(|| mock::create_download_spec(&item, &options))
}

#[tauri::command]
pub fn enqueue_download(
    app: tauri::AppHandle,
    state: State<AppState>,
    item: MediaItem,
    options: DownloadOptions,
) -> Result<String, String> {
    run(|| {
        let settings = state.db.settings().get_all()?;
        if !options.force_replace.unwrap_or(false)
            && state
                .db
                .library()
                .exists(&item.platform, &item.platform_item_id)?
        {
            return Err(AppError::structured(
                "content_unavailable",
                "该内容已在本地库中",
                Some("如需重新下载，请使用重试并覆盖（后续将提供显式覆盖选项）".to_string()),
            ));
        }

        let output_dir = format!(
            "{}/{}/{}/{}",
            settings.download_directory,
            item.platform,
            item.author.id,
            item.platform_item_id
        );

        let task = state
            .db
            .tasks()
            .insert(&item, &options, &output_dir)?;

        if item.platform == "douyin" {
            state.sidecar.start()?;
        }

        tasks::spawn_task(
            app,
            Arc::clone(&state.db),
            Arc::clone(&state.sidecar),
            task.id.clone(),
            item,
            output_dir,
            options,
        );

        Ok(task.id)
    })
}

#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> Result<Vec<DownloadTask>, String> {
    run(|| state.db.tasks().list())
}

#[tauri::command]
pub fn task_action(
    app: tauri::AppHandle,
    state: State<AppState>,
    task_id: String,
    action: String,
) -> Result<(), String> {
    run(|| {
        match action.as_str() {
            "cancel" => state.db.tasks().mark_cancelled(&task_id)?,
            "retry" => {
                state.db.tasks().mark_retry(&task_id)?;
                if let Some(task) = state.db.tasks().get(&task_id)? {
                    let settings = state.db.settings().get_all()?;
                    let output_dir = task.output_dir.clone().unwrap_or_else(|| {
                        format!(
                            "{}/{}/{}",
                            settings.download_directory, task.platform, task.platform_item_id
                        )
                    });
                    let item = MediaItem {
                        platform: task.platform.clone(),
                        platform_item_id: task.platform_item_id.clone(),
                        original_url: output_dir.clone(),
                        canonical_url: output_dir.clone(),
                        title: task.title.clone(),
                        description: None,
                        author: crate::models::Author {
                            id: "retry".to_string(),
                            name: "Retry".to_string(),
                            avatar_url: None,
                        },
                        published_at: None,
                        media_type: "video".to_string(),
                        duration_sec: None,
                        cover_url: None,
                        search_keyword: None,
                    };
                    if item.platform == "douyin" {
                        state.sidecar.start()?;
                    }
                    tasks::spawn_task(
                        app,
                        Arc::clone(&state.db),
                        Arc::clone(&state.sidecar),
                        task_id,
                        item,
                        output_dir,
                        DownloadOptions {
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
                        },
                    );
                }
            }
            "pause" | "resume" => {}
            _ => {
                return Err(AppError::Message(format!("unsupported action: {action}")));
            }
        }
        Ok(())
    })
}

#[tauri::command]
pub fn list_library(state: State<AppState>, query: Option<String>) -> Result<Vec<LibraryItem>, String> {
    run(|| state.db.library().list(query.as_deref()))
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    run(|| state.db.settings().get_all())
}

#[tauri::command]
pub fn update_settings(
    state: State<AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    run(|| state.db.settings().update(&settings))
}

#[tauri::command]
pub fn validate_platform_auth(
    state: State<AppState>,
    platform: String,
) -> Result<AuthStatus, String> {
    run(|| {
        let settings = state.db.settings().get_all()?;
        if platform == "douyin" {
            state.sidecar.start()?;
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
pub fn start_sidecar(state: State<AppState>) -> Result<SidecarHealth, String> {
    run(|| state.sidecar.start())
}

#[tauri::command]
pub fn sidecar_health(state: State<AppState>) -> Result<SidecarHealth, String> {
    run(|| state.sidecar.health())
}

fn run<T>(f: impl FnOnce() -> AppResult<T>) -> Result<T, String> {
    f().map_err(|error| error.to_string())
}
