pub const INITIAL_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS library_items (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    platform_item_id TEXT NOT NULL,
    original_url TEXT NOT NULL,
    canonical_url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    author_id TEXT NOT NULL,
    author_name TEXT NOT NULL,
    published_at INTEGER,
    media_type TEXT NOT NULL,
    duration_sec INTEGER,
    cover_path TEXT,
    media_paths TEXT NOT NULL DEFAULT '[]',
    metadata_path TEXT,
    subtitle_paths TEXT NOT NULL DEFAULT '[]',
    file_size INTEGER,
    checksum TEXT,
    search_keyword TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(platform, platform_item_id)
);

CREATE TABLE IF NOT EXISTS download_tasks (
    id TEXT PRIMARY KEY,
    library_item_id TEXT,
    platform TEXT NOT NULL,
    platform_item_id TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    stage TEXT,
    progress REAL NOT NULL DEFAULT 0,
    speed_bps INTEGER,
    retry_count INTEGER NOT NULL DEFAULT 0,
    options_json TEXT,
    error_json TEXT,
    output_dir TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    completed_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_library_items_title ON library_items(title);
CREATE INDEX IF NOT EXISTS idx_library_items_author ON library_items(author_name);
CREATE INDEX IF NOT EXISTS idx_download_tasks_status ON download_tasks(status);
"#;
