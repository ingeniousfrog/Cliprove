use crate::errors::{AppError, AppResult};
use crate::models::{
    AuthStatus, DownloadAsset, DownloadOptions, DownloadSpec, MediaAsset, MediaItem, ParsedMedia,
    QualityOption, SearchPage, SearchQuery,
};

pub fn detect_platform(input: &str) -> AppResult<&'static str> {
    let value = input.trim();
    if value.is_empty() {
        return Err(AppError::structured(
            "unsupported_link",
            "请输入有效链接",
            Some("粘贴抖音或 Bilibili 分享链接".to_string()),
        ));
    }

    if is_douyin(value) {
        return Ok("douyin");
    }
    if is_bilibili(value) {
        return Ok("bilibili");
    }

    Err(AppError::structured(
        "unsupported_link",
        "无法识别该平台链接",
        Some("当前 Mock 模式仅支持抖音与 Bilibili".to_string()),
    ))
}

pub fn parse_link(url: &str) -> AppResult<ParsedMedia> {
    let platform = detect_platform(url)?;
    match platform {
        "douyin" => Ok(mock_douyin_parsed(url)),
        "bilibili" => Ok(mock_bilibili_parsed(url)),
        _ => unreachable!(),
    }
}

pub fn search(platform: &str, query: &SearchQuery, cursor: Option<&str>) -> AppResult<SearchPage> {
    let offset = cursor.unwrap_or("0").parse::<usize>().unwrap_or(0);
    let page_size = query.page_size.unwrap_or(12).clamp(1, 50) as usize;
    let items = match platform {
        "douyin" => mock_search_items("douyin", &query.keyword, offset, page_size),
        "bilibili" => mock_search_items("bilibili", &query.keyword, offset, page_size),
        _ => {
            return Err(AppError::structured(
                "unsupported_link",
                "不支持的平台",
                None,
            ))
        }
    };

    let next_offset = offset + page_size;
    let has_more = next_offset < 36;

    Ok(SearchPage {
        items,
        cursor: if has_more {
            Some(next_offset.to_string())
        } else {
            None
        },
        has_more,
        supported_filters: match platform {
            "douyin" => vec!["sort".to_string(), "publish_time".to_string()],
            "bilibili" => vec!["sort".to_string(), "media_type".to_string()],
            _ => vec![],
        },
    })
}

pub fn create_download_spec(item: &MediaItem, options: &DownloadOptions) -> AppResult<DownloadSpec> {
    let output_base = format!(
        "{}/{}/{}",
        item.platform, item.author.id, item.platform_item_id
    );

    let mut assets = Vec::new();
    for asset_id in &options.assets {
        let kind = match asset_id.as_str() {
            "video" => "video",
            "cover" => "cover",
            "audio" => "audio",
            "metadata" => "metadata",
            "subtitle" => "subtitle",
            _ => continue,
        };
        assets.push(DownloadAsset {
            id: asset_id.clone(),
            kind: kind.to_string(),
            source_url: format!("https://mock.cliprove.local/{}/{}", item.platform, asset_id),
            output_path: format!("{output_base}/{asset_id}.{}", ext_for_kind(kind)),
        });
    }

    Ok(DownloadSpec {
        item: item.clone(),
        assets,
        requires_ffmpeg: Some(item.media_type == "multipart"),
    })
}

pub fn validate_auth(platform: &str, cookies: &str) -> AuthStatus {
    let valid = !cookies.trim().is_empty();
    AuthStatus {
        platform: platform.to_string(),
        valid,
        message: Some(if valid {
            "Mock：Cookie 格式看起来有效".to_string()
        } else {
            "Mock：请配置 Cookie 后再验证".to_string()
        }),
    }
}

fn is_douyin(value: &str) -> bool {
    value.contains("douyin.com")
        || value.contains("iesdouyin.com")
        || value.contains("v.douyin.com")
}

fn is_bilibili(value: &str) -> bool {
    value.contains("bilibili.com")
        || value.contains("b23.tv")
        || value.starts_with("BV")
        || value.starts_with("bv")
}

