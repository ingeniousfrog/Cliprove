import { useCallback, useMemo, useRef, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Grid3x3, List, Loader2 } from "lucide-react";
import { searchAdapters } from "@/adapters";
import { Button } from "@/components/ui/button";
import { Card, CardBody } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { CoverImage } from "@/components/media/CoverImage";
import { MediaPreviewDialog } from "@/components/media/MediaPreviewDialog";
import { FfmpegRequiredDialog } from "@/components/setup/FfmpegRequiredDialog";
import { PlatformAuthDialog } from "@/components/setup/PlatformAuthDialog";
import { isAuthErrorCode, parseErrorCode } from "@/lib/errors";
import { batchItemsRequireFfmpeg } from "@/lib/ffmpeg";
import { enqueueDownloadBatch, ensureFfmpeg, searchMedia } from "@/lib/tauri";
import {
  cn,
  formatDuration,
  formatInvokeError,
  isAuthPlatform,
  platformLabel,
} from "@/lib/utils";
import type { AuthPlatform, MediaItem, Platform, SearchPage } from "@/types";

type ViewMode = "grid" | "table";

const BILIBILI_SORT_OPTIONS = [
  { value: "total", label: "综合排序" },
  { value: "click", label: "最多播放" },
  { value: "pubdate", label: "最新发布" },
  { value: "dm", label: "最多弹幕" },
];

const YOUTUBE_SORT_OPTIONS = [
  { value: "relevance", label: "相关性" },
  { value: "date", label: "最新上传" },
];

