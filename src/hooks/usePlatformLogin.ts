import { useEffect, useRef, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import {
  getSettings,
  pollPlatformLogin,
  startPlatformLogin,
  updateSettings,
  validatePlatformAuth,
} from "@/lib/tauri";
import { formatInvokeError } from "@/lib/utils";
import type { AppSettings, Platform, PlatformLoginSession } from "@/types";

const TERMINAL_STATUSES = new Set(["completed", "failed", "expired"]);

interface UsePlatformLoginOptions {
  platform: Platform;
  cookieField: "douyinCookies" | "bilibiliCookies";
  onSaved?: (settings: AppSettings) => void;
}

export function usePlatformLogin({
  platform,
  cookieField,
  onSaved,
}: UsePlatformLoginOptions) {
  const [loginSession, setLoginSession] = useState<PlatformLoginSession | null>(
    null
  );
  const [loginMessage, setLoginMessage] = useState<string | null>(null);
  const savingRef = useRef(false);

  const validateMutation = useMutation({
    mutationFn: () => validatePlatformAuth(platform),
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
      onSaved?.(settings);
      validateMutation.mutate();
    },
    onError: (error) => {
      savingRef.current = false;
      setLoginMessage(`保存凭证失败：${formatInvokeError(error)}`);
    },
  });

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

  const startLogin = () => {
    validateMutation.reset();
    setLoginSession(null);
    setLoginMessage(
      platform === "douyin" ? "正在启动浏览器登录窗口…" : "正在准备登录…"
    );
    startLoginMutation.mutate();
  };

  const resetLogin = () => {
    setLoginSession(null);
    setLoginMessage(null);
    startLoginMutation.reset();
    validateMutation.reset();
  };

  return {
    loginSession,
    loginMessage,
    isLoginActive,
    startLogin,
    resetLogin,
    validateMutation,
    startLoginMutation,
    saveCookiesMutation,
  };
}
