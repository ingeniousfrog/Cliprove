use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::errors::{AppError, AppResult};
use crate::models::Tag;

pub struct TagRepository<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> TagRepository<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> AppResult<Vec<Tag>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at FROM tags ORDER BY name COLLATE NOCASE ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn create(&self, name: &str) -> AppResult<Tag> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Message("标签名称不能为空".to_string()));
        }

        let tag = Tag {
            id: uuid::Uuid::new_v4().to_string(),
            name: trimmed.to_string(),
            created_at: chrono::Utc::now().timestamp_millis(),
        };

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "INSERT INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![tag.id, tag.name, tag.created_at],
        )?;

        Ok(tag)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_for_item(&self, library_item_id: &str) -> AppResult<Vec<Tag>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.created_at
             FROM tags t
             INNER JOIN item_tags it ON it.tag_id = t.id
             WHERE it.library_item_id = ?1
             ORDER BY t.name COLLATE NOCASE ASC",
        )?;
        let rows = stmt.query_map([library_item_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn set_for_item(&self, library_item_id: &str, tag_ids: &[String]) -> AppResult<Vec<Tag>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;

        conn.execute(
            "DELETE FROM item_tags WHERE library_item_id = ?1",
            [library_item_id],
        )?;

        for tag_id in tag_ids {
            conn.execute(
                "INSERT INTO item_tags (library_item_id, tag_id) VALUES (?1, ?2)",
                params![library_item_id, tag_id],
            )?;
        }

        drop(conn);
        self.get_for_item(library_item_id)
    }
}