fn mock_douyin_parsed(url: &str) -> ParsedMedia {
    ParsedMedia {
        item: MediaItem {
            platform: "douyin".to_string(),
            platform_item_id: "7604129988555574538".to_string(),
            original_url: url.to_string(),
            canonical_url: "https://www.douyin.com/video/7604129988555574538".to_string(),
            title: "Mock 抖音视频：城市夜景延时".to_string(),
            description: Some("Phase 0 mock 数据，用于验证解析与下载流程。".to_string()),
            author: author("MS4wLjABAAAAmock", "旅行摄影机"),
            published_at: Some(1_704_067_200_000),
            media_type: "video".to_string(),
            duration_sec: Some(42),
            cover_url: Some("https://picsum.photos/seed/douyin-mock/640/360".to_string()),
            preview_url: None,
            search_keyword: None,
        },
        assets: default_assets(true),
        qualities: Some(vec![QualityOption {
            id: "best".to_string(),
            label: "最高画质".to_string(),
            height: Some(1080),
        }]),
    }
}

fn mock_bilibili_parsed(url: &str) -> ParsedMedia {
    ParsedMedia {
        item: MediaItem {
            platform: "bilibili".to_string(),
            platform_item_id: "BV1mock12345".to_string(),
            original_url: url.to_string(),
            canonical_url: "https://www.bilibili.com/video/BV1mock12345".to_string(),
            title: "Mock Bilibili 视频：开源工具合集".to_string(),
            description: Some("Phase 0 mock 数据，包含多清晰度与字幕选项。".to_string()),
            author: author("12345678", "技术观察员"),
            published_at: Some(1_703_000_000_000),
            media_type: "video".to_string(),
            duration_sec: Some(612),
            cover_url: Some("https://picsum.photos/seed/bilibili-mock/640/360".to_string()),
            preview_url: Some(
                "https://player.bilibili.com/player.html?isOutside=true&bvid=BV1mock12345&p=1&high_quality=1&autoplay=0"
                    .to_string(),
            ),
            search_keyword: None,
        },
        assets: default_assets(true),
        qualities: Some(vec![
            QualityOption {
                id: "1080p".to_string(),
                label: "1080P".to_string(),
                height: Some(1080),
            },
            QualityOption {
                id: "720p".to_string(),
                label: "720P".to_string(),
                height: Some(720),
            },
        ]),
    }
}

fn mock_search_items(
    platform: &str,
    keyword: &str,
    offset: usize,
    page_size: usize,
) -> Vec<MediaItem> {
    (0..page_size)
        .map(|index| {
            let serial = offset + index + 1;
            MediaItem {
                platform: platform.to_string(),
                platform_item_id: format!("{platform}-search-{serial:03}"),
                original_url: format!("https://example.com/{platform}/{serial}"),
                canonical_url: format!("https://example.com/{platform}/{serial}"),
                title: format!("{keyword} · Mock 结果 #{serial}"),
                description: Some(format!("来自 {platform} 的 mock 搜索结果")),
                author: author(
                    &format!("author-{serial}"),
                    &format!("作者 {serial}"),
                ),
                published_at: Some(1_704_000_000_000 + serial as i64 * 86_400_000),
                media_type: "video".to_string(),
                duration_sec: Some(30 + (serial as i64 % 120)),
                cover_url: Some(format!(
                    "https://picsum.photos/seed/{platform}-{serial}/640/360"
                )),
                preview_url: None,
                search_keyword: Some(keyword.to_string()),
            }
        })
        .collect()
}

fn author(id: &str, name: &str) -> crate::models::Author {
    crate::models::Author {
        id: id.to_string(),
        name: name.to_string(),
        avatar_url: Some(format!("https://picsum.photos/seed/{id}/64/64")),
    }
}

fn default_assets(include_subtitle: bool) -> Vec<MediaAsset> {
    let mut assets = vec![
        MediaAsset {
            id: "video".to_string(),
            kind: "video".to_string(),
            label: "视频".to_string(),
            url: None,
            selected: Some(true),
        },
        MediaAsset {
            id: "cover".to_string(),
            kind: "cover".to_string(),
            label: "封面".to_string(),
            url: None,
            selected: Some(true),
        },
        MediaAsset {
            id: "audio".to_string(),
            kind: "audio".to_string(),
            label: "音频".to_string(),
            url: None,
            selected: Some(false),
        },
        MediaAsset {
            id: "metadata".to_string(),
            kind: "metadata".to_string(),
            label: "元数据".to_string(),
            url: None,
            selected: Some(true),
        },
    ];

    if include_subtitle {
        assets.push(MediaAsset {
            id: "subtitle".to_string(),
            kind: "subtitle".to_string(),
            label: "字幕".to_string(),
            url: None,
            selected: Some(false),
        });
    }

    assets
}

fn ext_for_kind(kind: &str) -> &'static str {
    match kind {
        "video" => "mp4",
        "cover" => "jpg",
        "audio" => "mp3",
        "metadata" => "json",
        "subtitle" => "srt",
        _ => "bin",
    }
}
