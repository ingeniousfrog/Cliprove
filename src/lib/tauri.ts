import { invoke } from "@tauri-apps/api/core";
import type {
  AppPaths,
  AppSettings,
  AuthStatus,
  Collection,
  DownloadOptions,
  DownloadSpec,
  DownloadTask,
  LibraryFilter,
  LibraryItem,
  MediaItem,
  ParsedMedia,
  Platform,
  SearchPage,
  SearchQuery,
  SidecarHealth,
  Tag,
  FfmpegStatus,
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

export async function listLibrary(
  filter?: LibraryFilter
): Promise<LibraryItem[]> {
  return invoke<LibraryItem[]>("list_library", { filter: filter ?? null });
}

export async function getLibraryItem(id: string): Promise<LibraryItem> {
  return invoke<LibraryItem>("get_library_item", { id });
}

export async function deleteLibraryItem(
  id: string,
  deleteFiles: boolean
): Promise<void> {
  return invoke("delete_library_item", { id, deleteFiles });
}

export async function listTags(): Promise<Tag[]> {
  return invoke<Tag[]>("list_tags");
}

export async function createTag(name: string): Promise<Tag> {
  return invoke<Tag>("create_tag", { name });
}

export async function deleteTag(id: string): Promise<void> {
  return invoke("delete_tag", { id });
}

export async function setLibraryTags(
  libraryItemId: string,
  tagIds: string[]
): Promise<string[]> {
  return invoke<string[]>("set_library_tags", { libraryItemId, tagIds });
}

export async function listCollections(): Promise<Collection[]> {
  return invoke<Collection[]>("list_collections");
}

export async function createCollection(name: string): Promise<Collection> {
  return invoke<Collection>("create_collection", { name });
}

export async function renameCollection(
  id: string,
  name: string
): Promise<Collection> {
  return invoke<Collection>("rename_collection", { id, name });
}

export async function deleteCollection(id: string): Promise<void> {
  return invoke("delete_collection", { id });
}

export async function addToCollection(
  collectionId: string,
  libraryItemId: string
): Promise<void> {
  return invoke("add_to_collection", { collectionId, libraryItemId });
}

export async function removeFromCollection(
  collectionId: string,
  libraryItemId: string
): Promise<void> {
  return invoke("remove_from_collection", { collectionId, libraryItemId });
}

export async function revealInFinder(path: string): Promise<void> {
  return invoke("reveal_in_finder", { path });
}

export async function openLocalFile(path: string): Promise<void> {
  return invoke("open_local_file", { path });
}

export async function readLocalFile(path: string): Promise<string> {
  return invoke<string>("read_local_file", { path });
}

export async function validateFfmpeg(path: string): Promise<FfmpegStatus> {
  return invoke<FfmpegStatus>("validate_ffmpeg", { path });
}

export async function getAppPaths(): Promise<AppPaths> {
  return invoke<AppPaths>("get_app_paths");
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
