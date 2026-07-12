use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;

use crate::app_state::AppState;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use crate::models::{DownloadOptions, MediaItem, StructuredError};
use crate::sidecar::SidecarManager;

const TASK_TIMEOUT: Duration = Duration::from_secs(600);
const SUBMIT_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_TIMEOUT: Duration = Duration::from_secs(10);
const STAGNANT_POLL_LIMIT: u32 = 120;

pub async fn run_task(
    app: AppHandle,
    state: Arc<AppState>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    options: DownloadOptions,
) -> AppResult<()> {
    let _permit = state
        .download_slots
        .acquire()
        .await
        .map_err(|_| AppError::Message("下载并发槽不可用".to_string()))?;

    if item.platform == "douyin" || item.platform == "bilibili" || item.platform == "youtube" {
        return run_sidecar_task(
            app,
            Arc::clone(&state.db),
            Arc::clone(&state.sidecar),
            task_id,
            item,
            output_dir,
            options,
        )
        .await;
    }

    run_mock_task(app, Arc::clone(&state.db), task_id, item, output_dir).await
}

async fn run_sidecar_task(
    app: AppHandle,
    db: Arc<Database>,
    sidecar: Arc<SidecarManager>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    options: DownloadOptions,
) -> AppResult<()> {
    db.tasks()
        .update_progress(&task_id, "parsing", "连接引擎", 0.05, None)?;
    emit_progress(&app, &task_id, "连接引擎", 0.05);

    db.tasks()
        .update_progress(&task_id, "downloading", "提交下载任务", 0.1, None)?;
    emit_progress(&app, &task_id, "提交下载任务", 0.1);

    let settings = db.settings().get_all()?;
    let poll_port = sidecar.port();
    let job = submit_download_job(
        Arc::clone(&sidecar),
        &item,
        &options,
        &output_dir,
        &settings,
    )
    .await?;

    let mut last_progress = 0.1;
    let mut last_stage = "提交下载任务".to_string();
    let started = Instant::now();
    let mut stagnant_ticks = 0u32;

    loop {
        if started.elapsed() > TASK_TIMEOUT {
            return Err(AppError::structured(
                "engine_failure",
                "下载超时（超过 10 分钟）",
                Some("请取消任务后重试，或检查网络与 Sidecar 状态".to_string()),
            ));
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        let current = poll_job(poll_port, &job.job_id).await?;

        let progress = current.progress.clamp(0.0, 1.0);
        let stage = if current.stage.is_empty() {
            "下载中".to_string()
        } else {
            current.stage.clone()
        };
        let display_progress = progress.max(last_progress);

        if progress > last_progress || stage != last_stage {
            db.tasks().update_progress(
                &task_id,
                "downloading",
                &stage,
                display_progress,
                None,
            )?;
            emit_progress(&app, &task_id, &stage, display_progress);
            last_progress = display_progress;
            last_stage = stage;
            stagnant_ticks = 0;
        } else {
            stagnant_ticks += 1;
            if stagnant_ticks > STAGNANT_POLL_LIMIT {
                return Err(AppError::structured(
                    "engine_failure",
                    "下载进度长时间无变化",
                    Some("Sidecar 可能已卡住，请重启应用后重试".to_string()),
                ));
            }
        }

        match current.status.as_str() {
            "completed" => {
                let result = current.result.ok_or_else(|| {
                    AppError::structured("download_incomplete", "下载完成但缺少结果", None)
                })?;

                db.tasks().update_progress(
                    &task_id,
                    "post_processing",
                    "写入库记录",
                    0.98,
                    None,
                )?;

                let library_item = db.library().insert_with_paths(
                    &item,
                    result.cover_path,
                    result.media_paths,
                    result.metadata_path,
                    result.subtitle_paths,
                    result.file_size,
                )?;

                db.tasks().mark_completed(&task_id, &library_item.id)?;
                emit_progress(&app, &task_id, "已完成", 1.0);
                return Ok(());
            }
            "failed" => {
                let message = current.error.unwrap_or_else(|| "下载失败".to_string());
                let error = structured_error_from_sidecar_message(&message);
                db.tasks().mark_failed(&task_id, &error)?;
                return Ok(());
            }
            "running" | "queued" => continue,
            _ => continue,
        }
    }
}

async fn submit_download_job(
    sidecar: Arc<SidecarManager>,
    item: &MediaItem,
    options: &DownloadOptions,
    output_dir: &str,
    settings: &crate::models::AppSettings,
) -> AppResult<crate::sidecar::SidecarJob> {
    match try_submit_download(Arc::clone(&sidecar), item, options, output_dir, settings).await {
        Ok(job) => Ok(job),
        Err(error) if should_restart_sidecar(&error) => {
            let sidecar_for_restart = Arc::clone(&sidecar);
            tokio::time::timeout(
                Duration::from_secs(15),
                tokio::task::spawn_blocking(move || sidecar_for_restart.start()),
            )
            .await
            .map_err(|_| {
                AppError::structured(
                    "engine_failure",
                    "Sidecar 重启超时",
                    Some("请完全退出应用后重新打开".to_string()),
                )
            })?
            .map_err(|_| AppError::Message("Sidecar 重启异常退出".to_string()))??;

            try_submit_download(sidecar, item, options, output_dir, settings).await
        }
        Err(error) => Err(error),
    }
}

fn should_restart_sidecar(error: &AppError) -> bool {
    match error {
        AppError::Http(_) => true,
        AppError::Structured { message, .. } => {
            let lowered = message.to_lowercase();
            lowered.contains("sidecar")
                || lowered.contains("connect")
                || lowered.contains("connection")
                || lowered.contains("timeout")
                || lowered.contains("引擎")
        }
        AppError::Message(message) => {
            let lowered = message.to_lowercase();
            lowered.contains("timeout") || lowered.contains("异常退出")
        }
        _ => false,
    }
}

async fn try_submit_download(
    sidecar: Arc<SidecarManager>,
    item: &MediaItem,
    options: &DownloadOptions,
    output_dir: &str,
    settings: &crate::models::AppSettings,
) -> AppResult<crate::sidecar::SidecarJob> {
    let item = item.clone();
    let options = options.clone();
    let output_dir = output_dir.to_string();
    let settings = settings.clone();

    tokio::time::timeout(
        SUBMIT_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            let client = sidecar.client()?;
            client.start_download(&item, &options, &output_dir, &settings)
        }),
    )
    .await
    .map_err(|_| {
        AppError::structured(
            "engine_failure",
            "提交下载任务超时",
            Some("Sidecar 无响应，请重启应用后重试".to_string()),
        )
    })?
    .map_err(|_| AppError::Message("提交下载任务异常退出".to_string()))?
}

