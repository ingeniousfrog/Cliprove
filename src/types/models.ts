export type Platform = "douyin" | "bilibili";

export type MediaType = "video" | "image_post" | "audio" | "multipart";

export type TaskStatus =
  | "pending"
  | "parsing"
  | "queued"
  | "downloading"
  | "post_processing"
  | "completed"
  | "failed"
  | "cancelled"
  | "interrupted";

export type ErrorCode =
  | "unsupported_link"
  | "unsupported_content_type"
  | "auth_required"
  | "auth_expired"
  | "verification_required"
  | "rate_limited"
  | "platform_changed"
  | "content_unavailable"
  | "private_content"
  | "region_restricted"
  | "media_url_expired"
  | "network_timeout"
  | "download_incomplete"
  | "ffmpeg_unavailable"
  | "disk_full"
  | "permission_denied"
  | "engine_failure"
  | "unknown";

export interface Author {
  id: string;
  name: string;
  avatarUrl?: string;
}

export interface MediaAsset {
  id: string;
  kind: "video" | "image" | "cover" | "audio" | "subtitle" | "metadata";
  label: string;
  url?: string;
  selected?: boolean;
}

export interface MediaItem {
  platform: Platform;
  platformItemId: string;
  originalUrl: string;
  canonicalUrl: string;
  title: string;
  description?: string;
  author: Author;
  publishedAt?: number;
  mediaType: MediaType;
  durationSec?: number;
  coverUrl?: string;
  previewUrl?: string;
  searchKeyword?: string;
}

export interface ParsedMedia {
  item: MediaItem;
  assets: MediaAsset[];
  qualities?: QualityOption[];
}

export interface QualityOption {
  id: string;
  label: string;
  height?: number;
}

export type SearchFilterKey = "sort" | "publish_time" | "media_type";

export interface SearchQuery {
  keyword: string;
  filters?: Partial<Record<SearchFilterKey, string>>;
  pageSize?: number;
}

export interface SearchPage {
  items: MediaItem[];
  cursor?: string;
  hasMore: boolean;
  supportedFilters: SearchFilterKey[];
}

export interface DownloadOptions {
  assets: string[];
  qualityId?: string;
  saveCover?: boolean;
  saveAudio?: boolean;
  saveMetadata?: boolean;
  saveSubtitles?: boolean;
  forceReplace?: boolean;
}

export interface DownloadAsset {
  id: string;
  kind: MediaAsset["kind"];
  sourceUrl: string;
  outputPath: string;
}

export interface DownloadSpec {
  item: MediaItem;
  assets: DownloadAsset[];
  requiresFfmpeg?: boolean;
}

export interface StructuredError {
  code: ErrorCode;
  message: string;
  suggestion?: string;
  technicalDetail?: string;
}

export interface DownloadTask {
  id: string;
  platform: Platform;
  platformItemId: string;
  title: string;
  status: TaskStatus;
  stage?: string;
  progress: number;
  speedBps?: number;
  retryCount: number;
  error?: StructuredError;
  outputDir?: string;
  libraryItemId?: string;
  createdAt: number;
  updatedAt: number;
  completedAt?: number;
}

export interface DownloadProgress {
  taskId: string;
  stage: string;
  progress: number;
  speedBps?: number;
  retryCount: number;
}

export interface LibraryItem {
  id: string;
  platform: Platform;
  platformItemId: string;
  originalUrl: string;
  canonicalUrl: string;
  title: string;
  description?: string;
  authorId: string;
  authorName: string;
  publishedAt?: number;
  mediaType: MediaType;
  durationSec?: number;
  coverPath?: string;
  mediaPaths: string[];
  metadataPath?: string;
  subtitlePaths: string[];
  fileSize?: number;
  checksum?: string;
  searchKeyword?: string;
  tags: string[];
  createdAt: number;
  updatedAt: number;
}

export interface LibraryFilter {
  query?: string;
  platform?: Platform;
  mediaType?: MediaType;
  dateFrom?: number;
  dateTo?: number;
  collectionId?: string;
  tagId?: string;
}

export interface Tag {
  id: string;
  name: string;
  createdAt: number;
}

export interface Collection {
  id: string;
  name: string;
  itemCount: number;
  createdAt: number;
  updatedAt: number;
}

export interface AppPaths {
  databasePath: string;
  downloadDirectory: string;
  logDirectory: string;
}

export interface FfmpegStatus {
  valid: boolean;
  message: string;
  resolvedPath?: string;
}

export interface AuthStatus {
  platform: Platform;
  valid: boolean;
  message?: string;
}

export type PlatformLoginStatus =
  | "pending"
  | "scanned"
  | "confirmed"
  | "completed"
  | "failed"
  | "expired";

export interface PlatformLoginSession {
  sessionId: string;
  platform: Platform;
  status: PlatformLoginStatus | string;
  message?: string;
  qrImageBase64?: string;
  cookies?: string;
}

export interface AuthenticationProfile {
  platform: Platform;
  cookies?: string;
  updatedAt?: number;
}

export interface AppSettings {
  downloadDirectory: string;
  filenameTemplate: string;
  maxConcurrentDownloads: number;
  retryCount: number;
  ffmpegPath: string;
  douyinCookies: string;
  bilibiliCookies: string;
  saveMetadata: boolean;
  saveCover: boolean;
  saveAudio: boolean;
  saveSubtitles: boolean;
  clipboardDetect: boolean;
  onboardingCompleted: boolean;
}

export interface SidecarHealth {
  status: string;
  version?: string;
}

export interface BatchEnqueueItemResult {
  platformItemId: string;
  status: "enqueued" | "skipped" | "failed" | string;
  taskId?: string;
  message?: string;
}

export interface BatchEnqueueResult {
  results: BatchEnqueueItemResult[];
  enqueuedCount: number;
}
