import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Copy,
  ExternalLink,
  FolderOpen,
  Grid3x3,
  List,
  MoreHorizontal,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  addToCollection,
  createCollection,
  createTag,
  deleteLibraryItem,
  deleteTag,
  listCollections,
  listLibrary,
  listTags,
  openLocalFile,
  readLocalFile,
  revealInFinder,
  setLibraryTags,
} from "@/lib/tauri";
import {
  cn,
  formatDate,
  formatDuration,
  mediaTypeLabel,
  platformLabel,
} from "@/lib/utils";
import type { LibraryFilter, LibraryItem, MediaType, Platform } from "@/types";

type ViewMode = "grid" | "table";
type DateRange = "all" | "7d" | "30d" | "90d";

const PLATFORMS: Array<{ value: Platform | ""; label: string }> = [
  { value: "", label: "全部平台" },
  { value: "douyin", label: "抖音" },
  { value: "bilibili", label: "Bilibili" },
];

const MEDIA_TYPES: Array<{ value: MediaType | ""; label: string }> = [
  { value: "", label: "全部类型" },
  { value: "video", label: "视频" },
  { value: "image_post", label: "图文" },
  { value: "multipart", label: "分 P" },
  { value: "audio", label: "音频" },
];

const DATE_RANGES: Array<{ value: DateRange; label: string }> = [
  { value: "all", label: "全部时间" },
  { value: "7d", label: "近 7 天" },
  { value: "30d", label: "近 30 天" },
  { value: "90d", label: "近 90 天" },
];

function dateRangeToFrom(range: DateRange): number | undefined {
  if (range === "all") return undefined;
  const days = range === "7d" ? 7 : range === "30d" ? 30 : 90;
  return Date.now() - days * 24 * 60 * 60 * 1000;
}