export function SearchPage() {
  const [platform, setPlatform] = useState<Platform>("bilibili");
  const [keyword, setKeyword] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [selected, setSelected] = useState<string[]>([]);
  const [results, setResults] = useState<MediaItem[]>([]);
  const [cursor, setCursor] = useState<string | undefined>();
  const [hasMore, setHasMore] = useState(false);
  const [sort, setSort] = useState("total");
  const [isSearching, setIsSearching] = useState(false);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [batchMessage, setBatchMessage] = useState<string | null>(null);
  const [previewItem, setPreviewItem] = useState<MediaItem | null>(null);
  const [ffmpegDialogOpen, setFfmpegDialogOpen] = useState(false);
  const [authDialogOpen, setAuthDialogOpen] = useState(false);
  const [authPlatform, setAuthPlatform] = useState<AuthPlatform>("bilibili");
  const [authBannerVisible, setAuthBannerVisible] = useState(false);
  const pendingBatchRef = useRef<MediaItem[]>([]);
  const queryClient = useQueryClient();
  const parentRef = useRef<HTMLDivElement>(null);

  const filters = useMemo(() => {
    if (platform === "bilibili" || platform === "youtube") return { sort };
    return undefined;
  }, [platform, sort]);

  const runSearch = useCallback(
    async (nextCursor?: string) => {
      const page = await searchMedia(
        platform,
        { keyword: keyword.trim(), pageSize: 24, filters },
        nextCursor
      );
      return page;
    },
    [platform, keyword, filters]
  );

  const handleSearch = async () => {
    if (!keyword.trim()) return;
    setIsSearching(true);
    setSearchError(null);
    setBatchMessage(null);
    setSelected([]);
    setHasSearched(true);
    try {
      const page = await runSearch();
      applyPage(page, false);
      setAuthBannerVisible(false);
    } catch (error) {
      setResults([]);
      setCursor(undefined);
      setHasMore(false);
      setSearchError(formatInvokeError(error));
      if (isAuthErrorCode(parseErrorCode(error)) && isAuthPlatform(platform)) {
        setAuthPlatform(platform);
        setAuthBannerVisible(true);
      } else {
        setAuthBannerVisible(false);
      }
    } finally {
      setIsSearching(false);
    }
  };

  const handleLoadMore = async () => {
    if (!cursor || isLoadingMore) return;
    setIsLoadingMore(true);
    setSearchError(null);
    try {
      const page = await runSearch(cursor);
      applyPage(page, true);
    } catch (error) {
      setSearchError(formatInvokeError(error));
    } finally {
      setIsLoadingMore(false);
    }
  };

  const applyPage = (page: SearchPage, append: boolean) => {
    setResults((current) => (append ? [...current, ...page.items] : page.items));
    setCursor(page.cursor);
    setHasMore(page.hasMore);
  };

  const buildDownloadOptions = (items: MediaItem[]) => {
    const assets = new Set<string>();
    for (const item of items) {
      const itemAssets =
        item.mediaType === "image_post"
          ? ["images", "cover", "metadata"]
          : item.platform === "bilibili" || item.platform === "youtube"
            ? ["video", "cover", "metadata", "subtitle"]
            : ["video", "cover", "metadata"];
      itemAssets.forEach((asset) => assets.add(asset));
    }

    return {
      assets: Array.from(assets),
      saveCover: assets.has("cover"),
      saveMetadata: assets.has("metadata"),
      saveSubtitles: assets.has("subtitle"),
    };
  };

  const runBatchDownload = async (items: MediaItem[]) => {
    if (items.length === 0) {
      throw new Error("请先选择要下载的视频");
    }
    if (batchItemsRequireFfmpeg(items)) {
      const status = await ensureFfmpeg();
      if (!status.valid) {
        pendingBatchRef.current = items;
        setFfmpegDialogOpen(true);
        return;
      }
    }
    const result = await enqueueDownloadBatch(items, buildDownloadOptions(items));
    if (result.enqueuedCount === 0) {
      const firstSkip = result.results.find((entry) => entry.message)?.message;
      throw new Error(firstSkip ?? "没有可加入队列的下载任务");
    }
    return result;
  };

  const batchMutation = useMutation({
    mutationFn: runBatchDownload,
    onSuccess: async (result) => {
      if (!result) return;
      const skipped = result.results.filter((entry) => entry.status === "skipped").length;
      setBatchMessage(
        skipped > 0
          ? `已加入 ${result.enqueuedCount} 个下载任务，跳过 ${skipped} 个（已在库中或正在下载）`
          : `已加入 ${result.enqueuedCount} 个下载任务`
      );
      setSelected([]);
      await queryClient.invalidateQueries({ queryKey: ["tasks"] });
      await queryClient.invalidateQueries({ queryKey: ["library"] });
    },
    onError: (error) => {
      setBatchMessage(formatInvokeError(error));
    },
  });

  const toggle = (id: string) => {
    setSelected((current) =>
      current.includes(id) ? current.filter((x) => x !== id) : [...current, id]
    );
  };

  const toggleAll = () => {
    if (selected.length === results.length) {
      setSelected([]);
    } else {
      setSelected(results.map((item) => item.platformItemId));
    }
  };

  const selectedItems = results.filter((item) =>
    selected.includes(item.platformItemId)
  );

  const rowVirtualizer = useVirtualizer({
    count: viewMode === "table" ? results.length : 0,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 52,
    overscan: 6,
  });

  const showBilibiliFilters = platform === "bilibili";
  const showYouTubeFilters = platform === "youtube";

  return (
    <div className="mx-auto flex h-full max-w-6xl flex-col gap-4 p-6">
      <div>
        <h1 className="text-xl font-semibold">关键词搜索</h1>
        <p className="mt-1 text-sm text-slate-500">
          Bilibili 与 YouTube 已接入真实搜索；YouTube 基于 yt-dlp，结果和分页可能受网络影响。
        </p>
      </div>

      {authBannerVisible ? (
        <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-amber-100 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <span>
            {authPlatform === "douyin"
              ? "抖音搜索需要有效登录；如果遇到滑块、答题等验证，请在浏览器登录窗口内完成后重试。"
              : "登录 Bilibili 后可搜索更多内容或下载高清资源。"}
          </span>
          <Button
            size="sm"
            onClick={() => {
              if (!isAuthPlatform(platform)) return;
              setAuthPlatform(platform);
              setAuthDialogOpen(true);
            }}
          >
            去登录
          </Button>
        </div>
      ) : null}

      <Card>
        <CardBody className="space-y-3">
          <div className="flex flex-wrap items-end gap-3">
            <div className="space-y-1">
              <label className="text-xs text-slate-500">平台</label>
              <select
                className="h-9 rounded-md border border-slate-200 bg-white px-3 text-sm"
                value={platform}
                onChange={(event) => {
                  const next = event.target.value as Platform;
                  setPlatform(next);
                  setResults([]);
                  setSelected([]);
                  setCursor(undefined);
                  setHasMore(false);
                  setHasSearched(false);
                  setSearchError(null);
                  setAuthBannerVisible(false);
                  setSort(next === "bilibili" ? "total" : "relevance");
                }}
              >
                {searchAdapters.map((item) => (
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
                onKeyDown={(event) => {
                  if (event.key === "Enter") void handleSearch();
                }}
              />
            </div>
            <Button
              loading={isSearching}
              onClick={() => void handleSearch()}
              disabled={!keyword.trim()}
            >
              {isSearching ? "搜索中…" : "搜索"}
            </Button>
            <div className="ml-auto flex gap-1">
              <Button
                size="sm"
                variant={viewMode === "grid" ? "primary" : "secondary"}
                onClick={() => setViewMode("grid")}
                aria-label="网格视图"
              >
                <Grid3x3 className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant={viewMode === "table" ? "primary" : "secondary"}
                onClick={() => setViewMode("table")}
                aria-label="表格视图"
              >
                <List className="h-4 w-4" />
              </Button>
            </div>
          </div>

          {showBilibiliFilters ? (
            <div className="flex flex-wrap gap-3 border-t border-slate-100 pt-3">
              <FilterSelect
                label="排序"
                value={sort}
                options={BILIBILI_SORT_OPTIONS}
                onChange={setSort}
              />
            </div>
          ) : null}

          {showYouTubeFilters ? (
            <div className="flex flex-wrap gap-3 border-t border-slate-100 pt-3">
              <FilterSelect
                label="排序"
                value={sort}
                options={YOUTUBE_SORT_OPTIONS}
                onChange={setSort}
              />
            </div>
          ) : null}
        </CardBody>
      </Card>

      {searchError ? (
        <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {searchError}
        </div>
      ) : null}

      {batchMessage ? (
        <div
          className={cn(
            "rounded-md border px-3 py-2 text-sm",
            batchMessage.startsWith("已加入")
              ? "border-emerald-200 bg-emerald-50 text-emerald-700"
              : "border-amber-200 bg-amber-50 text-amber-800"
          )}
        >
          {batchMessage}
        </div>
      ) : null}

      {results.length > 0 ? (
        <div className="flex items-center justify-between text-xs text-slate-500">
          <span>共 {results.length} 条结果</span>
          <button
            type="button"
            className="text-slate-600 hover:text-slate-900"
            onClick={toggleAll}
          >
            {selected.length === results.length ? "取消全选" : "全选当前结果"}
          </button>
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
            {batchMutation.isPending ? "加入队列…" : "批量下载"}
          </Button>
        </div>
      ) : null}

      <div
        ref={parentRef}
        className="h-[min(560px,calc(100vh-340px))] overflow-auto rounded-lg border border-slate-200 bg-white"
      >
        {results.length === 0 ? (
          <div className="flex h-48 flex-col items-center justify-center gap-1 px-6 text-center text-sm text-slate-400">
            {isSearching ? (
              <span>搜索中…</span>
            ) : hasSearched ? (
              <>
                <span>未找到相关结果</span>
                <span className="text-xs">
                  请尝试更换关键词或调整筛选条件
                </span>
              </>
            ) : (
              <>
                <span>输入关键词后，点击「搜索」开始</span>
                <span className="text-xs text-slate-300">
                  支持排序筛选与批量下载
                </span>
              </>
            )}
          </div>
        ) : viewMode === "grid" ? (
          <div className="grid w-full grid-cols-1 gap-3 p-3 sm:grid-cols-2 lg:grid-cols-3">
            {results.map((item) => (
              <ResultCard
                key={item.platformItemId}
                item={item}
                active={selected.includes(item.platformItemId)}
                onToggle={() => toggle(item.platformItemId)}
                onPreview={() => setPreviewItem(item)}
              />
            ))}
          </div>
        ) : (
          <div
            style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
            className="relative w-full"
          >
            <div className="sticky top-0 z-[1] grid grid-cols-[1fr_140px_80px_100px] border-b border-slate-100 bg-white px-3 py-2 text-xs text-slate-500">
              <span>标题</span>
              <span>作者</span>
              <span>时长</span>
              <span>类型</span>
            </div>
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const item = results[virtualRow.index];
              if (!item) return null;
              const active = selected.includes(item.platformItemId);
              return (
                <div
                  key={item.platformItemId}
                  className={cn(
                    "absolute left-0 grid w-full cursor-pointer grid-cols-[1fr_140px_80px_100px] border-b border-slate-50 px-3 py-2 text-sm hover:bg-slate-50",
                    active && "bg-slate-100"
                  )}
                  style={{ transform: `translateY(${virtualRow.start + 32}px)` }}
                  onClick={() => toggle(item.platformItemId)}
                >
                  <span className="truncate font-medium">{item.title}</span>
                  <span className="truncate text-slate-600">{item.author.name}</span>
                  <span className="text-slate-500">
                    {formatDuration(item.durationSec)}
                  </span>
                  <span>
                    <Badge>{item.mediaType}</Badge>
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {hasMore ? (
        <div className="flex justify-center pb-2">
          <Button
            variant="secondary"
            onClick={() => void handleLoadMore()}
            disabled={isLoadingMore}
          >
            {isLoadingMore ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                加载中…
              </>
            ) : (
              "加载更多"
            )}
          </Button>
        </div>
      ) : null}

      <MediaPreviewDialog item={previewItem} onClose={() => setPreviewItem(null)} />
      <FfmpegRequiredDialog
        open={ffmpegDialogOpen}
        onClose={() => {
          setFfmpegDialogOpen(false);
          pendingBatchRef.current = [];
        }}
        onReady={() => {
          const items = pendingBatchRef.current;
          if (items.length > 0) {
            batchMutation.mutate(items);
          }
        }}
      />
      <PlatformAuthDialog
        open={authDialogOpen}
        platform={authPlatform}
        onClose={() => setAuthDialogOpen(false)}
        onLoggedIn={() => {
          setAuthBannerVisible(false);
          if (keyword.trim()) {
            void handleSearch();
          }
        }}
      />
    </div>
  );
}

function FilterSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}) {
  return (
    <label className="space-y-1 text-xs text-slate-500">
      <span>{label}</span>
      <select
        className="block h-9 rounded-md border border-slate-200 bg-white px-3 text-sm text-slate-700"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function ResultCard({
  item,
  active,
  onToggle,
  onPreview,
}: {
  item: MediaItem;
  active: boolean;
  onToggle: () => void;
  onPreview: () => void;
}) {
  return (
    <div
      className={cn(
        "rounded-lg border p-3 text-left transition-colors",
        active
          ? "border-slate-900 bg-slate-50"
          : "border-slate-200 bg-white hover:border-slate-300"
      )}
    >
      <div className="mb-2 flex items-start gap-2">
        <input
          type="checkbox"
          checked={active}
          onChange={onToggle}
          className="mt-1"
          aria-label={`选择 ${item.title}`}
        />
        <button
          type="button"
          className="block flex-1 overflow-hidden rounded-md"
          onClick={onPreview}
        >
          <CoverImage
            src={item.coverUrl}
            platform={item.platform}
            className="aspect-video w-full"
          />
        </button>
      </div>
      <button type="button" className="w-full text-left" onClick={onToggle}>
        <div className="line-clamp-2 text-sm font-medium">{item.title}</div>
        <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-slate-500">
          <Badge>{platformLabel(item.platform)}</Badge>
          <span>{item.author.name}</span>
          <span>{formatDuration(item.durationSec)}</span>
        </div>
      </button>
    </div>
  );
}
