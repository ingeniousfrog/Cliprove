use std::time::Duration;

use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AppSettings, AuthStatus, DownloadOptions, MediaItem, ParsedMedia, SearchPage, SearchQuery,
    SidecarHealth,
};

#[derive(Clone)]
pub struct SidecarClient {
    base_url: String,
    http: Client,
}

impl SidecarClient {
    pub fn new(port: u16) -> AppResult<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self {
            base_url: format!("http://127.0.0.1:{port}"),
            http,
        })
    }

    pub fn health(&self) -> AppResult<SidecarHealth> {
        self.get("/health")
    }

    pub fn parse_link(&self, url: &str, settings: &AppSettings) -> AppResult<ParsedMedia> {
        let body = serde_json::json!({
            "url": url,
            "cookies": settings.douyin_cookies,
            "proxy": ""
        });
        self.post("/v1/parse", &body)
    }

    pub fn start_download(
        &self,
        item: &MediaItem,
        options: &DownloadOptions,
        output_dir: &str,
        settings: &AppSettings,
    ) -> AppResult<SidecarJob> {
        let body = serde_json::json!({
            "platform": item.platform,
            "platformItemId": item.platform_item_id,
            "outputDir": output_dir,
            "assetIds": options.assets,
            "cookies": settings.douyin_cookies,
            "proxy": ""
        });
        self.post("/v1/download", &body)
    }

    pub fn get_job(&self, job_id: &str) -> AppResult<SidecarJob> {
        self.get(&format!("/v1/jobs/{job_id}"))
    }

    pub fn search_media(
        &self,
        platform: &str,
        query: &SearchQuery,
        cursor: Option<&str>,
        settings: &AppSettings,
    ) -> AppResult<SearchPage> {
        let body = serde_json::json!({
            "platform": platform,
            "keyword": query.keyword,
            "cursor": cursor,
            "pageSize": query.page_size.unwrap_or(20),
            "filters": query.filters,
            "cookies": settings.douyin_cookies,
            "proxy": ""
        });
        self.post("/v1/search", &body)
    }

    pub fn validate_auth(&self, platform: &str, settings: &AppSettings) -> AppResult<AuthStatus> {
        let cookies = match platform {
            "douyin" => settings.douyin_cookies.clone(),
            "bilibili" => settings.bilibili_cookies.clone(),
            _ => String::new(),
        };
        let body = serde_json::json!({
            "platform": platform,
            "cookies": cookies,
            "proxy": ""
        });
        self.post("/v1/auth/validate", &body)
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> AppResult<T> {
        let response = self.http.get(format!("{}{}", self.base_url, path)).send()?;
        self.decode(response)
    }

    fn post<T: DeserializeOwned>(&self, path: &str, body: &Value) -> AppResult<T> {
        let response = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .json(body)
            .send()?;
        self.decode(response)
    }

    fn decode<T: DeserializeOwned>(&self, response: reqwest::blocking::Response) -> AppResult<T> {
        if response.status().is_success() {
            return Ok(response.json()?);
        }

        let status = response.status();
        let text = response.text().unwrap_or_default();
        let detail = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|json| json.get("detail").and_then(|v| v.as_str()).map(str::to_string))
            .unwrap_or(text);

        Err(AppError::structured(
            if status.as_u16() == 400 {
                "unsupported_link"
            } else {
                "engine_failure"
            },
            detail,
            Some("检查 Sidecar 日志与 Cookie 配置".to_string()),
        ))
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarJob {
    pub job_id: String,
    pub status: String,
    pub stage: String,
    pub progress: f64,
    pub error: Option<String>,
    pub result: Option<SidecarDownloadResult>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarDownloadResult {
    pub output_dir: String,
    pub media_paths: Vec<String>,
    pub cover_path: Option<String>,
    pub metadata_path: Option<String>,
    pub subtitle_paths: Vec<String>,
    pub file_size: Option<i64>,
}
