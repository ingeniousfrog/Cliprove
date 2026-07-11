use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::errors::{AppError, AppResult};
use crate::models::{LibraryFilter, LibraryItem, MediaItem};

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

    pub fn count(&self) -> AppResult<i64> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let count: i64 = conn.query_row("SELECT COUNT(1) FROM library_items", [], |row| {
            row.get(0)
        })?;
        Ok(count)
    }

    pub fn get(&self, id: &str) -> AppResult<Option<LibraryItem>> {
        let item = {
            let conn = self.conn.lock().map_err(|_| {
                AppError::Message("database lock poisoned".to_string())
            })?;
            conn.query_row(
                "SELECT id, platform, platform_item_id, original_url, canonical_url, title,
                        description, author_id, author_name, published_at, media_type, duration_sec,
                        cover_path, media_paths, metadata_path, subtitle_paths, file_size, checksum,
                        search_keyword, created_at, updated_at
                 FROM library_items WHERE id = ?1",
                [id],
                map_library_row,
            )
            .optional()
            .map_err(AppError::from)?
        };

        match item {
            Some(value) => Ok(Some(self.attach_tags(value)?)),
            None => Ok(None),
        }
    }

    pub fn list(&self, filter: &LibraryFilter) -> AppResult<Vec<LibraryItem>> {
        let items = {
            let conn = self.conn.lock().map_err(|_| {
                AppError::Message("database lock poisoned".to_string())
            })?;

            let mut sql = String::from(
                "SELECT DISTINCT li.id, li.platform, li.platform_item_id, li.original_url, li.canonical_url,
                        li.title, li.description, li.author_id, li.author_name, li.published_at,
                        li.media_type, li.duration_sec, li.cover_path, li.media_paths, li.metadata_path,
                        li.subtitle_paths, li.file_size, li.checksum, li.search_keyword,
                        li.created_at, li.updated_at
                 FROM library_items li",
            );
            let mut joins = Vec::new();
            let mut conditions = Vec::new();
            let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(query) = filter.query.as_deref().filter(|value| !value.trim().is_empty())
            {
                joins.push(
                    "INNER JOIN library_items_fts fts ON fts.library_item_id = li.id".to_string(),
                );
                conditions.push("library_items_fts MATCH ?".to_string());
                bind_values.push(Box::new(fts_query(query)));
            }

            if let Some(platform) = &filter.platform {
                conditions.push("li.platform = ?".to_string());
                bind_values.push(Box::new(platform.clone()));
            }

            if let Some(media_type) = &filter.media_type {
                conditions.push("li.media_type = ?".to_string());
                bind_values.push(Box::new(media_type.clone()));
            }

            if let Some(date_from) = filter.date_from {
                conditions.push("li.created_at >= ?".to_string());
                bind_values.push(Box::new(date_from));
            }

            if let Some(date_to) = filter.date_to {
                conditions.push("li.created_at <= ?".to_string());
                bind_values.push(Box::new(date_to));
            }

            if let Some(collection_id) = &filter.collection_id {
                joins.push(
                    "INNER JOIN collection_items ci ON ci.library_item_id = li.id".to_string(),
                );
                conditions.push("ci.collection_id = ?".to_string());
                bind_values.push(Box::new(collection_id.clone()));
            }

            if let Some(tag_id) = &filter.tag_id {
                joins.push("INNER JOIN item_tags it ON it.library_item_id = li.id".to_string());
                conditions.push("it.tag_id = ?".to_string());
                bind_values.push(Box::new(tag_id.clone()));
            }

            if !joins.is_empty() {
                sql.push(' ');
                sql.push_str(&joins.join(" "));
            }

            if !conditions.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&conditions.join(" AND "));
            }

            sql.push_str(" ORDER BY li.created_at DESC");

            let mut stmt = conn.prepare(&sql)?;
            let params_ref: Vec<&dyn rusqlite::types::ToSql> =
                bind_values.iter().map(|value| value.as_ref()).collect();
            let rows = stmt.query_map(params_ref.as_slice(), map_library_row)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)?
        };

        items
            .into_iter()
            .map(|item| self.attach_tags(item))
            .collect()
    }

    pub fn delete(&self, id: &str, delete_files: bool) -> AppResult<()> {
        let item = self
            .get(id)?
            .ok_or_else(|| AppError::Message("库条目不存在".to_string()))?;

        if delete_files {
            delete_local_files(&item)?;
        }

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "DELETE FROM library_items_fts WHERE library_item_id = ?1",
            [id],
        )?;
        conn.execute("DELETE FROM library_items WHERE id = ?1", [id])?;
        Ok(())
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
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        self.insert_row(&library_item)?;
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
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        self.insert_row(&library_item)?;
        Ok(library_item)
    }

    pub fn refresh_fts_tags(&self, library_item_id: &str) -> AppResult<()> {
        let tags = crate::db::TagRepository::new(self.conn).get_for_item(library_item_id)?;
        let tags_text = tags
            .iter()
            .map(|tag| tag.name.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let row = conn.query_row(
            "SELECT title, author_name, platform_item_id, COALESCE(search_keyword, '')
             FROM library_items WHERE id = ?1",
            [library_item_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )?;

        conn.execute(
            "DELETE FROM library_items_fts WHERE library_item_id = ?1",
            [library_item_id],
        )?;
        conn.execute(
            "INSERT INTO library_items_fts (
                library_item_id, title, author_name, platform_item_id, search_keyword, tags
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                library_item_id,
                row.0,
                row.1,
                row.2,
                row.3,
                tags_text
            ],
        )?;
        Ok(())
    }

    fn insert_row(&self, library_item: &LibraryItem) -> AppResult<()> {
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
        conn.execute(
            "INSERT INTO library_items_fts (
                library_item_id, title, author_name, platform_item_id, search_keyword, tags
             ) VALUES (?1, ?2, ?3, ?4, ?5, '')",
            params![
                library_item.id,
                library_item.title,
                library_item.author_name,
                library_item.platform_item_id,
                library_item.search_keyword.clone().unwrap_or_default()
            ],
        )?;
        Ok(())
    }

    fn attach_tags(&self, mut item: LibraryItem) -> AppResult<LibraryItem> {
        let tags = crate::db::TagRepository::new(self.conn).get_for_item(&item.id)?;
        item.tags = tags.into_iter().map(|tag| tag.name).collect();
        Ok(item)
    }
}

fn fts_query(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn delete_local_files(item: &LibraryItem) -> AppResult<()> {
    let mut paths = Vec::new();
    if let Some(cover) = &item.cover_path {
        paths.push(cover.clone());
    }
    if let Some(metadata) = &item.metadata_path {
        paths.push(metadata.clone());
    }
    paths.extend(item.media_paths.clone());
    paths.extend(item.subtitle_paths.clone());

    for path in paths {
        let file = Path::new(&path);
        if file.exists() {
            std::fs::remove_file(file).ok();
        }
        if let Some(parent) = file.parent() {
            if parent.exists() {
                let _ = std::fs::remove_dir(parent);
            }
        }
    }
    Ok(())
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
        tags: vec![],
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fts_query_wraps_tokens() {
        assert_eq!(fts_query("hello world"), "\"hello\"* \"world\"*");
    }
}
