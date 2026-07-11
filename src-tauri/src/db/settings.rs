use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::errors::{AppError, AppResult};
use crate::models::AppSettings;

pub struct SettingsRepository<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self {
        Self { conn }
    }

    pub fn get_all(&self) -> AppResult<AppSettings> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;

        let mut settings = AppSettings::default();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            apply_setting(&mut settings, &key, &value);
        }

        Ok(settings)
    }

    pub fn update(&self, partial: &AppSettings) -> AppResult<AppSettings> {
        let current = self.get_all()?;
        let merged = merge_settings(current, partial);
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;

        for (key, value) in settings_entries(&merged) {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![key, value, now],
            )?;
        }

        Ok(merged)
    }
}

fn apply_setting(settings: &mut AppSettings, key: &str, value: &str) {
    match key {
        "download_directory" => settings.download_directory = value.to_string(),
        "filename_template" => settings.filename_template = value.to_string(),
        "max_concurrent_downloads" => {
            settings.max_concurrent_downloads = value.parse().unwrap_or(settings.max_concurrent_downloads)
        }
        "retry_count" => settings.retry_count = value.parse().unwrap_or(settings.retry_count),
        "ffmpeg_path" => settings.ffmpeg_path = value.to_string(),
        "douyin_cookies" => settings.douyin_cookies = value.to_string(),
        "bilibili_cookies" => settings.bilibili_cookies = value.to_string(),
        "save_metadata" => settings.save_metadata = value == "true",
        "save_cover" => settings.save_cover = value == "true",
        "save_audio" => settings.save_audio = value == "true",
        "save_subtitles" => settings.save_subtitles = value == "true",
        _ => {}
    }
}

fn merge_settings(mut base: AppSettings, partial: &AppSettings) -> AppSettings {
    if !partial.download_directory.is_empty() {
        base.download_directory = partial.download_directory.clone();
    }
    if !partial.filename_template.is_empty() {
        base.filename_template = partial.filename_template.clone();
    }
    if partial.max_concurrent_downloads > 0 {
        base.max_concurrent_downloads = partial.max_concurrent_downloads;
    }
    if partial.retry_count >= 0 {
        base.retry_count = partial.retry_count;
    }
    if !partial.ffmpeg_path.is_empty() {
        base.ffmpeg_path = partial.ffmpeg_path.clone();
    }
    base.douyin_cookies = partial.douyin_cookies.clone();
    base.bilibili_cookies = partial.bilibili_cookies.clone();
    base.save_metadata = partial.save_metadata;
    base.save_cover = partial.save_cover;
    base.save_audio = partial.save_audio;
    base.save_subtitles = partial.save_subtitles;
    base
}

fn settings_entries(settings: &AppSettings) -> Vec<(&str, String)> {
    vec![
        ("download_directory", settings.download_directory.clone()),
        ("filename_template", settings.filename_template.clone()),
        (
            "max_concurrent_downloads",
            settings.max_concurrent_downloads.to_string(),
        ),
        ("retry_count", settings.retry_count.to_string()),
        ("ffmpeg_path", settings.ffmpeg_path.clone()),
        ("douyin_cookies", settings.douyin_cookies.clone()),
        ("bilibili_cookies", settings.bilibili_cookies.clone()),
        ("save_metadata", settings.save_metadata.to_string()),
        ("save_cover", settings.save_cover.to_string()),
        ("save_audio", settings.save_audio.to_string()),
        ("save_subtitles", settings.save_subtitles.to_string()),
    ]
}
