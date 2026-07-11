import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatBytes(bytes?: number): string {
  if (!bytes) return "—";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function formatSpeed(bps?: number): string {
  if (!bps) return "—";
  return `${formatBytes(bps)}/s`;
}

export function formatDuration(seconds?: number): string {
  if (!seconds) return "—";
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function formatDate(ms?: number): string {
  if (!ms) return "—";
  return new Date(ms).toLocaleString("zh-CN");
}

export function platformLabel(platform: string): string {
  switch (platform) {
    case "douyin":
      return "抖音";
    case "bilibili":
      return "Bilibili";
    default:
      return platform;
  }
}

export function statusLabel(status: string): string {
  const map: Record<string, string> = {
    pending: "等待中",
    parsing: "解析中",
    queued: "排队中",
    downloading: "下载中",
    post_processing: "后处理",
    completed: "已完成",
    failed: "失败",
    cancelled: "已取消",
    interrupted: "已中断",
  };
  return map[status] ?? status;
}