async fn poll_job(port: u16, job_id: &str) -> AppResult<crate::sidecar::SidecarJob> {
    let job_id = job_id.to_string();

    tokio::time::timeout(
        POLL_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            let client = crate::sidecar::SidecarClient::with_timeout(
                port,
                Duration::from_secs(5),
            )?;
            client.get_job(&job_id)
        }),
    )
    .await
    .map_err(|_| {
        AppError::structured(
            "engine_failure",
            "查询下载进度超时",
            Some("Sidecar 可能已卡住，请重启应用后重试".to_string()),
        )
    })?
    .map_err(|_| AppError::Message("查询下载进度异常退出".to_string()))?
}

async fn run_mock_task(
    app: AppHandle,
    db: Arc<Database>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
) -> AppResult<()> {
    let stages = [
        ("parsing", "解析资源", 0.15),
        ("downloading", "下载媒体", 0.75),
        ("post_processing", "写入库记录", 0.95),
    ];

    for (status, stage, progress) in stages {
        db.tasks()
            .update_progress(&task_id, status, stage, progress, Some(2_500_000))?;
        emit_progress(&app, &task_id, stage, progress);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    std::fs::create_dir_all(&output_dir).ok();
    let library_item = db.library().insert_from_media(&item, &output_dir)?;
    db.tasks().mark_completed(&task_id, &library_item.id)?;
    emit_progress(&app, &task_id, "已完成", 1.0);
    Ok(())
}

pub fn spawn_task(
    app: AppHandle,
    state: Arc<AppState>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    options: DownloadOptions,
) {
    tauri::async_runtime::spawn(async move {
        let result = tokio::time::timeout(
            TASK_TIMEOUT + Duration::from_secs(30),
            run_task(
                app,
                state.clone(),
                task_id.clone(),
                item,
                output_dir,
                options,
            ),
        )
        .await;

        let error = match result {
            Ok(Ok(())) => return,
            Ok(Err(error)) => error,
            Err(_) => AppError::structured(
                "engine_failure",
                "下载任务超时",
                Some("请取消任务后重试，或重启应用".to_string()),
            ),
        };

        tracing::error!("task {task_id} failed: {error}");
        let structured = structured_error_from_app_error(&error);
        let _ = state.db.tasks().mark_failed(&task_id, &structured);
    });
}

fn structured_error_from_sidecar_message(message: &str) -> StructuredError {
    if let Some(rest) = message.strip_prefix("CLIPROVE_FFMPEG_UNAVAILABLE:") {
        return StructuredError {
            code: "ffmpeg_unavailable".to_string(),
            message: rest.trim().to_string(),
            suggestion: Some(
                "请安装 FFmpeg（brew install ffmpeg）或在设置中指定路径".to_string(),
            ),
            technical_detail: Some(message.to_string()),
        };
    }
    if let Some(rest) = message.strip_prefix("CLIPROVE_AUTH_REQUIRED:") {
        return StructuredError {
            code: "auth_required".to_string(),
            message: rest.trim().to_string(),
            suggestion: Some("请在弹窗中扫码登录后重试".to_string()),
            technical_detail: Some(message.to_string()),
        };
    }
    if let Some(rest) = message.strip_prefix("CLIPROVE_AUTH_EXPIRED:") {
        return StructuredError {
            code: "auth_expired".to_string(),
            message: rest.trim().to_string(),
            suggestion: Some("请重新登录平台账号后重试".to_string()),
            technical_detail: Some(message.to_string()),
        };
    }
    if let Some(rest) = message.strip_prefix("CLIPROVE_VERIFICATION_REQUIRED:") {
        return StructuredError {
            code: "verification_required".to_string(),
            message: rest.trim().to_string(),
            suggestion: Some("请在浏览器完成验证后重试".to_string()),
            technical_detail: Some(message.to_string()),
        };
    }
    if let Some(rest) = message.strip_prefix("CLIPROVE_REGION_RESTRICTED:") {
        return StructuredError {
            code: "region_restricted".to_string(),
            message: rest.trim().to_string(),
            suggestion: Some("请换一个视频，或切换到允许访问该视频的网络后重试".to_string()),
            technical_detail: Some(message.to_string()),
        };
    }

    let lowered = message.to_lowercase();
    if lowered.contains("ffmpeg") {
        return StructuredError {
            code: "ffmpeg_unavailable".to_string(),
            message: message.to_string(),
            suggestion: Some(
                "请安装 FFmpeg（brew install ffmpeg）或在设置中指定路径".to_string(),
            ),
            technical_detail: Some(message.to_string()),
        };
    }
    if lowered.contains("sessdata")
        || lowered.contains("login")
        || lowered.contains("sign in")
        || lowered.contains("登录")
        || lowered.contains("cookie")
    {
        return StructuredError {
            code: "auth_required".to_string(),
            message: message.to_string(),
            suggestion: Some("请登录对应平台后重试".to_string()),
            technical_detail: Some(message.to_string()),
        };
    }

    StructuredError {
        code: "engine_failure".to_string(),
        message: message.to_string(),
        suggestion: Some("检查 Cookie、FFmpeg 路径与网络连接".to_string()),
        technical_detail: Some(message.to_string()),
    }
}

fn structured_error_from_app_error(error: &AppError) -> StructuredError {
    match error {
        AppError::Structured {
            code,
            message,
            suggestion,
            technical_detail,
        } => StructuredError {
            code: code.clone(),
            message: message.clone(),
            suggestion: suggestion.clone(),
            technical_detail: technical_detail.clone(),
        },
        other => StructuredError {
            code: "engine_failure".to_string(),
            message: other.to_string(),
            suggestion: Some("请检查 Sidecar、网络与平台凭证".to_string()),
            technical_detail: Some(other.to_string()),
        },
    }
}

fn emit_progress(app: &AppHandle, task_id: &str, stage: &str, progress: f64) {
    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "taskId": task_id,
            "stage": stage,
            "progress": progress,
            "speedBps": null,
            "retryCount": 0
        }),
    );
}

pub fn open_path(app: &AppHandle, path: &str) -> AppResult<()> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| AppError::Message(error.to_string()))
}
