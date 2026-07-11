mod settings;
mod tasks;
mod library;
mod tags;
mod collections;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::errors::{AppError, AppResult};

pub use collections::CollectionRepository;
pub use library::LibraryRepository;
pub use settings::SettingsRepository;
pub use tags::TagRepository;
pub use tasks::TaskRepository;

const MIGRATIONS: &[(&str, i64)] = &[
    (include_str!("../../migrations/001_initial.sql"), 1),
    (include_str!("../../migrations/002_library_phase4.sql"), 2),
];

pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    pub fn open(app_data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(app_data_dir)?;
        let path = app_data_dir.join("cliprove.db");
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let db = Self {
            conn: Mutex::new(conn),
            path,
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn migrate(&self) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|_| {
            AppError::Message("database lock poisoned".to_string())
        })?;

        for (sql, version) in MIGRATIONS {
            if self.is_migration_applied(&conn, *version)? {
                continue;
            }

            conn.execute_batch(sql)?;
            let now = chrono::Utc::now().timestamp_millis();
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![version, now],
            )?;
        }

        Ok(())
    }

    fn is_migration_applied(&self, conn: &Connection, version: i64) -> AppResult<bool> {
        let table_exists: i64 = conn.query_row(
            "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )?;
        if table_exists == 0 {
            return Ok(false);
        }

        let applied: Option<i64> = conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )
            .optional()?;
        Ok(applied.is_some())
    }

    pub fn settings(&self) -> SettingsRepository<'_> {
        SettingsRepository::new(&self.conn)
    }

    pub fn tasks(&self) -> TaskRepository<'_> {
        TaskRepository::new(&self.conn)
    }

    pub fn library(&self) -> LibraryRepository<'_> {
        LibraryRepository::new(&self.conn)
    }

    pub fn tags(&self) -> TagRepository<'_> {
        TagRepository::new(&self.conn)
    }

    pub fn collections(&self) -> CollectionRepository<'_> {
        CollectionRepository::new(&self.conn)
    }
}
