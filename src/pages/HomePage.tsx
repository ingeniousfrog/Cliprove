import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { TaskActionButtons } from "@/components/tasks/TaskActionButtons";
import { CoverImage } from "@/components/media/CoverImage";
import { MediaPreviewDialog } from "@/components/media/MediaPreviewDialog";
import { detectAdapter } from "@/adapters";
import { useTaskActions } from "@/hooks/useTaskActions";
import { enqueueDownload, getSettings, listTasks, parseLink } from "@/lib/tauri";
import { useAppStore } from "@/stores/app";
import {
  formatDate,
  formatDuration,
  platformLabel,
  statusLabel,
} from "@/lib/utils";
import type { DownloadOptions, DownloadProgress, MediaItem } from "@/types";

export function HomePage() {
  const [url, setUrl] = useState("");
  const [selectedAssets, setSelectedAssets] = useState<string[]>([]);
  const [selectedQuality, setSelectedQuality] = useState<string>("best");
  const queryClient = useQueryClient();
  const { parsedMedia, setParsedMedia } = useAppStore();
  const { pendingAction, runAction } = useTaskActions();
  const [pasting, setPasting] = useState(false);
  const [previewItem, setPreviewItem] = useState<MediaItem | null>(null);

  const detected = url.trim() ? detectAdapter(url.trim()) : undefined;

  const parseMutation = useMutation({
    mutationFn: () => {
      if (!detected) {
        throw new Error("当前版本暂只支持 Bilibili 链接");
      }
      return parseLink(url.trim());
    },
    onSuccess: (data) => {
      setParsedMedia(data);
      setSelectedAssets(data.assets.map((asset) => asset.id));
      setSelectedQuality(data.qualities?.[0]?.id ?? "best");
    },
  });

  const downloadMutation = useMutation({
    mutationFn: async () => {
      if (!parsedMedia) return;
      const options: DownloadOptions = {
        assets: selectedAssets,
        qualityId: selectedQuality,
        saveCover: selectedAssets.includes("cover"),
        saveAudio: selectedAssets.includes("audio"),
        saveMetadata: selectedAssets.includes("metadata"),
        saveSubtitles: selectedAssets.includes("subtitle"),
      };
      await enqueueDownload(parsedMedia.item, options);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["tasks"] });
      await queryClient.invalidateQueries({ queryKey: ["library"] });
    },
  });

  const { data: recentTasks = [] } = useQuery({
    queryKey: ["tasks"],
    queryFn: listTasks,
    refetchInterval: (query) => {
      const items = query.state.data ?? [];
      const hasActive = items.some((task) =>
        ["queued", "parsing", "downloading", "post_processing"].includes(task.status)
      );
      return hasActive ? 1000 : 5000;
    },
  });

  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
  });

  const pasteFromClipboard = async () => {
    setPasting(true);
    try {
      const text = await navigator.clipboard.readText();
      if (text.trim()) setUrl(text.trim());
    } catch {
      // Clipboard permission may be denied; ignore silently.
    } finally {
      setPasting(false);
    }
  };

  useEffect(() => {
    const unlisten = listen<DownloadProgress>("download-progress", () => {
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
    });
    return () => {
      unlisten.then((dispose) => dispose());
    };
  }, [queryClient]);

  useEffect(() => {
    if (!settingsQuery.data?.clipboardDetect) return;

    const detectClipboard = async () => {
      try {
        const text = await navigator.clipboard.readText();
        if (!text.trim() || url.trim()) return;
        if (detectAdapter(text.trim())) setUrl(text.trim());
      } catch {
        // Clipboard permission may be denied; ignore silently.
      }
    };

    detectClipboard();
    const onFocus = () => detectClipboard();
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [settingsQuery.data?.clipboardDetect, url]);

  const toggleAsset = (assetId: string) => {
    setSelectedAssets((current) =>
      current.includes(assetId)
        ? current.filter((id) => id !== assetId)
        : [...current, assetId]
    );
  };

  return (
    <div className="mx-auto flex max-w-5xl flex-col gap-4 p-6">
      <div>
        <h1 className="text-xl font-semibold">粘贴链接</h1>
        <p className="mt-1 text-sm text-slate-500">
          自动识别平台，解析后预览并选择要保存的资源。
        </p>
      </div>

      <Card>
        <CardHeader title="分享链接" />
        <CardBody className="space-y-3">
          <Textarea
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            placeholder="粘贴 Bilibili 分享链接…"
          />
          <div className="flex items-center justify-between gap-3">
            <div className="text-xs text-slate-500">
              {detected ? (
                <span>
                  已识别平台：<Badge>{detected.name}</Badge>
                </span>
              ) : (
                "等待输入 Bilibili 链接"
              )}
            </div>
            <div className="flex gap-2">
              <Button
                variant="secondary"
                size="sm"
                loading={pasting}
                onClick={pasteFromClipboard}
              >
                {pasting ? "粘贴中…" : "从剪贴板粘贴"}
              </Button>
              <Button
                loading={parseMutation.isPending}
                onClick={() => parseMutation.mutate()}
                disabled={!url.trim() || !detected}
              >
                {parseMutation.isPending ? "解析中…" : "解析链接"}
              </Button>
            </div>
          </div>
          {parseMutation.isError ? (
            <p className="text-sm text-red-600">
              {(parseMutation.error as Error).message}
            </p>
          ) : null}
        </CardBody>
      </Card>

      {parsedMedia ? (
        <Card>
          <CardHeader
            title={parsedMedia.item.title}
            description={`${platformLabel(parsedMedia.item.platform)} · ${parsedMedia.item.author.name}`}
          />
          <CardBody className="space-y-4">
            <div className="grid gap-3 sm:grid-cols-2">
              <button
                type="button"
                className="overflow-hidden rounded-md border border-slate-200 text-left transition-colors hover:border-slate-300"
                onClick={() => setPreviewItem(parsedMedia.item)}
              >
                <CoverImage
                  src={parsedMedia.item.coverUrl}
                  platform={parsedMedia.item.platform}
                  className="h-44 w-full"
                />
              </button>
              <div className="space-y-2 text-sm text-slate-600">
                <div>时长：{formatDuration(parsedMedia.item.durationSec)}</div>
                <div>类型：{parsedMedia.item.mediaType}</div>
                <div>发布时间：{formatDate(parsedMedia.item.publishedAt)}</div>
                <p className="line-clamp-4 text-slate-500">
                  {parsedMedia.item.description || "无描述"}
                </p>
              </div>
            </div>

            <div>
              <div className="mb-2 text-sm font-medium text-slate-800">
                选择要保存的资源
              </div>
              <div className="flex flex-wrap gap-2">
                {parsedMedia.assets.map((asset) => {
                  const active = selectedAssets.includes(asset.id);
                  return (
                    <button
                      key={asset.id}
                      type="button"
                      onClick={() => toggleAsset(asset.id)}
                      className={`rounded-md border px-3 py-1.5 text-xs transition-all duration-100 active:scale-[0.98] ${
                        active
                          ? "border-slate-900 bg-slate-900 text-white"
                          : "border-slate-200 bg-white text-slate-600"
                      }`}
                    >
                      {asset.label}
                    </button>
                  );
                })}
              </div>
            </div>

            {parsedMedia.qualities && parsedMedia.qualities.length > 0 ? (
              <div className="space-y-1">
                <div className="text-sm font-medium text-slate-800">清晰度</div>
                <select
                  className="h-9 rounded-md border border-slate-200 bg-white px-3 text-sm"
                  value={selectedQuality}
                  onChange={(event) => setSelectedQuality(event.target.value)}
                >
                  {parsedMedia.qualities.map((quality) => (
                    <option key={quality.id} value={quality.id}>
                      {quality.label}
                    </option>
                  ))}
                </select>
              </div>
            ) : null}

            <div className="flex justify-end gap-2">
              <Button
                variant="secondary"
                onClick={() => setPreviewItem(parsedMedia.item)}
              >
                预览
              </Button>
              <Button
                loading={downloadMutation.isPending}
                onClick={() => downloadMutation.mutate()}
                disabled={selectedAssets.length === 0}
              >
                {downloadMutation.isPending ? "加入队列…" : "开始下载"}
              </Button>
            </div>
          </CardBody>
        </Card>
      ) : null}

      <Card>
        <CardHeader title="最近任务" description="展示最近 5 条任务" />
        <CardBody>
          {recentTasks.length === 0 ? (
            <p className="text-sm text-slate-500">暂无任务</p>
          ) : (
            <div className="space-y-2">
              {recentTasks.slice(0, 5).map((task) => (
                <div
                  key={task.id}
                  className="flex flex-wrap items-center justify-between gap-3 rounded-md border border-slate-100 px-3 py-2 text-sm"
                >
                  <div className="min-w-0 flex-1">
                    <div className="truncate font-medium">{task.title}</div>
                    <div className="text-xs text-slate-500">
                      {platformLabel(task.platform)}
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge
                      tone={
                        task.status === "completed"
                          ? "success"
                          : task.status === "failed"
                            ? "danger"
                            : task.status === "interrupted"
                              ? "warning"
                              : "default"
                      }
                    >
                      {statusLabel(task.status)}
                    </Badge>
                    <TaskActionButtons
                      task={task}
                      pendingAction={pendingAction}
                      onAction={runAction}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardBody>
      </Card>

      <MediaPreviewDialog item={previewItem} onClose={() => setPreviewItem(null)} />
    </div>
  );
}
