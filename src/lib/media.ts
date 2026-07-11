import type { Platform } from "@/types";

export const SIDECAR_PORT = 18765;

export function normalizeCoverUrl(url?: string | null): string | null {
  if (!url?.trim()) return null;
  const text = url.trim();
  if (text.startsWith("//")) return `https:${text}`;
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

export function bilibiliPlayerUrl(platformItemId: string): string {
  return `https://player.bilibili.com/player.html?bvid=${encodeURIComponent(
    platformItemId
  )}&high_quality=1&autoplay=0`;
}

export function canEmbedPreview(platform: Platform | string): boolean {
  return platform === "bilibili";
}
