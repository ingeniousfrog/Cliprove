use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::errors::{AppError, AppResult};
use crate::models::{LibraryItem, MediaItem};

pub struct LibraryRepository<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> LibraryRepository<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self {
        Self { conn }
    }

    pub fn exists(&self, platform: &str, platform_item_id: &str) -> AppResult<bool> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(1) FROM library_items WHERE platform = ?1 AND platform_item_id = ?2",
            params![platform, platform_item_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn list(&self, query: Option<&str>) -> AppResult<Vec<LibraryItem>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;

        if let Some(query) = query.filter(|value| !value.trim().is_empty()) {
            let pattern = format!("%{}%", query.trim());
            let mut stmt = conn.prepare(
                "SELECT id, platform, platform_item_id, original_url, canonical_url, title,
                        description, author_id, author_name, published_at, media_type, duration_sec,
                        cover_path, media_paths, metadata_path, subtitle_paths, file_size, checksum,
                        search_keyword, created_at, updated_at
                 FROM library_items
                 WHERE title LIKE ?1 OR author_name LIKE ?1 OR platform_item_id LIKE ?1
                 ORDER BY created_at DESC",
            )?;
            let rows = stmt.query_map([pattern], map_library_row)?;
            return rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from);
        }

        let mut stmt = conn.prepare(
            "SELECT id, platform, platform_item_id, original_url, canonical_url, title,
                    description, author_id, author_name, published_at, media_type, duration_sec,
                    cover_path, media_paths, metadata_path, subtitle_paths, file_size, checksum,
                    search_keyword, created_at, updated_at
             FROM library_items
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], map_library_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn insert_from_media(&self, item: &MediaItem, output_dir: &str) -> AppResult<LibraryItem> {
        let now = chrono::Utc::now().timestamp_millis();
        let library_item = LibraryItem {
            id: uuid::Uuid::new_v4().to_string(),
            platform: item.platform.clone(),
            platform_item_id: item.platform_item_id.clone(),
            original_url: item.original_url.clone(),
            canonical_url: item.canonical_url.clone(),
            title: item.title.clone(),
            description: item.description.clone(),
            author_id: item.author.id.clone(),
            author_name: item.author.name.clone(),
            published_at: item.published_at,
            media_type: item.media_type.clone(),
            duration_sec: item.duration_sec,
            cover_path: Some(format!("{output_dir}/cover.jpg")),
            media_paths: vec![format!("{output_dir}/video.mp4")],
            metadata_path: Some(format!("{output_dir}/metadata.json")),
            subtitle_paths: vec![],
            file_size: Some(12_884_901),
            checksum: None,
            search_keyword: item.search_keyword.clone(),
            created_at: now,
            updated_at: now,
        };

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "INSERT INTO library_items (
                id, platform, platform_item_id, original_url, canonical_url, title, description,
                author_id, author_name, published_at, media_type, duration_sec, cover_path,
                media_paths, metadata_path, subtitle_paths, file_size, checksum, search_keyword,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                library_item.id,
                library_item.platform,
                library_item.platform_item_id,
                library_item.original_url,
                library_item.canonical_url,
                library_item.title,
                library_item.description,
                library_item.author_id,
                library_item.author_name,
                library_item.published_at,
                library_item.media_type,
                library_item.duration_sec,
                library_item.cover_path,
                serde_json::to_string(&library_item.media_paths)?,
                library_item.metadata_path,
                serde_json::to_string(&library_item.subtitle_paths)?,
                library_item.file_size,
                library_item.checksum,
                library_item.search_keyword,
                library_item.created_at,
                library_item.updated_at,
            ],
        )?;

        Ok(library_item)
    }

    pub fn insert_with_paths(
        &self,
        item: &MediaItem,
        cover_path: Option<String>,
        media_paths: Vec<String>,
        metadata_path: Option<String>,
        subtitle_paths: Vec<String>,
        file_size: Option<i64>,
    ) -> AppResult<LibraryItem> {
        let now = chrono::Utc::now().timestamp_millis();
        let library_item = LibraryItem {
            id: uuid::Uuid::new_v4().to_string(),
            platform: item.platform.clone(),
            platform_item_id: item.platform_item_id.clone(),
            original_url: item.original_url.clone(),
            canonical_url: item.canonical_url.clone(),
            title: item.title.clone(),
            description: item.description.clone(),
            author_id: item.author.id.clone(),
            author_name: item.author.name.clone(),
            published_at: item.published_at,
            media_type: item.media_type.clone(),
            duration_sec: item.duration_sec,
            cover_path,
            media_paths,
            metadata_path,
            subtitle_paths,
            file_size,
            checksum: None,
            search_keyword: item.search_keyword.clone(),
            created_at: now,
            updated_at: now,
        };

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "INSERT INTO library_items (
                id, platform, platform_item_id, original_url, canonical_url, title, description,
                author_id, author_name, published_at, media_type, duration_sec, cover_path,
                media_paths, metadata_path, subtitle_paths, file_size, checksum, search_keyword,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                library_item.id,
                library_item.platform,
                library_item.platform_item_id,
                library_item.original_url,
                library_item.canonical_url,
                library_item.title,
                library_item.description,
                library_item.author_id,
                library_item.author_name,
                library_item.published_at,
                library_item.media_type,
                library_item.duration_sec,
                library_item.cover_path,
                serde_json::to_string(&library_item.media_paths)?,
                library_item.metadata_path,
                serde_json::to_string(&library_item.subtitle_paths)?,
                library_item.file_size,
                library_item.checksum,
                library_item.search_keyword,
                library_item.created_at,
                library_item.updated_at,
            ],
        )?;

        Ok(library_item)
    }
}

fn map_library_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryItem> {
    let media_paths: String = row.get(13)?;
    let subtitle_paths: String = row.get(15)?;

    Ok(LibraryItem {
        id: row.get(0)?,
        platform: row.get(1)?,
        platform_item_id: row.get(2)?,
        original_url: row.get(3)?,
        canonical_url: row.get(4)?,
        title: row.get(5)?,
        description: row.get(6)?,
        author_id: row.get(7)?,
        author_name: row.get(8)?,
        published_at: row.get(9)?,
        media_type: row.get(10)?,
        duration_sec: row.get(11)?,
        cover_path: row.get(12)?,
        media_paths: serde_json::from_str(&media_paths).unwrap_or_default(),
        metadata_path: row.get(14)?,
        subtitle_paths: serde_json::from_str(&subtitle_paths).unwrap_or_default(),
        file_size: row.get(16)?,
        checksum: row.get(17)?,
        search_keyword: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
    })
}
