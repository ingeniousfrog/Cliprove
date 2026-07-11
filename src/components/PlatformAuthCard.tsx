import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { usePlatformLogin } from "@/hooks/usePlatformLogin";
import type { AppSettings, Platform } from "@/types";

interface PlatformAuthCardProps {
  platform: Platform;
  title: string;
  description: string;
  loginLabel: string;
  cookieField: "douyinCookies" | "bilibiliCookies";
  draft: AppSettings;
  onDraftChange: (settings: AppSettings) => void;
  onSaved: (settings: AppSettings) => void;
}

export function PlatformAuthCard({
  platform,
  title,
  description,
  loginLabel,
  cookieField,
  draft,
  onDraftChange,
  onSaved,
}: PlatformAuthCardProps) {
  const [showAdvanced, setShowAdvanced] = useState(false);
  const {
    loginSession,
    loginMessage,
    isLoginActive,
    startLogin,
    validateMutation,
    startLoginMutation,
  } = usePlatformLogin({
    platform,
    cookieField,
    onSaved: (settings) => {
      onDraftChange(settings);
      onSaved(settings);
    },
  });

  const hasCookies = Boolean(draft[cookieField].trim());
  const configured = validateMutation.data
    ? validateMutation.data.valid && hasCookies
    : hasCookies;

  const handleStartLogin = () => {
    startLogin();
  };

  return (
    <div className="space-y-3 rounded-md border border-slate-100 p-3">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-medium text-slate-800">{title}</div>
          <p className="mt-1 text-xs text-slate-500">{description}</p>
        </div>
        <Badge
          tone={
            validateMutation.data?.valid
              ? "success"
              : validateMutation.data && draft[cookieField].trim()
                ? "warning"
                : configured
                  ? "success"
                  : "default"
          }
        >
          {validateMutation.data?.valid
            ? "已配置"
            : validateMutation.data && draft[cookieField].trim()
              ? "搜索受限"
              : configured
                ? "已配置"
                : "未配置"}
        </Badge>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          loading={startLoginMutation.isPending || isLoginActive}
          onClick={handleStartLogin}
          disabled={isLoginActive}
        >
          {startLoginMutation.isPending || isLoginActive
            ? "登录中…"
            : platform === "douyin" && configured
              ? "重新登录"
              : loginLabel}
        </Button>
        <Button
          size="sm"
          variant="secondary"
          loading={validateMutation.isPending}
          onClick={() => validateMutation.mutate()}
          disabled={!configured}
        >
          验证登录状态
        </Button>
        <Button
          size="sm"
          variant="secondary"
          onClick={() => setShowAdvanced((value) => !value)}
        >
          {showAdvanced ? "隐藏高级选项" : "高级：手动编辑 Cookie"}
        </Button>
      </div>

      {loginSession?.qrImageBase64 ? (
        <div className="flex flex-col items-start gap-2 rounded-md border border-slate-100 bg-slate-50 p-3">
          <div className="text-xs text-slate-500">
            {loginMessage ?? "请使用 B 站 App 扫描二维码"}
          </div>
          <img
            src={`data:image/png;base64,${loginSession.qrImageBase64}`}
            alt="Bilibili 登录二维码"
            className="h-44 w-44 rounded-md border border-slate-200 bg-white p-2"
          />
        </div>
      ) : null}

      {loginMessage && !loginSession?.qrImageBase64 && !validateMutation.data ? (
        <p className="text-xs text-slate-500">{loginMessage}</p>
      ) : null}

      {validateMutation.data ? (
        <p
          className={
            validateMutation.data.valid ? "text-xs text-slate-500" : "text-xs text-red-600"
          }
        >
          {validateMutation.data.message}
        </p>
      ) : null}

      {showAdvanced ? (
        <label className="block space-y-1">
          <span className="text-xs text-slate-500">Cookie（高级）</span>
          <textarea
            className="min-h-[72px] w-full rounded-md border border-slate-200 px-3 py-2 text-xs"
            value={draft[cookieField]}
            onChange={(event) =>
              onDraftChange({ ...draft, [cookieField]: event.target.value })
            }
            placeholder="仅高级用户手动粘贴 Cookie"
          />
        </label>
      ) : null}
    </div>
  );
}
