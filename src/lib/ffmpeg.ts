import type { DownloadOptions, MediaItem, MediaType } from "@/types";

export function requiresFfmpeg(
  assets: string[],
  mediaType?: MediaType
): boolean {
  if (mediaType === "multipart") return true;
  if (assets.includes("video")) return true;
  return assets.some((asset) => asset.startsWith("part-"));
}

export function downloadOptionsRequireFfmpeg(
  options: DownloadOptions,
  mediaType?: MediaType
): boolean {
  return requiresFfmpeg(options.assets, mediaType);
}

export function batchItemsRequireFfmpeg(items: MediaItem[]): boolean {
  return items.some((item) =>
    requiresFfmpeg(
      item.mediaType === "image_post"
        ? ["images"]
        : item.platform === "bilibili"
          ? ["video", "cover", "metadata", "subtitle"]
          : ["video", "cover", "metadata"],
      item.mediaType
    )
  );
}
