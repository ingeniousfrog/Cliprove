ALTER TABLE download_tasks ADD COLUMN item_json TEXT;

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS item_tags (
    library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (library_item_id, tag_id)
);

CREATE TABLE IF NOT EXISTS collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS collection_items (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    library_item_id TEXT NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (collection_id, library_item_id)
);

CREATE INDEX IF NOT EXISTS idx_item_tags_tag ON item_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_collection_items_collection ON collection_items(collection_id);
CREATE INDEX IF NOT EXISTS idx_library_items_platform ON library_items(platform);
CREATE INDEX IF NOT EXISTS idx_library_items_media_type ON library_items(media_type);
CREATE INDEX IF NOT EXISTS idx_library_items_created_at ON library_items(created_at);

CREATE VIRTUAL TABLE IF NOT EXISTS library_items_fts USING fts5(
    library_item_id UNINDEXED,
    title,
    author_name,
    platform_item_id,
    search_keyword,
    tags,
    tokenize = 'unicode61'
);

INSERT INTO library_items_fts(library_item_id, title, author_name, platform_item_id, search_keyword, tags)
SELECT id, title, author_name, platform_item_id, COALESCE(search_keyword, ''), ''
FROM library_items;
