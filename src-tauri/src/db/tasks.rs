use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::errors::{AppError, AppResult};
use crate::models::{DownloadOptions, DownloadTask, MediaItem, StructuredError};

#[derive(Debug, Clone)]
pub struct TaskPayload {
    pub item_json: Option<String>,
    pub options_json: Option<String>,
    pub output_dir: Option<String>,
}

pub struct TaskRepository<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> TaskRepository<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> AppResult<Vec<DownloadTask>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let mut stmt = conn.prepare(
            "SELECT id, platform, platform_item_id, title, status, stage, progress,
                    speed_bps, retry_count, error_json, output_dir, library_item_id,
                    created_at, updated_at, completed_at
             FROM download_tasks
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], map_task_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<DownloadTask>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.query_row(
            "SELECT id, platform, platform_item_id, title, status, stage, progress,
                    speed_bps, retry_count, error_json, output_dir, library_item_id,
                    created_at, updated_at, completed_at
             FROM download_tasks WHERE id = ?1",
            [id],
            map_task_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn insert(
        &self,
        item: &MediaItem,
        options: &DownloadOptions,
        output_dir: &str,
    ) -> AppResult<DownloadTask> {
        let now = chrono::Utc::now().timestamp_millis();
        let item_json = serde_json::to_string(item)?;
        let options_json = serde_json::to_string(options)?;
        let task = DownloadTask {
            id: uuid::Uuid::new_v4().to_string(),
            platform: item.platform.clone(),
            platform_item_id: item.platform_item_id.clone(),
            title: item.title.clone(),
            status: "queued".to_string(),
            stage: Some("等待执行".to_string()),
            progress: 0.0,
            speed_bps: None,
            retry_count: 0,
            error: None,
            output_dir: Some(output_dir.to_string()),
            library_item_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "INSERT INTO download_tasks (
                id, platform, platform_item_id, title, status, stage, progress,
                speed_bps, retry_count, options_json, item_json, output_dir, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                task.id,
                task.platform,
                task.platform_item_id,
                task.title,
                task.status,
                task.stage,
                task.progress,
                task.speed_bps,
                task.retry_count,
                options_json,
                item_json,
                task.output_dir,
                task.created_at,
                task.updated_at,
            ],
        )?;

        Ok(task)
    }

    pub fn update_progress(
        &self,
        id: &str,
        status: &str,
        stage: &str,
        progress: f64,
        speed_bps: Option<i64>,
    ) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = ?2, stage = ?3, progress = ?4, speed_bps = ?5, updated_at = ?6
             WHERE id = ?1",
            params![id, status, stage, progress, speed_bps, now],
        )?;
        Ok(())
    }

    pub fn mark_completed(&self, id: &str, library_item_id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = 'completed', stage = '已完成', progress = 1.0, library_item_id = ?2,
                 completed_at = ?3, updated_at = ?3, speed_bps = NULL
             WHERE id = ?1",
            params![id, library_item_id, now],
        )?;
        Ok(())
    }

    pub fn mark_failed(&self, id: &str, error: &StructuredError) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = 'failed', stage = '失败', error_json = ?2, updated_at = ?3, speed_bps = NULL
             WHERE id = ?1",
            params![id, serde_json::to_string(error)?, now],
        )?;
        Ok(())
    }

    pub fn mark_cancelled(&self, id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = 'cancelled', stage = '已取消', updated_at = ?2, speed_bps = NULL
             WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }

    pub fn mark_retry(&self, id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = 'queued', stage = '等待重试', progress = 0, error_json = NULL,
                 retry_count = retry_count + 1, updated_at = ?2
             WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute("DELETE FROM download_tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_payload(&self, id: &str) -> AppResult<Option<TaskPayload>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.query_row(
            "SELECT item_json, options_json, output_dir FROM download_tasks WHERE id = ?1",
            [id],
            |row| {
                Ok(TaskPayload {
                    item_json: row.get(0)?,
                    options_json: row.get(1)?,
                    output_dir: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn mark_resume(&self, id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = 'queued', stage = '等待恢复', progress = 0, error_json = NULL, updated_at = ?2
             WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }

    pub fn recover_interrupted(&self) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE download_tasks
             SET status = 'interrupted', stage = '上次运行中断', updated_at = ?1
             WHERE status IN ('parsing', 'downloading', 'post_processing')",
            [now],
        )?;
        Ok(())
    }
}

fn map_task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DownloadTask> {
    let error_json: Option<String> = row.get(9)?;
    let error = error_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<StructuredError>(value).ok());

    Ok(DownloadTask {
        id: row.get(0)?,
        platform: row.get(1)?,
        platform_item_id: row.get(2)?,
        title: row.get(3)?,
        status: row.get(4)?,
        stage: row.get(5)?,
        progress: row.get(6)?,
        speed_bps: row.get(7)?,
        retry_count: row.get(8)?,
        error,
        output_dir: row.get(10)?,
        library_item_id: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        completed_at: row.get(14)?,
    })
}
