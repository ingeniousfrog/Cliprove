import { useEffect, useRef, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  getSettings,
  pollPlatformLogin,
  startPlatformLogin,
  updateSettings,
  validatePlatformAuth,
} from "@/lib/tauri";
import { formatInvokeError } from "@/lib/utils";
import type { AppSettings, Platform, PlatformLoginSession } from "@/types";

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

const TERMINAL_STATUSES = new Set(["completed", "failed", "expired"]);

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
  const [loginSession, setLoginSession] = useState<PlatformLoginSession | null>(
    null
  );
  const [loginMessage, setLoginMessage] = useState<string | null>(null);
  const [configured, setConfigured] = useState(false);
  const savingRef = useRef(false);

  const validateMutation = useMutation({
    mutationFn: () => validatePlatformAuth(platform),
    onSuccess: (status) => {
      setConfigured(status.valid || Boolean(draft[cookieField].trim()));
    },
  });

  const startLoginMutation = useMutation({
    mutationFn: () => startPlatformLogin(platform),
    onSuccess: (session) => {
      setLoginSession(session);
      setLoginMessage(session.message ?? null);
    },
    onError: (error) => {
      setLoginMessage(formatInvokeError(error));
    },
  });

  const saveCookiesMutation = useMutation({
    mutationFn: async (cookies: string) => {
      const current = await getSettings();
      return updateSettings({ ...current, [cookieField]: cookies });
    },
    onSuccess: (settings) => {
      savingRef.current = false;
      onDraftChange(settings);
      onSaved(settings);
      setConfigured(true);
      setLoginMessage("登录成功，凭证已自动保存");
      validateMutation.mutate();
    },
    onError: (error) => {
      savingRef.current = false;
      setLoginMessage(`保存凭证失败：${formatInvokeError(error)}`);
    },
  });

  useEffect(() => {
    setConfigured(Boolean(draft[cookieField].trim()));
  }, [cookieField, draft]);

  useEffect(() => {
    if (!loginSession || TERMINAL_STATUSES.has(loginSession.status)) {
      return;
    }

    let cancelled = false;
    const sessionId = loginSession.sessionId;

    const poll = async () => {
      while (!cancelled) {
        try {
          const next = await pollPlatformLogin(sessionId);
          if (cancelled) return;

          setLoginSession(next);
          setLoginMessage(next.message ?? null);

          if (next.status === "completed") {
            if (next.cookies && !savingRef.current) {
              savingRef.current = true;
              saveCookiesMutation.mutate(next.cookies);
            } else if (!next.cookies) {
              setLoginMessage("登录成功，但未获取到凭证，请重试");
            }
            break;
          }
          if (TERMINAL_STATUSES.has(next.status)) {
            break;
          }
        } catch (error) {
          if (!cancelled) {
            setLoginMessage(formatInvokeError(error));
          }
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 1500));
      }
    };

    void poll();
    return () => {
      cancelled = true;
    };
  }, [loginSession?.sessionId]);

  const isLoginActive =
    loginSession !== null && !TERMINAL_STATUSES.has(loginSession.status);

  const handleStartLogin = () => {
    setLoginSession(null);
    setLoginMessage(
      platform === "douyin"
        ? "正在启动浏览器登录窗口…"
        : "正在准备登录…"
    );
    startLoginMutation.mutate();
  };

  return (
    <div className="space-y-3 rounded-md border border-slate-100 p-3">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="text-sm font-medium text-slate-800">{title}</div>
          <p className="mt-1 text-xs text-slate-500">{description}</p>
        </div>
        <Badge tone={configured ? "success" : "default"}>
          {configured ? "已配置" : "未配置"}
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

      {loginMessage && !loginSession?.qrImageBase64 ? (
        <p
          className={
            validateMutation.data?.valid === false && !isLoginActive
              ? "text-xs text-red-600"
              : "text-xs text-slate-500"
          }
        >
          {loginMessage}
        </p>
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
