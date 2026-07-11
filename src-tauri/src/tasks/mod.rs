use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::db::Database;
use crate::errors::{AppError, AppResult};
use crate::models::{DownloadOptions, MediaItem, StructuredError};
use crate::sidecar::SidecarManager;

pub async fn run_task(
    app: AppHandle,
    db: Arc<Database>,
    sidecar: Arc<SidecarManager>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    options: DownloadOptions,
) -> AppResult<()> {
    if item.platform == "douyin" {
        return run_douyin_task(app, db, sidecar, task_id, item, output_dir, options).await;
    }

    run_mock_task(app, db, task_id, item, output_dir).await
}

async fn run_douyin_task(
    app: AppHandle,
    db: Arc<Database>,
    sidecar: Arc<SidecarManager>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    options: DownloadOptions,
) -> AppResult<()> {
    db.tasks().update_progress(&task_id, "parsing", "连接引擎", 0.05, None)?;

    let settings = db.settings().get_all()?;
    let client = sidecar.client()?;

    db.tasks().update_progress(&task_id, "downloading", "提交下载任务", 0.1, None)?;
    let job = client.start_download(&item, &options, &output_dir, &settings)?;

    let mut last_progress = 0.1;
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let current = client.get_job(&job.job_id)?;

        let progress = current.progress.clamp(0.0, 1.0);
        if progress > last_progress {
            db.tasks().update_progress(
                &task_id,
                "downloading",
                &current.stage,
                progress,
                None,
            )?;
            last_progress = progress;
            emit_progress(&app, &task_id, &current.stage, progress);
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
                let error = StructuredError {
                    code: "engine_failure".to_string(),
                    message: message.clone(),
                    suggestion: Some("检查抖音 Cookie、网络连接或稍后重试".to_string()),
                    technical_detail: Some(message),
                };
                db.tasks().mark_failed(&task_id, &error)?;
                return Ok(());
            }
            "running" | "queued" => continue,
            _ => continue,
        }
    }
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
    db: Arc<Database>,
    sidecar: Arc<SidecarManager>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    options: DownloadOptions,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = run_task(
            app,
            db,
            sidecar,
            task_id.clone(),
            item,
            output_dir,
            options,
        )
        .await
        {
            tracing::error!("task {task_id} failed: {error}");
        }
    });
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
