import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  getAppPaths,
  getSettings,
  ensureFfmpeg,
  sidecarHealth,
  startSidecar,
  updateSettings,
  validateFfmpeg,
} from "@/lib/tauri";
import { PlatformAuthCard } from "@/components/PlatformAuthCard";
import type { AppSettings } from "@/types";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);

  const [showAdvancedFfmpeg, setShowAdvancedFfmpeg] = useState(false);

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
  });

  const sidecarQuery = useQuery({
    queryKey: ["sidecar-health"],
    queryFn: sidecarHealth,
    retry: false,
  });

  const pathsQuery = useQuery({
    queryKey: ["app-paths"],
    queryFn: getAppPaths,
  });

  useEffect(() => {
    if (settingsQuery.data && !dirty) {
      setDraft(settingsQuery.data);
    }
  }, [settingsQuery.data, dirty]);

  const saveMutation = useMutation({
    mutationFn: (settings: Partial<AppSettings>) => updateSettings(settings),
    onSuccess: (data) => {
      setDraft(data);
      setDirty(false);
      setSaved(true);
      queryClient.setQueryData(["settings"], data);
      setTimeout(() => setSaved(false), 2000);
    },
  });

  const persistDraft = (next: AppSettings) => {
    setDraft(next);
    saveMutation.mutate(next);
  };

  const startSidecarMutation = useMutation({
    mutationFn: startSidecar,
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["sidecar-health"] }),
  });

  const ensureFfmpegMutation = useMutation({
    mutationFn: ensureFfmpeg,
    onSuccess: (status) => {
      if (!status.valid || !draft) return;
      const nextPath = status.resolvedPath?.trim() || draft.ffmpegPath;
      if (nextPath !== draft.ffmpegPath) {
        setDraft({ ...draft, ffmpegPath: nextPath });
        queryClient.setQueryData(["settings"], { ...draft, ffmpegPath: nextPath });
      }
    },
  });

  useEffect(() => {
    ensureFfmpegMutation.mutate();
  }, []);

  const validateFfmpegMutation = useMutation({
    mutationFn: () => validateFfmpeg(draft?.ffmpegPath ?? "ffmpeg"),
    onSuccess: (status) => {
      if (!status.valid || !draft) return;
      const nextPath = status.resolvedPath?.trim() || draft.ffmpegPath;
      if (nextPath === draft.ffmpegPath) return;
      persistDraft({ ...draft, ffmpegPath: nextPath });
    },
  });

  if (!draft) {
    return (
      <div className="p-6 text-sm text-slate-500">加载设置中…</div>
    );
  }

  const updateField = <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) => {
    setDirty(true);
    setDraft((current) => (current ? { ...current, [key]: value } : current));
  };

  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-4 p-6">
      <div>
        <h1 className="text-xl font-semibold">设置</h1>
        <p className="mt-1 text-sm text-slate-500">
          下载目录、并发、重试与平台认证配置。
        </p>
      </div>

      <Card>
        <CardHeader
          title="引擎服务"
          description="Python Sidecar 健康状态"
          action={
            <Button
              size="sm"
              variant="secondary"
              onClick={() => startSidecarMutation.mutate()}
              disabled={startSidecarMutation.isPending}
            >
              启动 Sidecar
            </Button>
          }
        />
        <CardBody>
          <div className="flex items-center gap-2 text-sm">
            状态：
            <Badge tone={sidecarQuery.data?.status === "ok" ? "success" : "warning"}>
              {sidecarQuery.data?.status ?? "未连接"}
            </Badge>
            {sidecarQuery.data?.version ? (
              <span className="text-slate-500">v{sidecarQuery.data.version}</span>
            ) : null}
          </div>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="下载" />
        <CardBody className="space-y-3">
          <Field label="下载目录">
            <Input
              value={draft.downloadDirectory}
              onChange={(event) =>
                updateField("downloadDirectory", event.target.value)
              }
            />
          </Field>
          <Field label="文件名模板">
            <Input
              value={draft.filenameTemplate}
              onChange={(event) =>
                updateField("filenameTemplate", event.target.value)
              }
            />
          </Field>
          <div className="grid gap-3 sm:grid-cols-2">
            <Field label="最大并发下载">
              <Input
                type="number"
                min={1}
                value={draft.maxConcurrentDownloads}
                onChange={(event) =>
                  updateField(
                    "maxConcurrentDownloads",
                    Number(event.target.value)
                  )
                }
              />
            </Field>
            <Field label="重试次数">
              <Input
                type="number"
                min={0}
                value={draft.retryCount}
                onChange={(event) =>
                  updateField("retryCount", Number(event.target.value))
                }
              />
            </Field>
          </div>
          <div className="rounded-md border border-slate-100 bg-slate-50 p-3 text-sm text-slate-600">
            <div className="flex flex-wrap items-center gap-2">
              <span className="text-xs text-slate-500">FFmpeg 状态</span>
              <Badge
                tone={
                  (ensureFfmpegMutation.data ?? validateFfmpegMutation.data)?.valid
                    ? "success"
                    : "warning"
                }
              >
                {(ensureFfmpegMutation.data ?? validateFfmpegMutation.data)?.valid
                  ? "已就绪"
                  : "未找到"}
              </Badge>
            </div>
            {(ensureFfmpegMutation.data ?? validateFfmpegMutation.data)?.resolvedPath ? (
              <div className="mt-2 break-all text-xs text-slate-500">
                {(ensureFfmpegMutation.data ?? validateFfmpegMutation.data)?.resolvedPath}
              </div>
            ) : (
              <p className="mt-2 text-xs text-slate-500">
                应用启动时会自动检测 FFmpeg。下载视频时若未找到会提示安装。
              </p>
            )}
            {(ensureFfmpegMutation.data ?? validateFfmpegMutation.data)?.message ? (
              <p className="mt-1 text-xs text-slate-500">
                {(ensureFfmpegMutation.data ?? validateFfmpegMutation.data)?.message}
              </p>
            ) : null}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="sm"
              variant="secondary"
              onClick={() => ensureFfmpegMutation.mutate()}
              disabled={ensureFfmpegMutation.isPending}
            >
              刷新检测
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setShowAdvancedFfmpeg((value) => !value)}
            >
              {showAdvancedFfmpeg ? "隐藏高级路径" : "高级：手动指定路径"}
            </Button>
          </div>
          {showAdvancedFfmpeg ? (
            <>
              <Field label="FFmpeg 路径">
                <Input
                  value={draft.ffmpegPath}
                  onChange={(event) => updateField("ffmpegPath", event.target.value)}
                  onBlur={() => {
                    if (!draft) return;
                    persistDraft(draft);
                  }}
                />
              </Field>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => validateFfmpegMutation.mutate()}
                disabled={validateFfmpegMutation.isPending}
              >
                验证路径
              </Button>
            </>
          ) : null}
        </CardBody>
      </Card>

      <Card>
        <CardHeader
          title="平台认证"
          description="扫码或浏览器登录，凭证仅保存在本地"
        />
        <CardBody className="space-y-4">
          <PlatformAuthCard
            platform="douyin"
            title="抖音"
            description="浏览器登录后可下载和搜索抖音内容；如遇平台验证，请在打开的浏览器内完成。"
            loginLabel="浏览器登录"
            cookieField="douyinCookies"
            draft={draft}
            onDraftChange={setDraft}
            onSaved={(settings) => {
              setDraft(settings);
              queryClient.setQueryData(["settings"], settings);
              setSaved(true);
              setTimeout(() => setSaved(false), 2000);
            }}
          />
          <PlatformAuthCard
            platform="bilibili"
            title="Bilibili"
            description="公开视频通常无需登录；扫码后可下载高画质或会员内容。"
            loginLabel="扫码登录"
            cookieField="bilibiliCookies"
            draft={draft}
            onDraftChange={setDraft}
            onSaved={(settings) => {
              setDraft(settings);
              queryClient.setQueryData(["settings"], settings);
              setSaved(true);
              setTimeout(() => setSaved(false), 2000);
            }}
          />
        </CardBody>
      </Card>

      <Card>
        <CardHeader
          title="首页体验"
          description="资源保存内容在每次下载前选择"
        />
        <CardBody className="space-y-2 text-sm">
          <ToggleRow
            label="自动检测剪贴板链接（进入首页时）"
            checked={draft.clipboardDetect}
            onChange={(value) => updateField("clipboardDetect", value)}
          />
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="本地路径" description="数据库、下载目录与日志位置" />
        <CardBody className="space-y-2 text-xs text-slate-600">
          <div>
            <div className="text-slate-500">数据库</div>
            <div className="break-all">{pathsQuery.data?.databasePath ?? "—"}</div>
          </div>
          <div>
            <div className="text-slate-500">下载目录</div>
            <div className="break-all">
              {pathsQuery.data?.downloadDirectory ?? draft.downloadDirectory}
            </div>
          </div>
          <div>
            <div className="text-slate-500">日志目录</div>
            <div className="break-all">{pathsQuery.data?.logDirectory ?? "—"}</div>
          </div>
        </CardBody>
      </Card>

      <div className="flex items-center gap-3">
        <Button onClick={() => draft && persistDraft(draft)} disabled={saveMutation.isPending}>
          {saveMutation.isPending ? "保存中…" : "保存设置"}
        </Button>
        {saved ? <span className="text-sm text-emerald-600">已保存</span> : null}
      </div>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block space-y-1">
      <span className="text-xs text-slate-500">{label}</span>
      {children}
    </label>
  );
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between rounded-md border border-slate-100 px-3 py-2">
      <span>{label}</span>
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
      />
    </label>
  );
}
