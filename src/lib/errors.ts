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
