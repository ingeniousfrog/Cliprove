import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { adapters } from "@/adapters";
import { Button } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { enqueueDownload, searchMedia } from "@/lib/tauri";
import { formatDuration, platformLabel } from "@/lib/utils";
import type { MediaItem, Platform } from "@/types";

export function SearchPage() {
  const [platform, setPlatform] = useState<Platform>("douyin");
  const [keyword, setKeyword] = useState("");
  const [selected, setSelected] = useState<string[]>([]);
  const queryClient = useQueryClient();

  const adapter = useMemo(
    () => adapters.find((item) => item.id === platform)!,
    [platform]
  );

  const searchQuery = useQuery({
    queryKey: ["search", platform, keyword],
    queryFn: () => searchMedia(platform, { keyword, pageSize: 20 }),
    enabled: false,
  });

  const batchMutation = useMutation({
    mutationFn: async (items: MediaItem[]) => {
      for (const item of items) {
        await enqueueDownload(item, {
          assets: ["video", "cover", "metadata"],
          saveCover: true,
          saveMetadata: true,
        });
      }
    },
    onSuccess: async () => {
      setSelected([]);
      await queryClient.invalidateQueries({ queryKey: ["tasks"] });
    },
  });

  const results = searchQuery.data?.items ?? [];

  const toggle = (id: string) => {
    setSelected((current) =>
      current.includes(id) ? current.filter((x) => x !== id) : [...current, id]
    );
  };

  const selectedItems = results.filter((item) =>
    selected.includes(item.platformItemId)
  );

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-4 p-6">
      <div>
        <h1 className="text-xl font-semibold">关键词搜索</h1>
        <p className="mt-1 text-sm text-slate-500">
          选择平台后搜索，支持多选批量加入下载队列。
        </p>
      </div>

      <Card>
        <CardBody className="flex flex-wrap items-end gap-3">
          <div className="space-y-1">
            <label className="text-xs text-slate-500">平台</label>
            <select
              className="h-9 rounded-md border border-slate-200 bg-white px-3 text-sm"
              value={platform}
              onChange={(event) =>
                setPlatform(event.target.value as Platform)
              }
            >
              {adapters.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name}
                </option>
              ))}
            </select>
          </div>
          <div className="min-w-[280px] flex-1 space-y-1">
            <label className="text-xs text-slate-500">关键词</label>
            <Input
              value={keyword}
              onChange={(event) => setKeyword(event.target.value)}
              placeholder="输入搜索关键词"
            />
          </div>
          <Button
            onClick={() => searchQuery.refetch()}
            disabled={!keyword.trim() || searchQuery.isFetching}
          >
            {searchQuery.isFetching ? "搜索中…" : "搜索"}
          </Button>
        </CardBody>
      </Card>

      {searchQuery.data ? (
        <div className="text-xs text-slate-500">
          支持筛选：
          {searchQuery.data.supportedFilters.length > 0
            ? searchQuery.data.supportedFilters.join("、")
            : "无"}
          {adapter.supportedFilters.length === 0 ? "（当前平台未声明筛选）" : ""}
        </div>
      ) : null}

      {selected.length > 0 ? (
        <div className="sticky top-0 z-10 flex items-center justify-between rounded-lg border border-slate-200 bg-white px-4 py-2 shadow-sm">
          <span className="text-sm text-slate-600">已选 {selected.length} 项</span>
          <Button
            size="sm"
            onClick={() => batchMutation.mutate(selectedItems)}
            disabled={batchMutation.isPending}
          >
            批量下载
          </Button>
        </div>
      ) : null}

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {results.map((item) => {
          const active = selected.includes(item.platformItemId);
          return (
            <button
              key={item.platformItemId}
              type="button"
              onClick={() => toggle(item.platformItemId)}
              className={`rounded-lg border p-3 text-left transition-colors ${
                active
                  ? "border-slate-900 bg-slate-50"
                  : "border-slate-200 bg-white hover:border-slate-300"
              }`}
            >
              <div className="mb-2 aspect-video overflow-hidden rounded-md bg-slate-100">
                {item.coverUrl ? (
                  <img
                    src={item.coverUrl}
                    alt=""
                    className="h-full w-full object-cover"
                  />
                ) : null}
              </div>
              <div className="line-clamp-2 text-sm font-medium">{item.title}</div>
              <div className="mt-1 flex items-center gap-2 text-xs text-slate-500">
                <Badge>{platformLabel(item.platform)}</Badge>
                <span>{item.author.name}</span>
                <span>{formatDuration(item.durationSec)}</span>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
