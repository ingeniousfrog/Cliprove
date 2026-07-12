import { useEffect, useState } from "react";
import { ExternalLink, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { CoverImage } from "@/components/media/CoverImage";
import { embeddedPlayerUrl } from "@/lib/media";
import { resolveMediaPreview } from "@/lib/tauri";
import { formatDuration, platformLabel } from "@/lib/utils";
import type { MediaItem } from "@/types";

interface MediaPreviewDialogProps {
  item: MediaItem | null;
  onClose: () => void;
}

export function MediaPreviewDialog({ item, onClose }: MediaPreviewDialogProps) {
  const [resolvedPreviewUrl, setResolvedPreviewUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setResolvedPreviewUrl(null);

    if (!item || item.platform !== "bilibili") return;

    const loadPreviewUrl = async () => {
      try {
        const url = await resolveMediaPreview(item.platform, item.platformItemId);
        if (!cancelled) setResolvedPreviewUrl(url);
      } catch {
        if (!cancelled) setResolvedPreviewUrl(null);
      }
    };

    void loadPreviewUrl();
    return () => {
      cancelled = true;
    };
  }, [item?.platform, item?.platformItemId]);

  if (!item) return null;

  const embedUrl = embeddedPlayerUrl({
    platform: item.platform,
    platformItemId: item.platformItemId,
    previewUrl: resolvedPreviewUrl ?? item.previewUrl,
  });

  const openInBrowser = () => {
    window.open(item.canonicalUrl || item.originalUrl, "_blank", "noopener,noreferrer");
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="flex max-h-[90vh] w-full max-w-3xl flex-col overflow-hidden rounded-xl bg-white shadow-xl">
        <div className="flex items-start justify-between gap-3 border-b border-slate-100 px-4 py-3">
          <div className="min-w-0">
            <div className="text-sm font-medium text-slate-900">{item.title}</div>
            <div className="mt-1 text-xs text-slate-500">
              {platformLabel(item.platform)} · {item.author.name} ·{" "}
              {formatDuration(item.durationSec)}
            </div>
          </div>
          <button
            type="button"
            className="rounded-md p-1 text-slate-500 hover:bg-slate-100 hover:text-slate-800"
            onClick={onClose}
            aria-label="关闭预览"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="overflow-y-auto p-4">
          {embedUrl ? (
            <div className="overflow-hidden rounded-lg border border-slate-200 bg-black">
              <iframe
                title={item.title}
                src={embedUrl}
                className="aspect-video w-full"
                allow="autoplay; fullscreen; picture-in-picture"
                allowFullScreen
                referrerPolicy="strict-origin-when-cross-origin"
              />
            </div>
          ) : (
            <CoverImage
              src={item.coverUrl}
              platform={item.platform}
              className="aspect-video w-full rounded-lg"
            />
          )}

          {item.description ? (
            <p className="mt-3 line-clamp-4 text-sm text-slate-600">{item.description}</p>
          ) : null}
        </div>

        <div className="flex justify-end gap-2 border-t border-slate-100 px-4 py-3">
          <Button variant="secondary" size="sm" onClick={openInBrowser}>
            <ExternalLink className="mr-1 h-4 w-4" />
            在浏览器中打开
          </Button>
          <Button size="sm" onClick={onClose}>
            关闭
          </Button>
        </div>
      </div>
    </div>
  );
}
