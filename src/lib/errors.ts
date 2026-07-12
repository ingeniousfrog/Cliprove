import { formatInvokeError } from "@/lib/utils";
import type { ErrorCode } from "@/types";

const ERROR_PREFIX = /^CLIPROVE_([A-Z_]+):\s*/i;

export function parseErrorCode(error: unknown): ErrorCode | null {
  const message = formatInvokeError(error);
  const match = message.match(ERROR_PREFIX);
  if (match) {
    const code = match[1].toLowerCase();
    if (code === "ffmpeg_unavailable") return "ffmpeg_unavailable";
    if (code === "auth_required") return "auth_required";
    if (code === "auth_expired") return "auth_expired";
    if (code === "verification_required") return "verification_required";
    if (code === "region_restricted") return "region_restricted";
  }

  const lowered = message.toLowerCase();
  if (lowered.includes("ffmpeg")) return "ffmpeg_unavailable";
  if (
    lowered.includes("sessdata") ||
    lowered.includes("login") ||
    lowered.includes("sign in") ||
    lowered.includes("登录") ||
    lowered.includes("cookie")
  ) {
    if (lowered.includes("expired") || lowered.includes("过期") || lowered.includes("失效")) {
      return "auth_expired";
    }
    return "auth_required";
  }
  if (
    lowered.includes("not available in your country") ||
    lowered.includes("not made this video available") ||
    lowered.includes("geo-restricted") ||
    lowered.includes("地区不可用") ||
    lowered.includes("区域不可用")
  ) {
    return "region_restricted";
  }
  return null;
}

export function isAuthErrorCode(code: ErrorCode | null): boolean {
  return (
    code === "auth_required" ||
    code === "auth_expired" ||
    code === "verification_required"
  );
}

export function stripErrorPrefix(message: string): string {
  return message.replace(ERROR_PREFIX, "").trim();
}

export function formatKnownError(error: unknown): string {
  const rawMessage = formatInvokeError(error);
  const code = parseErrorCode(rawMessage);
  const message = stripErrorPrefix(rawMessage);

  if (code === "region_restricted") {
    return (
      message ||
      "该视频对当前网络所在地区不可用，请换一个视频，或切换到允许访问该视频的网络后重试"
    );
  }

  return message || "操作失败，请稍后重试";
}