export function LibraryPage() {
  const queryClient = useQueryClient();
  const [query, setQuery] = useState("");
  const [platform, setPlatform] = useState<Platform | "">("");
  const [mediaType, setMediaType] = useState<MediaType | "">("");
  const [dateRange, setDateRange] = useState<DateRange>("all");
  const [collectionId, setCollectionId] = useState("");
  const [tagId, setTagId] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("table");
  const [activeItem, setActiveItem] = useState<LibraryItem | null>(null);
  const [metadataText, setMetadataText] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<LibraryItem | null>(null);
  const [deleteFiles, setDeleteFiles] = useState(false);
  const [newTagName, setNewTagName] = useState("");
  const [newCollectionName, setNewCollectionName] = useState("");

  const filter = useMemo<LibraryFilter>(() => {
    const value: LibraryFilter = {};
    if (query.trim()) value.query = query.trim();
    if (platform) value.platform = platform;
    if (mediaType) value.mediaType = mediaType;
    const dateFrom = dateRangeToFrom(dateRange);
    if (dateFrom) value.dateFrom = dateFrom;
    if (collectionId) value.collectionId = collectionId;
    if (tagId) value.tagId = tagId;
    return value;
  }, [query, platform, mediaType, dateRange, collectionId, tagId]);

  const { data: items = [], isLoading } = useQuery({
    queryKey: ["library", filter],
    queryFn: () => listLibrary(filter),
  });

  const tagsQuery = useQuery({
    queryKey: ["tags"],
    queryFn: listTags,
  });

  const collectionsQuery = useQuery({
    queryKey: ["collections"],
    queryFn: listCollections,
  });

  const deleteMutation = useMutation({
    mutationFn: ({ id, deleteFiles }: { id: string; deleteFiles: boolean }) =>
      deleteLibraryItem(id, deleteFiles),
    onSuccess: () => {
      setDeleteTarget(null);
      queryClient.invalidateQueries({ queryKey: ["library"] });
    },
  });

  const createTagMutation = useMutation({
    mutationFn: createTag,
    onSuccess: () => {
      setNewTagName("");
      queryClient.invalidateQueries({ queryKey: ["tags"] });
    },
  });

  const createCollectionMutation = useMutation({
    mutationFn: createCollection,
    onSuccess: () => {
      setNewCollectionName("");
      queryClient.invalidateQueries({ queryKey: ["collections"] });
    },
  });

  const copyText = async (text: string) => {
    await navigator.clipboard.writeText(text);
  };

  const openPrimaryMedia = async (item: LibraryItem) => {
    const path = item.mediaPaths[0] ?? item.coverPath;
    if (path) await openLocalFile(path);
  };

  const revealItem = async (item: LibraryItem) => {
    const path = item.mediaPaths[0] ?? item.coverPath ?? item.metadataPath;
    if (path) await revealInFinder(path);
  };

  const showMetadata = async (item: LibraryItem) => {
    setActiveItem(item);
    if (!item.metadataPath) {
      setMetadataText("无元数据文件");
      return;
    }
    try {
      const text = await readLocalFile(item.metadataPath);
      setMetadataText(text);
    } catch {
      setMetadataText("无法读取元数据文件");
    }
  };

  const toggleItemTag = async (item: LibraryItem, tagIdValue: string) => {
    const allTags = tagsQuery.data ?? [];
    const currentIds = allTags
      .filter((tag) => item.tags.includes(tag.name))
      .map((tag) => tag.id);
    const next = currentIds.includes(tagIdValue)
      ? currentIds.filter((id) => id !== tagIdValue)
      : [...currentIds, tagIdValue];
    await setLibraryTags(item.id, next);
    queryClient.invalidateQueries({ queryKey: ["library"] });
  };

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-4 p-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold">本地库</h1>
          <p className="mt-1 text-sm text-slate-500">
            全文搜索、筛选、标签与收藏夹；支持打开文件与 Finder 定位。
          </p>
        </div>
        <div className="flex gap-1">
          <Button
            size="sm"
            variant={viewMode === "grid" ? "primary" : "ghost"}
            onClick={() => setViewMode("grid")}
          >
            <Grid3x3 className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            variant={viewMode === "table" ? "primary" : "ghost"}
            onClick={() => setViewMode("table")}
          >
            <List className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <Card>
        <CardBody className="flex flex-col gap-3">
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索标题、作者、标签或平台 ID…"
          />
          <div className="flex flex-wrap gap-2">
            <FilterSelect
              value={platform}
              options={PLATFORMS}
              onChange={(value) => setPlatform(value as Platform | "")}
            />
            <FilterSelect
              value={mediaType}
              options={MEDIA_TYPES}
              onChange={(value) => setMediaType(value as MediaType | "")}
            />
            <FilterSelect
              value={dateRange}
              options={DATE_RANGES}
              onChange={(value) => setDateRange(value as DateRange)}
            />
            <FilterSelect
              value={collectionId}
              options={[
                { value: "", label: "全部收藏夹" },
                ...(collectionsQuery.data ?? []).map((collection) => ({
                  value: collection.id,
                  label: `${collection.name} (${collection.itemCount})`,
                })),
              ]}
              onChange={setCollectionId}
            />
            <FilterSelect
              value={tagId}
              options={[
                { value: "", label: "全部标签" },
                ...(tagsQuery.data ?? []).map((tag) => ({
                  value: tag.id,
                  label: tag.name,
                })),
              ]}
              onChange={setTagId}
            />
          </div>
        </CardBody>
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader title="标签" description="为库条目打标签以便搜索" />
          <CardBody className="flex flex-col gap-2">
            <div className="flex gap-2">
              <Input
                value={newTagName}
                onChange={(event) => setNewTagName(event.target.value)}
                placeholder="新建标签"
              />
              <Button
                size="sm"
                onClick={() => createTagMutation.mutate(newTagName)}
                disabled={!newTagName.trim()}
              >
                添加
              </Button>
            </div>
            <div className="flex flex-wrap gap-2">
              {(tagsQuery.data ?? []).map((tag) => (
                <Badge key={tag.id}>
                  <span className="inline-flex items-center gap-2">
                    {tag.name}
                    <button
                      type="button"
                      className="text-slate-400 hover:text-red-500"
                      onClick={() => {
                        deleteTag(tag.id).then(() =>
                          queryClient.invalidateQueries({ queryKey: ["tags"] })
                        );
                      }}
                    >
                      ×
                    </button>
                  </span>
                </Badge>
              ))}
            </div>
          </CardBody>
        </Card>

        <Card>
          <CardHeader title="收藏夹" description="将条目归类到收藏夹" />
          <CardBody className="flex flex-col gap-2">
            <div className="flex gap-2">
              <Input
                value={newCollectionName}
                onChange={(event) => setNewCollectionName(event.target.value)}
                placeholder="新建收藏夹"
              />
              <Button
                size="sm"
                onClick={() =>
                  createCollectionMutation.mutate(newCollectionName)
                }
                disabled={!newCollectionName.trim()}
              >
                添加
              </Button>
            </div>
            <div className="flex flex-wrap gap-2">
              {(collectionsQuery.data ?? []).map((collection) => (
                <Badge key={collection.id}>{collection.name}</Badge>
              ))}
            </div>
          </CardBody>
        </Card>
      </div>

      <Card>
        <CardHeader title="库条目" description={`共 ${items.length} 条`} />
        <CardBody>
          {isLoading ? (
            <p className="text-sm text-slate-500">加载中…</p>
          ) : items.length === 0 ? (
            <p className="text-sm text-slate-500">没有匹配的库条目</p>
          ) : viewMode === "grid" ? (
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {items.map((item) => (
                <LibraryCard
                  key={item.id}
                  item={item}
                  tags={tagsQuery.data ?? []}
                  collections={collectionsQuery.data ?? []}
                  onOpen={() => openPrimaryMedia(item)}
                  onReveal={() => revealItem(item)}
                  onCopy={() => copyText(item.originalUrl)}
                  onMetadata={() => showMetadata(item)}
                  onDelete={() => {
                    setDeleteTarget(item);
                    setDeleteFiles(false);
                  }}
                  onToggleTag={(tagIdValue) => toggleItemTag(item, tagIdValue)}
                  onAddToCollection={(collectionIdValue) =>
                    addToCollection(collectionIdValue, item.id).then(() =>
                      queryClient.invalidateQueries({ queryKey: ["collections"] })
                    )
                  }
                />
              ))}
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full text-left text-sm">
                <thead className="border-b border-slate-100 text-xs text-slate-500">
                  <tr>
                    <th className="px-2 py-2 font-medium">标题</th>
                    <th className="px-2 py-2 font-medium">平台</th>
                    <th className="px-2 py-2 font-medium">类型</th>
                    <th className="px-2 py-2 font-medium">作者</th>
                    <th className="px-2 py-2 font-medium">标签</th>
                    <th className="px-2 py-2 font-medium">下载时间</th>
                    <th className="px-2 py-2 font-medium">操作</th>
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) => (
                    <tr key={item.id} className="border-b border-slate-50">
                      <td className="max-w-[220px] truncate px-2 py-2">
                        {item.title}
                      </td>
                      <td className="px-2 py-2">
                        <Badge>{platformLabel(item.platform)}</Badge>
                      </td>
                      <td className="px-2 py-2">
                        {mediaTypeLabel(item.mediaType)}
                      </td>
                      <td className="px-2 py-2">{item.authorName}</td>
                      <td className="px-2 py-2">
                        <div className="flex flex-wrap gap-1">
                          {item.tags.length
                            ? item.tags.map((tag) => (
                                <Badge key={tag}>{tag}</Badge>
                              ))
                            : "—"}
                        </div>
                      </td>
                      <td className="px-2 py-2 text-xs text-slate-500">
                        {formatDate(item.createdAt)}
                      </td>
                      <td className="px-2 py-2">
                        <ItemActions
                          item={item}
                          onOpen={() => openPrimaryMedia(item)}
                          onReveal={() => revealItem(item)}
                          onCopy={() => copyText(item.originalUrl)}
                          onMetadata={() => showMetadata(item)}
                          onDelete={() => {
                            setDeleteTarget(item);
                            setDeleteFiles(false);
                          }}
                        />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </CardBody>
      </Card>

      {metadataText !== null && activeItem ? (
        <Card>
          <CardHeader
            title={`元数据 · ${activeItem.title}`}
            description={activeItem.metadataPath}
          />
          <CardBody>
            <pre className="max-h-80 overflow-auto rounded-lg bg-slate-50 p-3 text-xs text-slate-700">
              {metadataText}
            </pre>
            <Button
              size="sm"
              variant="ghost"
              className="mt-2"
              onClick={() => {
                setMetadataText(null);
                setActiveItem(null);
              }}
            >
              关闭
            </Button>
          </CardBody>
        </Card>
      ) : null}

      {deleteTarget ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4">
          <Card className="w-full max-w-md">
            <CardHeader
              title="删除库条目"
              description={`确定删除「${deleteTarget.title}」？`}
            />
            <CardBody className="flex flex-col gap-3">
              <label className="flex items-center gap-2 text-sm text-slate-600">
                <input
                  type="checkbox"
                  checked={deleteFiles}
                  onChange={(event) => setDeleteFiles(event.target.checked)}
                />
                同时删除本地文件（不可恢复）
              </label>
              <div className="flex justify-end gap-2">
                <Button
                  variant="ghost"
                  onClick={() => setDeleteTarget(null)}
                >
                  取消
                </Button>
                <Button
                  variant="danger"
                  onClick={() =>
                    deleteMutation.mutate({
                      id: deleteTarget.id,
                      deleteFiles,
                    })
                  }
                >
                  确认删除
                </Button>
              </div>
            </CardBody>
          </Card>
        </div>
      ) : null}
    </div>
  );
}

