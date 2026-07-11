use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::errors::{AppError, AppResult};
use crate::models::Collection;

pub struct CollectionRepository<'a> {
    conn: &'a Mutex<Connection>,
}

impl<'a> CollectionRepository<'a> {
    pub fn new(conn: &'a Mutex<Connection>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> AppResult<Vec<Collection>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.created_at, c.updated_at,
                    (SELECT COUNT(1) FROM collection_items ci WHERE ci.collection_id = c.id) AS item_count
             FROM collections c
             ORDER BY c.updated_at DESC",
        )?;
        let rows = stmt.query_map([], map_collection_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn create(&self, name: &str) -> AppResult<Collection> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Message("收藏夹名称不能为空".to_string()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        let collection = Collection {
            id: uuid::Uuid::new_v4().to_string(),
            name: trimmed.to_string(),
            item_count: 0,
            created_at: now,
            updated_at: now,
        };

        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "INSERT INTO collections (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                collection.id,
                collection.name,
                collection.created_at,
                collection.updated_at
            ],
        )?;

        Ok(collection)
    }

    pub fn rename(&self, id: &str, name: &str) -> AppResult<Collection> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::Message("收藏夹名称不能为空".to_string()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "UPDATE collections SET name = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, trimmed, now],
        )?;

        self.get(id)?
            .ok_or_else(|| AppError::Message("收藏夹不存在".to_string()))
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute("DELETE FROM collections WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn add_item(&self, collection_id: &str, library_item_id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "INSERT OR IGNORE INTO collection_items (collection_id, library_item_id, added_at)
             VALUES (?1, ?2, ?3)",
            params![collection_id, library_item_id, now],
        )?;
        conn.execute(
            "UPDATE collections SET updated_at = ?2 WHERE id = ?1",
            params![collection_id, now],
        )?;
        Ok(())
    }

    pub fn remove_item(&self, collection_id: &str, library_item_id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.execute(
            "DELETE FROM collection_items WHERE collection_id = ?1 AND library_item_id = ?2",
            params![collection_id, library_item_id],
        )?;
        conn.execute(
            "UPDATE collections SET updated_at = ?2 WHERE id = ?1",
            params![collection_id, now],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Collection>> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;
        conn.query_row(
            "SELECT c.id, c.name, c.created_at, c.updated_at,
                    (SELECT COUNT(1) FROM collection_items ci WHERE ci.collection_id = c.id) AS item_count
             FROM collections c WHERE c.id = ?1",
            [id],
            map_collection_row,
        )
        .optional()
        .map_err(AppError::from)
    }
}

fn map_collection_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Collection> {
    Ok(Collection {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
        item_count: row.get(4)?,
    })
}
