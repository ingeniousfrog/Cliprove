use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::db::Database;
use crate::errors::AppResult;
use crate::models::{DownloadOptions, MediaItem};

pub async fn simulate_task(
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
        db.tasks().update_progress(
            &task_id,
            status,
            stage,
            progress,
            Some(2_500_000),
        )?;
        let _ = app.emit(
            "download-progress",
            serde_json::json!({
                "taskId": task_id,
                "stage": stage,
                "progress": progress,
                "speedBps": 2_500_000,
                "retryCount": 0
            }),
        );
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    std::fs::create_dir_all(&output_dir).ok();
    let library_item = db.library().insert_from_media(&item, &output_dir)?;
    db.tasks()
        .mark_completed(&task_id, &library_item.id)?;

    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "taskId": task_id,
            "stage": "已完成",
            "progress": 1.0,
            "speedBps": null,
            "retryCount": 0
        }),
    );

    Ok(())
}

pub fn spawn_task(
    app: AppHandle,
    db: Arc<Database>,
    task_id: String,
    item: MediaItem,
    output_dir: String,
    _options: DownloadOptions,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = simulate_task(app, db, task_id, item, output_dir).await {
            tracing::error!("mock task failed: {error}");
        }
    });
}
