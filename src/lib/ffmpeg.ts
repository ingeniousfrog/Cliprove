import type { DownloadOptions, MediaItem, MediaType, Platform } from "@/types";

export function requiresFfmpeg(
  assets: string[],
  mediaType?: MediaType,
  platform?: Platform
): boolean {
  if (platform === "douyin") return false;
  if (mediaType === "multipart") return true;
  if (assets.includes("video")) return true;
  return assets.some((asset) => asset.startsWith("part-"));
}

export function downloadOptionsRequireFfmpeg(
  options: DownloadOptions,
  mediaType?: MediaType,
  platform?: Platform
): boolean {
  return requiresFfmpeg(options.assets, mediaType, platform);
}

export function batchItemsRequireFfmpeg(items: MediaItem[]): boolean {
  return items.some((item) =>
    requiresFfmpeg(
      item.mediaType === "image_post"
        ? ["images"]
        : item.platform === "bilibili" || item.platform === "youtube"
          ? ["video", "cover", "metadata", "subtitle"]
          : ["video", "cover", "metadata"],
      item.mediaType,
      item.platform
    )
  );
}