function FilterSelect<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: Array<{ value: T; label: string }>;
  onChange: (value: T) => void;
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value as T)}
      className="rounded-lg border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700"
    >
      {options.map((option) => (
        <option key={option.value || "all"} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}

function ItemActions({
  onOpen,
  onReveal,
  onCopy,
  onMetadata,
  onDelete,
}: {
  item: LibraryItem;
  onOpen: () => void;
  onReveal: () => void;
  onCopy: () => void;
  onMetadata: () => void;
  onDelete: () => void;
}) {
  return (
    <div className="flex flex-wrap gap-1">
      <ActionButton icon={ExternalLink} label="打开" onClick={onOpen} />
      <ActionButton icon={FolderOpen} label="显示" onClick={onReveal} />
      <ActionButton icon={Copy} label="复制链接" onClick={onCopy} />
      <ActionButton icon={MoreHorizontal} label="元数据" onClick={onMetadata} />
      <ActionButton icon={Trash2} label="删除" onClick={onDelete} />
    </div>
  );
}

function ActionButton({
  icon: Icon,
  label,
  onClick,
}: {
  icon: typeof ExternalLink;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button size="sm" variant="ghost" onClick={onClick} title={label}>
      <Icon className="h-4 w-4" />
    </Button>
  );
}

function LibraryCard({
  item,
  tags,
  collections,
  onOpen,
  onReveal,
  onCopy,
  onMetadata,
  onDelete,
  onToggleTag,
  onAddToCollection,
}: {
  item: LibraryItem;
  tags: Array<{ id: string; name: string }>;
  collections: Array<{ id: string; name: string }>;
  onOpen: () => void;
  onReveal: () => void;
  onCopy: () => void;
  onMetadata: () => void;
  onDelete: () => void;
  onToggleTag: (tagId: string) => void;
  onAddToCollection: (collectionId: string) => void;
}) {
  return (
    <div className="rounded-xl border border-slate-100 p-3">
      <div className="mb-2 flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="truncate font-medium">{item.title}</p>
          <p className="text-xs text-slate-500">
            {platformLabel(item.platform)} · {item.authorName} ·{" "}
            {formatDuration(item.durationSec)}
          </p>
        </div>
        <Badge>{mediaTypeLabel(item.mediaType)}</Badge>
      </div>
      <div className="mb-2 flex flex-wrap gap-1">
        {item.tags.map((tag) => (
          <Badge key={tag}>{tag}</Badge>
        ))}
      </div>
      <ItemActions
        item={item}
        onOpen={onOpen}
        onReveal={onReveal}
        onCopy={onCopy}
        onMetadata={onMetadata}
        onDelete={onDelete}
      />
      <details className="mt-2 text-xs text-slate-500">
        <summary className="cursor-pointer">标签 / 收藏夹</summary>
        <div className="mt-2 flex flex-wrap gap-1">
          {tags.map((tag) => (
            <button
              key={tag.id}
              type="button"
              className={cn(
                "rounded-full border px-2 py-0.5",
                item.tags.includes(tag.name)
                  ? "border-sky-300 bg-sky-50 text-sky-700"
                  : "border-slate-200"
              )}
              onClick={() => onToggleTag(tag.id)}
            >
              {tag.name}
            </button>
          ))}
        </div>
        <div className="mt-2 flex flex-wrap gap-1">
          {collections.map((collection) => (
            <button
              key={collection.id}
              type="button"
              className="rounded-full border border-slate-200 px-2 py-0.5"
              onClick={() => onAddToCollection(collection.id)}
            >
              + {collection.name}
            </button>
          ))}
        </div>
      </details>
    </div>
  );
}
