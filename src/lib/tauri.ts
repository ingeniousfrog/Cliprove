import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  AuthStatus,
  DownloadOptions,
  DownloadSpec,
  DownloadTask,
  LibraryItem,
  MediaItem,
  ParsedMedia,
  Platform,
  SearchPage,
  SearchQuery,
  SidecarHealth,
} from "@/types";

export async function parseLink(url: string): Promise<ParsedMedia> {
  return invoke<ParsedMedia>("parse_link", { url });
}

export async function searchMedia(
  platform: Platform,
  query: SearchQuery,
  cursor?: string
): Promise<SearchPage> {
  return invoke<SearchPage>("search_media", { platform, query, cursor });
}

export async function enqueueDownload(
  item: MediaItem,
  options: DownloadOptions
): Promise<string> {
  return invoke<string>("enqueue_download", { item, options });
}

export async function listTasks(): Promise<DownloadTask[]> {
  return invoke<DownloadTask[]>("list_tasks");
}

export async function taskAction(
  taskId: string,
  action: "pause" | "resume" | "retry" | "cancel"
): Promise<void> {
  return invoke("task_action", { taskId, action });
}

export async function listLibrary(query?: string): Promise<LibraryItem[]> {
  return invoke<LibraryItem[]>("list_library", { query: query ?? null });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(
  settings: Partial<AppSettings>
): Promise<AppSettings> {
  return invoke<AppSettings>("update_settings", { settings });
}

export async function validatePlatformAuth(
  platform: Platform
): Promise<AuthStatus> {
  return invoke<AuthStatus>("validate_platform_auth", { platform });
}

export async function createDownloadSpec(
  item: MediaItem,
  options: DownloadOptions
): Promise<DownloadSpec> {
  return invoke<DownloadSpec>("create_download_spec", { item, options });
}

export async function startSidecar(): Promise<SidecarHealth> {
  return invoke<SidecarHealth>("start_sidecar");
}

export async function sidecarHealth(): Promise<SidecarHealth> {
  return invoke<SidecarHealth>("sidecar_health");
}
