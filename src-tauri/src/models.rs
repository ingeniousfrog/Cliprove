use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub url: Option<String>,
    pub selected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub platform: String,
    pub platform_item_id: String,
    pub original_url: String,
    pub canonical_url: String,
    pub title: String,
    pub description: Option<String>,
    pub author: Author,
    pub published_at: Option<i64>,
    pub media_type: String,
    pub duration_sec: Option<i64>,
    pub cover_url: Option<String>,
    pub preview_url: Option<String>,
    pub search_keyword: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityOption {
    pub id: String,
    pub label: String,
    pub height: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedMedia {
    pub item: MediaItem,
    pub assets: Vec<MediaAsset>,
    pub qualities: Option<Vec<QualityOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub keyword: String,
    pub filters: Option<serde_json::Value>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub items: Vec<MediaItem>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub supported_filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    pub assets: Vec<String>,
    pub quality_id: Option<String>,
    pub save_cover: Option<bool>,
    pub save_audio: Option<bool>,
    pub save_metadata: Option<bool>,
    pub save_subtitles: Option<bool>,
    pub force_replace: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadAsset {
    pub id: String,
    pub kind: String,
    pub source_url: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSpec {
    pub item: MediaItem,
    pub assets: Vec<DownloadAsset>,
    pub requires_ffmpeg: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredError {
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub technical_detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub platform: String,
    pub platform_item_id: String,
    pub title: String,
    pub status: String,
    pub stage: Option<String>,
    pub progress: f64,
    pub speed_bps: Option<i64>,
    pub retry_count: i64,
    pub error: Option<StructuredError>,
    pub output_dir: Option<String>,
    pub library_item_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: String,
    pub platform: String,
    pub platform_item_id: String,
    pub original_url: String,
    pub canonical_url: String,
    pub title: String,
    pub description: Option<String>,
    pub author_id: String,
    pub author_name: String,
    pub published_at: Option<i64>,
    pub media_type: String,
    pub duration_sec: Option<i64>,
    pub cover_path: Option<String>,
    pub media_paths: Vec<String>,
    pub metadata_path: Option<String>,
    pub subtitle_paths: Vec<String>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub search_keyword: Option<String>,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFilter {
    pub query: Option<String>,
    pub platform: Option<String>,
    pub media_type: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub collection_id: Option<String>,
    pub tag_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub item_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub platform: String,
    pub valid: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformLoginSession {
    pub session_id: String,
    pub platform: String,
    pub status: String,
    pub message: Option<String>,
    pub qr_image_base64: Option<String>,
    pub cookies: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub download_directory: String,
    pub filename_template: String,
    pub max_concurrent_downloads: i64,
    pub retry_count: i64,
    pub ffmpeg_path: String,
    pub douyin_cookies: String,
    pub bilibili_cookies: String,
    pub save_metadata: bool,
    pub save_cover: bool,
    pub save_audio: bool,
    pub save_subtitles: bool,
    pub clipboard_detect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarHealth {
    pub status: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegStatus {
    pub valid: bool,
    pub message: String,
    pub resolved_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_directory = dirs::download_dir()
            .map(|path| path.join("Cliprove Library").to_string_lossy().to_string())
            .unwrap_or_else(|| "./Cliprove Library".to_string());

        Self {
            download_directory,
            filename_template: "{platform}_{author}_{title}_{id}".to_string(),
            max_concurrent_downloads: 3,
            retry_count: 3,
            ffmpeg_path: "ffmpeg".to_string(),
            douyin_cookies: String::new(),
            bilibili_cookies: String::new(),
            save_metadata: true,
            save_cover: true,
            save_audio: false,
            save_subtitles: true,
            clipboard_detect: false,
        }
    }
}
