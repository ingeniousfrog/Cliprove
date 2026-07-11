mod settings;
mod tasks;
mod library;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};

use crate::errors::{AppError, AppResult};

pub use library::LibraryRepository;
pub use settings::SettingsRepository;
pub use tasks::TaskRepository;

const MIGRATION_SQL: &str = include_str!("../../migrations/001_initial.sql");

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
        conn.execute_batch(MIGRATION_SQL)?;

        let applied: Option<i64> = conn
            .query_row(
                "SELECT version FROM schema_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if applied.is_none() {
            let now = chrono::Utc::now().timestamp_millis();
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
                [now],
            )?;
        }

        Ok(())
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
}
