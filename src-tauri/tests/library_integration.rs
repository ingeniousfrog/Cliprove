use cliprove_lib::db::Database;
use cliprove_lib::models::{LibraryFilter, MediaItem, Author};

#[test]
fn library_fts_search_and_delete() {
    let temp = tempfile::tempdir().expect("tempdir");
    let db = Database::open(temp.path()).expect("open db");

    let item = MediaItem {
        platform: "douyin".to_string(),
        platform_item_id: "123".to_string(),
        original_url: "https://example.com".to_string(),
        canonical_url: "https://example.com".to_string(),
        title: "测试视频标题".to_string(),
        description: None,
        author: Author {
            id: "author1".to_string(),
            name: "测试作者".to_string(),
            avatar_url: None,
        },
        published_at: None,
        media_type: "video".to_string(),
        duration_sec: Some(60),
        cover_url: None,
        search_keyword: Some("关键词".to_string()),
    };

    let library_item = db
        .library()
        .insert_from_media(&item, "/tmp/mock-output")
        .expect("insert");

    let tag = db.tags().create("收藏").expect("create tag");
    db.tags()
        .set_for_item(&library_item.id, &[tag.id.clone()])
        .expect("set tags");
    db.library()
        .refresh_fts_tags(&library_item.id)
        .expect("refresh fts");

    let results = db
        .library()
        .list(&LibraryFilter {
            query: Some("测试作者".to_string()),
            ..Default::default()
        })
        .expect("search");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, library_item.id);

    db.library()
        .delete(&library_item.id, false)
        .expect("delete");
    let after = db.library().list(&LibraryFilter::default()).expect("list");
    assert!(after.is_empty());
}
