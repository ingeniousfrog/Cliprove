import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  getSettings,
  sidecarHealth,
  startSidecar,
  updateSettings,
  validatePlatformAuth,
} from "@/lib/tauri";
import type { AppSettings } from "@/types";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState(false);

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
  });

  const sidecarQuery = useQuery({
    queryKey: ["sidecar-health"],
    queryFn: sidecarHealth,
    retry: false,
  });

  useEffect(() => {
    if (settingsQuery.data) {
      setDraft(settingsQuery.data);
    }
  }, [settingsQuery.data]);

  const saveMutation = useMutation({
    mutationFn: (settings: Partial<AppSettings>) => updateSettings(settings),
    onSuccess: (data) => {
      setDraft(data);
      setSaved(true);
      queryClient.setQueryData(["settings"], data);
      setTimeout(() => setSaved(false), 2000);
    },
  });

  const startSidecarMutation = useMutation({
    mutationFn: startSidecar,
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["sidecar-health"] }),
  });

  const validateDouyin = useMutation({
    mutationFn: () => validatePlatformAuth("douyin"),
  });

  const validateBilibili = useMutation({
    mutationFn: () => validatePlatformAuth("bilibili"),
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
          <Field label="FFmpeg 路径">
            <Input
              value={draft.ffmpegPath}
              onChange={(event) => updateField("ffmpegPath", event.target.value)}
            />
          </Field>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="平台认证" description="凭证仅保存在本地" />
        <CardBody className="space-y-3">
          <Field label="抖音 Cookie">
            <textarea
              className="min-h-[72px] w-full rounded-md border border-slate-200 px-3 py-2 text-xs"
              value={draft.douyinCookies}
              onChange={(event) =>
                updateField("douyinCookies", event.target.value)
              }
            />
          </Field>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="secondary"
              onClick={() => validateDouyin.mutate()}
            >
              验证抖音
            </Button>
            {validateDouyin.data ? (
              <span className="text-xs text-slate-500">
                {validateDouyin.data.message}
              </span>
            ) : null}
          </div>

          <Field label="Bilibili Cookie">
            <textarea
              className="min-h-[72px] w-full rounded-md border border-slate-200 px-3 py-2 text-xs"
              value={draft.bilibiliCookies}
              onChange={(event) =>
                updateField("bilibiliCookies", event.target.value)
              }
            />
          </Field>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="secondary"
              onClick={() => validateBilibili.mutate()}
            >
              验证 Bilibili
            </Button>
            {validateBilibili.data ? (
              <span className="text-xs text-slate-500">
                {validateBilibili.data.message}
              </span>
            ) : null}
          </div>
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="保存选项" />
        <CardBody className="space-y-2 text-sm">
          <ToggleRow
            label="保存元数据"
            checked={draft.saveMetadata}
            onChange={(value) => updateField("saveMetadata", value)}
          />
          <ToggleRow
            label="保存封面"
            checked={draft.saveCover}
            onChange={(value) => updateField("saveCover", value)}
          />
          <ToggleRow
            label="保存音频"
            checked={draft.saveAudio}
            onChange={(value) => updateField("saveAudio", value)}
          />
          <ToggleRow
            label="保存字幕"
            checked={draft.saveSubtitles}
            onChange={(value) => updateField("saveSubtitles", value)}
          />
        </CardBody>
      </Card>

      <div className="flex items-center gap-3">
        <Button onClick={() => saveMutation.mutate(draft)} disabled={saveMutation.isPending}>
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
