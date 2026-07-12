import { Link } from "react-router-dom";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { usePlatformLogin } from "@/hooks/usePlatformLogin";
import type { AppSettings, AuthPlatform } from "@/types";

interface PlatformAuthDialogProps {
  open: boolean;
  platform: AuthPlatform;
  title?: string;
  description?: string;
  onClose: () => void;
  onLoggedIn?: (settings: AppSettings) => void;
}

const PLATFORM_COPY: Record<
  AuthPlatform,
  {
    title: string;
    description: string;
    loginLabel: string;
    cookieField: "douyinCookies" | "bilibiliCookies";
  }
> = {
  bilibili: {
    title: "Bilibili 登录",
    description: "扫码登录后可下载高画质或会员内容。公开视频通常无需登录。",
    loginLabel: "扫码登录",
    cookieField: "bilibiliCookies",
  },
  douyin: {
    title: "抖音登录",
    description: "浏览器登录后可下载更多内容。",
    loginLabel: "浏览器登录",
    cookieField: "douyinCookies",
  },
};

export function PlatformAuthDialog({
  open,
  platform,
  title,
  description,
  onClose,
  onLoggedIn,
}: PlatformAuthDialogProps) {
  const copy = PLATFORM_COPY[platform];
  const {
    loginSession,
    loginMessage,
    isLoginActive,
    startLogin,
    validateMutation,
    startLoginMutation,
  } = usePlatformLogin({
    platform,
    cookieField: copy.cookieField,
    onSaved: (settings) => {
      onLoggedIn?.(settings);
      onClose();
    },
  });

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-md rounded-xl bg-white shadow-xl">
        <div className="flex items-start justify-between gap-3 border-b border-slate-100 px-4 py-3">
          <div>
            <div className="text-sm font-medium text-slate-900">
              {title ?? copy.title}
            </div>
            <p className="mt-1 text-xs text-slate-500">
              {description ?? copy.description}
            </p>
          </div>
          <button
            type="button"
            className="rounded-md p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
            onClick={onClose}
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="space-y-4 px-4 py-4">
          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="sm"
              loading={startLoginMutation.isPending || isLoginActive}
              onClick={startLogin}
              disabled={isLoginActive}
            >
              {startLoginMutation.isPending || isLoginActive
                ? "登录中…"
                : copy.loginLabel}
            </Button>
            <Button
              size="sm"
              variant="secondary"
              loading={validateMutation.isPending}
              onClick={() => validateMutation.mutate()}
            >
              验证状态
            </Button>
          </div>

          {loginSession?.qrImageBase64 ? (
            <div className="flex flex-col items-start gap-2 rounded-md border border-slate-100 bg-slate-50 p-3">
              <div className="text-xs text-slate-500">
                {loginMessage ?? "请使用 B 站 App 扫描二维码"}
              </div>
              <img
                src={`data:image/png;base64,${loginSession.qrImageBase64}`}
                alt="登录二维码"
                className="h-44 w-44 rounded-md border border-slate-200 bg-white p-2"
              />
            </div>
          ) : null}

          {loginMessage && !loginSession?.qrImageBase64 ? (
            <p className="text-xs text-slate-500">{loginMessage}</p>
          ) : null}

          {validateMutation.data ? (
            <Badge tone={validateMutation.data.valid ? "success" : "danger"}>
              {validateMutation.data.message ?? (validateMutation.data.valid ? "已登录" : "未登录")}
            </Badge>
          ) : null}
        </div>

        <div className="border-t border-slate-100 px-4 py-3 text-right">
          <Link
            to="/settings"
            className="text-xs text-slate-500 hover:text-slate-700"
            onClick={onClose}
          >
            在设置中管理登录状态
          </Link>
        </div>
      </div>
    </div>
  );
}
