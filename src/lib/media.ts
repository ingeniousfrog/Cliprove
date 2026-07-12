import type { MediaItem, Platform } from "@/types";

export const SIDECAR_PORT = 18765;

export function normalizeCoverUrl(url?: string | null): string | null {
  if (!url?.trim()) return null;
  const text = url.trim();
  if (text.startsWith("//")) return `https:${text}`;
  if (text.startsWith("http://")) return `https://${text.slice("http://".length)}`;
  if (!text.startsWith("http://") && !text.startsWith("https://")) {
    return `https://${text}`;
  }
  return text;
}

export function proxiedCoverSrc(
  url: string | null | undefined,
  platform: Platform | string
): string | null {
  const normalized = normalizeCoverUrl(url);
  if (!normalized) return null;
  if (platform === "bilibili" || platform === "douyin") {
    const params = new URLSearchParams({
      url: normalized,
      platform: String(platform),
    });
    return `http://127.0.0.1:${SIDECAR_PORT}/v1/proxy/image?${params.toString()}`;
  }
  return normalized;
}

export function bilibiliPlayerUrl(
  item: Pick<MediaItem, "platformItemId" | "previewUrl">
): string {
  if (item.previewUrl) return item.previewUrl;
  const params = new URLSearchParams({
    isOutside: "true",
    bvid: item.platformItemId,
    p: "1",
    high_quality: "1",
    autoplay: "0",
  });
  return `https://player.bilibili.com/player.html?${params.toString()}`;
}

export function youtubePlayerUrl(
  item: Pick<MediaItem, "platformItemId" | "previewUrl">
): string {
  if (item.previewUrl) return item.previewUrl;
  return `https://www.youtube.com/embed/${item.platformItemId}`;
}

export function embeddedPlayerUrl(
  item: Pick<MediaItem, "platform" | "platformItemId" | "previewUrl">
): string | null {
  if (item.platform === "bilibili") return bilibiliPlayerUrl(item);
  if (item.platform === "youtube") return youtubePlayerUrl(item);
  return null;
}
